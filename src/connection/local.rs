use std::collections::HashMap;

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Error, Result};
use async_process::{Command, Output};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics};

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct ConnectionLocal {}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone)]
pub struct ConnectionLocalConfig {}

impl TryFrom<Output> for ExecutionResult {
    type Error = Error;
    fn try_from(value: Output) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            status: value.status.code().ok_or(anyhow!("invalid status code"))?,
            stdout: String::from_utf8(value.stdout)?,
            stderr: String::from_utf8(value.stderr)?,
        })
    }
}

#[async_trait]
impl Connection for ConnectionLocal {
    const NAME: &'static str = "local";
    type Config<'a> = ConnectionLocalConfig;

    async fn execute<'a, I, K, V>(
        &self,
        _config: &Self::Config<'a>,
        cmd: &str,
        env: I,
    ) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = (K, V)> + Send + Sync,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        if cmd.len() > 0 {
            let mut command = Command::new("sh");
            command.arg("-c").arg(cmd);
            for (k, v) in env.into_iter() {
                command.env(k.as_ref(), v.as_ref());
            }
            Ok(command.output().await?.try_into()?)
        } else {
            Err(anyhow!("Command must not be empty"))
        }
    }

    /// Validate the state is valid
    async fn validate<'a>(
        &self,
        _diags: &mut Diagnostics,
        _attr_path: AttributePath,
        _config: &Self::Config<'a>,
    ) -> Option<()> {
        Some(())
    }

    fn schema() -> HashMap<String, Attribute> {
        Default::default()
    }
}
