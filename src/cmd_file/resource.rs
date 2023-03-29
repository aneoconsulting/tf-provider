use std::fmt::Debug;
use std::io::ErrorKind;

use async_trait::async_trait;
use crypto::md5::Md5;
use crypto::sha1::Sha1;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use serde::{Deserialize, Serialize};
use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributePath, AttributeType, Block, Description,
    Diagnostics, NestedBlock, Resource, Schema, Value, ValueEmpty, ValueString,
};

use crate::cmd_file::hash_stream::HashingStream;
use crate::connection::Connection;
use crate::utils::WithNormalize;

#[derive(Debug, Default)]
pub struct CmdFileResource<T: Connection> {
    pub(super) connect: T,
}

impl<T: Connection> CmdFileResource<T> {
    pub fn new(connect: T) -> Self {
        Self { connect }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceState<'a, T>
where
    T: Connection,
{
    #[serde(borrow = "'a")]
    pub id: ValueString<'a>,
    pub path: ValueString<'a>,
    pub content: ValueString<'a>,
    pub mode: ValueString<'a>,
    pub overwrite: Value<bool>,
    pub keep: Value<bool>,
    pub md5: ValueString<'a>,
    pub sha1: ValueString<'a>,
    #[serde(with = "value::serde_as_vec")]
    pub connect: Value<T::Config<'a>>,
}

#[async_trait]
impl<T> Resource for CmdFileResource<T>
where
    T: Connection,
    T: Debug,
    T: Clone,
{
    type State<'a> = ResourceState<'a, T>;
    type PrivateState<'a> = ValueEmpty;
    type ProviderMetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                attributes: map! {
                    "id" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Id of the fiel resource"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "path" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Remote path to the file"),
                        constraint: AttributeConstraint::Required,
                        ..Default::default()
                    },
                    "content" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file"),
                        constraint: AttributeConstraint::Required,
                        ..Default::default()
                    },
                    "mode" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "overwrite" => Attribute {
                        attr_type: AttributeType::Bool,
                        description: Description::plain("Content of the remote file"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "keep" => Attribute {
                        attr_type: AttributeType::Bool,
                        description: Description::plain("Content of the remote file"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "md5" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("MD5 fingerprint of the file (hex)"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "sha1" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("SHA1 fingerprint of the file (hex)"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                },
                blocks: map! {
                    "connect" => NestedBlock::Optional(Block {
                        attributes: T::schema(),
                        description: Description::plain("Connection configuration"),
                        ..Default::default()
                    }),
                },
                description: Description::plain("Reads a remote file"),
                ..Default::default()
            },
        })
    }

    async fn validate<'a>(&self, diags: &mut Diagnostics, config: Self::State<'a>) -> Option<()> {
        if let Value::Value(connect) = &config.connect {
            _ = self
                .connect
                .validate(diags, AttributePath::new("connect").index(0), connect)
                .await;
        }

        match &config.path {
            Value::Value(path) => {
                if path.is_empty() {
                    diags.error_short("`path` should not be empty", AttributePath::new("path"));
                }
            }
            Value::Null => {
                diags.error_short("`path` should not be null", AttributePath::new("path"));
            }
            Value::Unknown => (),
        }

        if config.content.is_null() {
            diags.error_short("`content` cannot be null", AttributePath::new("content"));
        }

        if let Value::Value(mode) = &config.mode {
            match isize::from_str_radix(mode.as_ref(), 8) {
                Ok(0..=4095) => (),
                Ok(_) => diags.error("Invalid `mode`", format!("Mode should be an octal number between 0000 and 7777, but is {mode}"), AttributePath::new("mode")),
                Err(err) => diags.error("Invalid `mode`", format!("Mode should be an octal number between 0000 and 7777, but is {mode}\n{err}"), AttributePath::new("mode")),
            }
        }

        if diags.errors.is_empty() {
            Some(())
        } else {
            None
        }
    }

    async fn read<'a>(
        &self,
        diags: &mut Diagnostics,
        mut state: Self::State<'a>,
        private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let default_connect_config = Default::default();
        let connect_config = state.connect.as_ref().unwrap_or(&default_connect_config);

        let reader = match self.connect.read(connect_config, state.path.as_str()).await {
            Ok(writer) => writer,
            Err(err) => match err.downcast_ref::<std::io::Error>() {
                Some(err) if err.kind() == ErrorKind::NotFound => {
                    return None;
                }
                _ => {
                    diags.root_error("Could not open file for reading", err.to_string());
                    return None;
                }
            },
        };
        tokio::pin!(reader);

        let reader = HashingStream {
            digest: (Md5::new(), Sha1::new()),
            inner: reader,
        };

        let writer = tokio::io::sink();
        tokio::pin!(reader, writer);

        let copy = tokio::io::copy(&mut reader, &mut writer).await;
        match &copy {
            Ok(_) => {
                let (md5, sha1) = reader.fingerprints();

                if md5 != state.md5.as_str() {
                    state.md5 = Value::Null;
                }
                if sha1 != state.sha1.as_str() {
                    state.sha1 = Value::Null;
                }
            }
            Err(err) => {
                diags.root_error("Could not read file", err.to_string());
            }
        }

        Some((state, private_state))
    }

    async fn plan_create<'a>(
        &self,
        diags: &mut Diagnostics,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = proposed_state;
        state.normalize(diags);
        Some((state, Default::default()))
    }
    async fn plan_update<'a>(
        &self,
        diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(
        Self::State<'a>,
        Self::PrivateState<'a>,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        let mut state = proposed_state;
        state.normalize(diags);
        Some((state, prior_private_state, vec![]))
    }

    async fn plan_destroy<'a>(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        _prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        Some(())
    }

    async fn create<'a>(
        &self,
        diags: &mut Diagnostics,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        planned_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = planned_state;
        state.normalize(diags);

        self.write_file(diags, &mut state).await?;

        Some((state, planned_private_state))
    }
    async fn update<'a>(
        &self,
        diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        planned_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        planned_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = planned_state;
        state.normalize(diags);

        self.write_file(diags, &mut state).await?;

        Some((state, planned_private_state))
    }
    async fn destroy<'a>(
        &self,
        diags: &mut Diagnostics,
        state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<()> {
        let default_connect_config = Default::default();
        let connect_config = state.connect.as_ref().unwrap_or(&default_connect_config);

        match self
            .connect
            .delete(connect_config, state.path.as_str())
            .await
        {
            Ok(_) => Some(()),
            Err(err) => match err.downcast_ref::<std::io::Error>() {
                Some(err) if err.kind() == ErrorKind::NotFound => {
                    diags.root_warning("File has already been deleted", err.to_string());
                    Some(())
                }
                _ => {
                    diags.root_warning("Could not delete file", err.to_string());
                    None
                }
            },
        }
    }
}

impl<'a, T: Connection> WithNormalize for ResourceState<'a, T> {
    fn normalize(&mut self, _diags: &mut Diagnostics) {
        if self.id.is_null() {
            self.id = Value::Unknown;
        }
        if self.md5.is_null() {
            self.md5 = Value::Unknown;
        }
        if self.sha1.is_null() {
            self.sha1 = Value::Unknown;
        }
        if !self.mode.is_value() {
            self.mode = Value::Value("0666".into());
        }
        if !self.overwrite.is_value() {
            self.overwrite = Value::Value(false);
        }
        if !self.keep.is_value() {
            self.keep = Value::Value(false);
        }
    }
}

impl<T: Connection> CmdFileResource<T> {
    async fn write_file<'a>(
        &self,
        diags: &mut Diagnostics,
        state: &mut ResourceState<'a, T>,
    ) -> Option<()> {
        if !state.id.is_value() {
            state.id = ValueString::Value(
                thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(30)
                    .map(char::from)
                    .collect(),
            );
        }

        let default_connect_config = Default::default();
        let connect_config = state.connect.as_ref().unwrap_or(&default_connect_config);

        let writer = match self
            .connect
            .write(
                connect_config,
                state.path.as_str(),
                u32::from_str_radix(state.mode.as_str(), 8).unwrap_or(0o666),
                true,
            )
            .await
        {
            Ok(writer) => writer,
            Err(err) => {
                diags.root_error("Could not open file for writing", err.to_string());
                return None;
            }
        };
        tokio::pin!(writer);

        let writer = HashingStream {
            digest: (Md5::new(), Sha1::new()),
            inner: writer,
        };
        tokio::pin!(writer);

        let mut content = state.content.as_bytes();

        match tokio::io::copy(&mut content, &mut writer).await {
            Ok(_) => (),
            Err(err) => {
                diags.root_error("Could not write to file", err.to_string());
                return None;
            }
        };

        let (md5, sha1) = writer.fingerprints();

        state.md5 = Value::Value(md5.into());
        state.sha1 = Value::Value(sha1.into());

        Some(())
    }
}
