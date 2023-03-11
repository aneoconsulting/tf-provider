use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use tf_provider::Attribute;

#[derive(Debug, PartialEq, Eq)]
pub struct ExecutionResult {
    pub status: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[async_trait]
pub trait Connection: Send + Sync + 'static {
    /// execute a command over the connection
    async fn execute(
        &self,
        cmd: Vec<Vec<u8>>,
        env: HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<ExecutionResult>;

    /// Get the schema for the connection block
    fn schema(&self) -> HashMap<String, Attribute>;

    /// Get the name of the connection type
    fn name(&self) -> String;
}
