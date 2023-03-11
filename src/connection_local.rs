use std::collections::HashMap;

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Result};
use async_process::{Command, Output};
use async_trait::async_trait;
use tf_provider::Attribute;

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

    fn schema(&self) -> HashMap<String, Attribute> {
        Default::default()
    }

    fn name(&self) -> String {
        "local".into()
    }
}
