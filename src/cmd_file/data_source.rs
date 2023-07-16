use std::fmt::Debug;

use async_trait::async_trait;

use base64::Engine;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributePath, AttributeType, Block, DataSource,
    Description, Diagnostics, NestedBlock, Schema, Value, ValueEmpty, ValueString,
};

use crate::{cmd_file::hash_stream::DefaultHashingStream, connection::Connection};

#[derive(Debug, Default)]
pub struct CmdFileDataSource<T: Connection> {
    pub(super) sensitive: bool,
    pub(super) connect: T,
}

impl<T: Connection> CmdFileDataSource<T> {
    pub fn new(sensitive: bool, connect: T) -> Self {
        Self { sensitive, connect }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataSourceState<'a, T>
where
    T: Connection,
{
    #[serde(borrow = "'a")]
    pub path: ValueString<'a>,
    #[serde(with = "value::serde_as_vec")]
    pub connect: Value<T::Config<'a>>,
    pub content: Value<String>,
    pub content_base64: Value<String>,
    pub md5: ValueString<'a>,
    pub sha1: ValueString<'a>,
    pub sha256: ValueString<'a>,
    pub sha512: ValueString<'a>,
    pub sha256_base64: ValueString<'a>,
    pub sha512_base64: ValueString<'a>,
}

#[async_trait]
impl<T> DataSource for CmdFileDataSource<T>
where
    T: Connection,
    T: Debug,
    T: Clone,
{
    type State<'a> = DataSourceState<'a, T>;
    type ProviderMetaState<'a> = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        Some(Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: map! {
                    "path" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Remote path to the file"),
                        constraint: AttributeConstraint::Required,
                        ..Default::default()
                    },
                    "content" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file"),
                        constraint: AttributeConstraint::Computed,
                        sensitive: self.sensitive,
                        ..Default::default()
                    },
                    "content_base64" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Content of the remote file base64 encoded"),
                        constraint: AttributeConstraint::Computed,
                        sensitive: self.sensitive,
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

        if diags.errors.is_empty() {
            Some(())
        } else {
            None
        }
    }

    async fn read<'a>(
        &self,
        diags: &mut Diagnostics,
        config: Self::State<'a>,
        _provider_meta_state: Self::ProviderMetaState<'a>,
    ) -> Option<Self::State<'a>> {
        let default_connect_config = Default::default();
        let connect_config = config.connect.as_ref().unwrap_or(&default_connect_config);
        let path = config.path.as_str();

        let reader = match self.connect.read(connect_config, path).await {
            Ok(reader) => reader,
            Err(err) => {
                diags.root_error("Could not read file", format!("{err}"));
                return None;
            }
        };
        tokio::pin!(reader);
        let reader = DefaultHashingStream::new(reader);
        tokio::pin!(reader);

        let mut content = Vec::new();

        if let Err(err) = reader.read_to_end(&mut content).await {
            diags.root_error("Could not read file", err.to_string());
            return None;
        }
        let mut output = config;

        output.content_base64 =
            Value::Value(base64::engine::general_purpose::STANDARD.encode(content.as_slice()));
        output.content = Value::Value(String::from_utf8_lossy(content.as_slice()).to_string());

        let (md5, sha1, sha256, sha512) = reader.fingerprints_hex();
        let (_, _, sha256_base64, sha512_base64) = reader.fingerprints_base64();

        output.md5 = Value::Value(md5.into());
        output.sha1 = Value::Value(sha1.into());
        output.sha256 = Value::Value(sha256.into());
        output.sha512 = Value::Value(sha512.into());
        output.sha256_base64 = Value::Value(sha256_base64.into());
        output.sha512_base64 = Value::Value(sha512_base64.into());

        Some(output)
    }
}
