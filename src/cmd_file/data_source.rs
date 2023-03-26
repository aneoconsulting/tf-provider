use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

use tf_provider::{
    map, value, Attribute, AttributeConstraint, AttributePath, AttributeType, Block, DataSource,
    Description, Diagnostics, NestedBlock, Schema, Value, ValueEmpty, ValueString,
};

use crate::connection::Connection;

#[derive(Debug, Default)]
pub struct CmdFileDataSource<T: Connection> {
    pub(super) connect: T,
}

impl<T: Connection> CmdFileDataSource<T> {
    pub fn new(connect: T) -> Self {
        Self { connect }
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

        if diags.errors.len() == 0 {
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
                diags.root_error("Could not read file", err.to_string());
                return None;
            }
        };
        tokio::pin!(reader);

        let mut content = String::new();

        if let Err(err) = reader.read_to_string(&mut content).await {
            diags.root_error("Could not read file", err.to_string());
            return None;
        }

        let mut output = config;
        output.content = Value::Value(content);

        Some(output)
    }
}
