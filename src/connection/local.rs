use std::collections::HashMap;

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Error, Result};
use async_process::{Command, Output};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tf_provider::{attribute_path::AttributePath, Attribute, Diagnostics};
use tokio::fs::{File, OpenOptions};

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
    type Reader = File;
    type Writer = File;

    async fn execute<'a, 'b, I, K, V>(
        &self,
        _config: &Self::Config<'a>,
        cmd: &str,
        dir: &str,
        env: I,
    ) -> Result<ExecutionResult>
    where
        'a: 'b,
        I: IntoIterator<Item = (&'b K, &'b V)> + Send + Sync + 'b,
        I::IntoIter: Send + Sync + 'b,
        K: AsRef<str> + Send + Sync + 'b,
        V: AsRef<str> + Send + Sync + 'b,
    {
        if !cmd.is_empty() {
            let mut command = Command::new("sh");
            eprintln!("Workdir: {dir}");
            if !dir.is_empty() {
                command.current_dir(dir);
            }
            command.arg("-c").arg(cmd);
            for (k, v) in env {
                command.env(k.as_ref(), v.as_ref());
            }
            Ok(command.output().await?.try_into()?)
        } else {
            Err(anyhow!("Command must not be empty"))
        }
    }

    /// Return a reader to read a remote file
    async fn read<'a>(&self, _config: &Self::Config<'a>, path: &str) -> Result<Self::Reader> {
        File::open(path).await.map_err(Into::into)
    }

    /// Return a writer to write a remote file
    #[cfg(target_family = "unix")]
    async fn write<'a>(
        &self,
        _config: &Self::Config<'a>,
        path: &str,
        mode: u32,
        overwrite: bool,
    ) -> Result<Self::Writer> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .create_new(!overwrite)
            .mode(mode)
            .open(path)
            .await
            .map_err(Into::into)
    }
    #[cfg(target_family = "windows")]
    async fn write<'a>(
        &self,
        _config: &Self::Config<'a>,
        path: &str,
        mode: u32,
        overwrite: bool,
    ) -> Result<Self::Writer> {
        // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea
        let attr = if (mode & 0o222) != 0 {
            128 // FILE_ATTRIBUTE_NORMAL
        } else {
            1 // FILE_ATTRIBUTE_READONLY
        };
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .create_new(!overwrite)
            .attributes(attr)
            .open(path)
            .await
            .map_err(Into::into)
    }

    /// Delete a file
    async fn delete<'a>(&self, _config: &Self::Config<'a>, path: &str) -> Result<()> {
        tokio::fs::remove_file(path).await.map_err(Into::into)
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
