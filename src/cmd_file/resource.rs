use std::fmt::Debug;
use std::io::ErrorKind;

use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use serde::{Deserialize, Serialize};
use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributePath, AttributeType, Block, Description,
    Diagnostics, NestedBlock, Resource, Schema, Value, ValueEmpty, ValueString,
};
use tokio::fs::File;
use tokio::io::AsyncRead;

use super::hash_stream::DefaultHashingStream;
use crate::connection::Connection;

#[derive(Debug, Default)]
pub struct CmdFileResource<T: Connection> {
    pub(super) sensitive: bool,
    pub(super) connect: T,
}

impl<T: Connection> CmdFileResource<T> {
    pub fn new(sensitive: bool, connect: T) -> Self {
        Self { sensitive, connect }
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
    pub content_base64: ValueString<'a>,
    pub content_source: ValueString<'a>,
    pub mode: ValueString<'a>,
    pub overwrite: Value<bool>,
    pub keep: Value<bool>,
    pub md5: ValueString<'a>,
    pub sha1: ValueString<'a>,
    pub sha256: ValueString<'a>,
    pub sha512: ValueString<'a>,
    pub sha256_base64: ValueString<'a>,
    pub sha512_base64: ValueString<'a>,
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
                        constraint: AttributeConstraint::OptionalComputed,
                        sensitive: self.sensitive,
                        ..Default::default()
                    },
                    "content_base64" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file encoded in base64"),
                        constraint: AttributeConstraint::OptionalComputed,
                        sensitive: self.sensitive,
                        ..Default::default()
                    },
                    "content_source" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file from a local file"),
                        constraint: AttributeConstraint::OptionalComputed,
                        sensitive: self.sensitive,
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
                    "sha256" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("SHA256 fingerprint of the file (hex)"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "sha512" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("SHA512 fingerprint of the file (hex)"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "sha256_base64" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("SHA256 fingerprint of the file (base64)"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "sha512_base64" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("SHA512 fingerprint of the file (base64)"),
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

        let nb_values = config.content.is_value() as i32
            + config.content_base64.is_value() as i32
            + config.content_source.is_value() as i32;
        let nb_unknowns = config.content.is_unknown() as i32
            + config.content_base64.is_unknown() as i32
            + config.content_source.is_unknown() as i32;

        if !matches!((nb_values, nb_unknowns), (1, _) | (0, 1..)) {
            diags.root_error("Invalid content specification", "Exactly one of `content`, `content_base64`, and `content_source` must be given. The others must be null.");
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

        let reader = DefaultHashingStream::new(reader);

        let writer = tokio::io::sink();
        tokio::pin!(reader, writer);

        let copy = tokio::io::copy(&mut reader, &mut writer).await;
        match &copy {
            Ok(_) => {
                let (md5, sha1, sha256, sha512) = reader.fingerprints_hex();
                let (_, _, sha256_base64, sha512_base64) = reader.fingerprints_base64();

                if md5 != state.md5.as_str()
                    || sha1 != state.sha1.as_str()
                    || sha256 != state.sha256.as_str()
                    || sha512 != state.sha512.as_str()
                    || sha256_base64 != state.sha256_base64.as_str()
                    || sha512_base64 != state.sha512_base64.as_str()
                {
                    state.md5 = Value::Null;
                    state.sha1 = Value::Null;
                    state.sha256 = Value::Null;
                    state.sha512 = Value::Null;
                    state.sha256_base64 = Value::Null;
                    state.sha512_base64 = Value::Null;
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
        _diags: &mut Diagnostics,
        proposed_state: Self::State<'a>,
        _config_state: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = proposed_state;
        self.normalize(&mut state);
        Some((state, Default::default()))
    }
    async fn plan_update<'a>(
        &self,
        _diags: &mut Diagnostics,
        prior_state: Self::State<'a>,
        proposed_state: Self::State<'a>,
        config_state: Self::State<'a>,
        prior_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(
        Self::State<'a>,
        Self::PrivateState<'a>,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        let mut state = proposed_state;
        if config_state.content.is_null() {
            state.content = Value::Null;
        }
        if config_state.content_base64.is_null() {
            state.content_base64 = Value::Null;
        }
        if config_state.content_source.is_null() {
            state.content_source = Value::Null;
        }
        if state.content != prior_state.content
            || state.content_base64 != prior_state.content_base64
            || state.content_source != prior_state.content_source
        {
            state.md5 = Value::Unknown;
            state.sha1 = Value::Unknown;
            state.sha256 = Value::Unknown;
            state.sha512 = Value::Unknown;
            state.sha256_base64 = Value::Unknown;
            state.sha512_base64 = Value::Unknown;
        }
        self.normalize(&mut state);
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
        self.normalize(&mut state);

        let overwrite = state.overwrite.unwrap_or(false);
        self.write_file(diags, &mut state, overwrite).await?;

        Some((state, planned_private_state))
    }
    async fn update<'a>(
        &self,
        diags: &mut Diagnostics,
        _prior_state: Self::State<'a>,
        planned_state: Self::State<'a>,
        config_state: Self::State<'a>,
        planned_private_state: Self::PrivateState<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<(Self::State<'a>, Self::PrivateState<'a>)> {
        let mut state = planned_state;
        if config_state.content.is_null() {
            state.content = Value::Null;
        }
        if config_state.content_base64.is_null() {
            state.content_base64 = Value::Null;
        }
        if config_state.content_source.is_null() {
            state.content_source = Value::Null;
        }
        self.normalize(&mut state);

        self.write_file(diags, &mut state, true).await?;

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

impl<T: Connection> CmdFileResource<T> {
    fn normalize(&self, state: &mut ResourceState<'_, T>) {
        if state.id.is_null() {
            state.id = Value::Unknown;
        }
        if state.md5.is_null() {
            state.md5 = Value::Unknown;
        }
        if state.sha1.is_null() {
            state.sha1 = Value::Unknown;
        }
        if state.sha256.is_null() {
            state.sha256 = Value::Unknown;
        }
        if state.sha512.is_null() {
            state.sha512 = Value::Unknown;
        }
        if state.sha256_base64.is_null() {
            state.sha256_base64 = Value::Unknown;
        }
        if state.sha512_base64.is_null() {
            state.sha512_base64 = Value::Unknown;
        }
        if !state.mode.is_value() {
            let mode = if self.sensitive { "0600" } else { "0666" };
            state.mode = Value::Value(mode.into());
        }
        if !state.overwrite.is_value() {
            state.overwrite = Value::Value(false);
        }
        if !state.keep.is_value() {
            state.keep = Value::Value(false);
        }
    }
    async fn write_file<'a>(
        &self,
        diags: &mut Diagnostics,
        state: &mut ResourceState<'a, T>,
        overwrite: bool,
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
                overwrite,
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

        let mut writer = DefaultHashingStream::new(writer);

        enum Content<'b> {
            Raw(&'b [u8]),
            Base64(Vec<u8>),
            File(File),
        }

        let content = if let Value::Value(content) = &state.content {
            Content::Raw(content.as_bytes())
        } else if let Value::Value(base64) = &state.content_base64 {
            match base64::decode(base64.as_bytes()) {
                Ok(decoded) => Content::Base64(decoded),
                Err(err) => {
                    diags.error(
                        "Invalid base64",
                        err.to_string(),
                        AttributePath::new("content_base64"),
                    );
                    return None;
                }
            }
        } else if let Value::Value(filename) = &state.content_source {
            match File::open(filename.as_ref()).await {
                Ok(file) => Content::File(file),
                Err(err) => {
                    diags.error(
                        "Could not open file",
                        err.to_string(),
                        AttributePath::new("content_source"),
                    );
                    return None;
                }
            }
        } else {
            diags.root_error_short("No content provided");
            return None;
        };

        enum ContentReader<'b> {
            Raw(&'b [u8]),
            File(File),
        }

        let mut content = match content {
            Content::Raw(raw) => ContentReader::Raw(raw),
            Content::Base64(ref decoded) => ContentReader::Raw(decoded.as_slice()),
            Content::File(file) => ContentReader::File(file),
        };

        let reader = match &mut content {
            ContentReader::Raw(raw) => raw as &mut (dyn AsyncRead + Send + Unpin),
            ContentReader::File(file) => file as &mut (dyn AsyncRead + Send + Unpin),
        };

        match tokio::io::copy(reader, &mut writer).await {
            Ok(_) => (),
            Err(err) => {
                diags.root_error("Could not write to file", err.to_string());
                return None;
            }
        };

        let (md5, sha1, sha256, sha512) = writer.fingerprints_hex();
        let (_, _, sha256_base64, sha512_base64) = writer.fingerprints_base64();

        state.md5 = Value::Value(md5.into());
        state.sha1 = Value::Value(sha1.into());
        state.sha256 = Value::Value(sha256.into());
        state.sha512 = Value::Value(sha512.into());
        state.sha256_base64 = Value::Value(sha256_base64.into());
        state.sha512_base64 = Value::Value(sha512_base64.into());

        Some(())
    }
}
