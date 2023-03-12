use std::collections::HashMap;

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Result};
use async_process::{Command, Output};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectionLocal {}

impl From<Output> for ExecutionResult {
    fn from(value: Output) -> Self {
        Self {
            status: value.status.code().unwrap_or(0x7fffffff),
            stdout: value.stdout,
            stderr: value.stderr,
        }
    }
}

#[async_trait]
impl Connection for ConnectionLocal {
    const NAME: &'static str = "local";

    async fn execute(
        &self,
        cmd: Vec<Vec<u8>>,
        env: HashMap<Vec<u8>, Vec<u8>>,
    ) -> Result<ExecutionResult> {
        if cmd.len() > 0 {
            let (cmd, args) = (&cmd[0], &cmd[1..]);
            let mut command = Command::new(std::str::from_utf8(cmd.as_slice())?);
            for arg in args {
                command.arg(std::str::from_utf8(arg.as_slice())?);
            }
            for (k, v) in env {
                let k = std::str::from_utf8(k.as_slice())?;
                let v = std::str::from_utf8(v.as_slice())?;
                command.env(k, v);
            }
            Ok(command.output().await?.into())
        } else {
            Err(anyhow!("Command must not be empty"))
        }
    }

    /// Validate the state is valid
    async fn validate(&self, _diags: &mut Diagnostics, _attr_path: AttributePath) -> Option<()> {
        Some(())
    }

    fn schema() -> HashMap<String, Attribute> {
        Default::default()
    }
}
