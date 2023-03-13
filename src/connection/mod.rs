use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics, ValueString};

pub mod local;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

#[async_trait]
pub trait Connection: Send + Sync + 'static + Default {
    const NAME: &'static str;

    /// execute a command over the connection
    async fn execute(
        &self,
        cmd: &str,
        env: &HashMap<String, ValueString>,
    ) -> Result<ExecutionResult>;

    /// Validate the state is valid
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) -> Option<()>;

    /// Get the schema for the connection block
    fn schema() -> HashMap<String, Attribute>;
}
