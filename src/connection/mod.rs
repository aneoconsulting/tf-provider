use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics};

pub mod local;
pub mod ssh;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait Connection: Send + Sync + 'static + Default {
    const NAME: &'static str;
    type Config<'a>: Send + Sync + Clone + Default + Serialize + for<'de> Deserialize<'de>;

    /// execute a command over the connection
    async fn execute<'a, 'b, I, K, V>(
        &self,
        config: &Self::Config<'a>,
        cmd: &str,
        env: I,
    ) -> Result<ExecutionResult>
    where
        'a: 'b,
        I: IntoIterator<Item = (&'b K, &'b V)> + Send + Sync + 'b,
        I::IntoIter: Send + Sync + 'b,
        K: AsRef<str> + Send + Sync + 'b,
        V: AsRef<str> + Send + Sync + 'b;

    /// Validate the state is valid
    async fn validate<'a>(
        &self,
        diags: &mut Diagnostics,
        attr_path: AttributePath,
        config: &Self::Config<'a>,
    ) -> Option<()>;

    /// Get the schema for the connection block
    fn schema() -> HashMap<String, Attribute>;
}
