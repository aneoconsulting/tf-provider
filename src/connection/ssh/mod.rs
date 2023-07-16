use std::{collections::HashMap, pin::Pin, sync::Arc};

use crate::connection::{Connection, ExecutionResult};
use anyhow::Result;
use async_trait::async_trait;
use rusftp::{SftpClient, StatusCode};
use serde::{Deserialize, Serialize};
use tf_provider::{
    map, Attribute, AttributeConstraint, AttributePath, AttributeType, Description, Diagnostics,
    Value, ValueString,
};
use tokio::sync::Mutex;

mod client;
mod reader;
mod writer;

use client::{Client, ClientHandler};
pub use reader::SftpReader;
pub use writer::SftpWriter;

#[derive(Default, Clone)]
pub struct ConnectionSsh {
    clients: Arc<Mutex<HashMap<ConnectionSshConfig<'static>, Arc<Client>>>>,
}

impl ConnectionSsh {
    async fn get_client<'a, 'b>(&'a self, config: &ConnectionSshConfig<'b>) -> Result<Arc<Client>> {
        let mut clients = self.clients.lock().await;
        let client = match clients.entry(config.clone().extend()) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                let client = Client::connect(entry.key()).await?;
                entry.insert(Arc::new(client))
            }
        };

        Ok(client.clone())
    }
}

impl Drop for ConnectionSsh {
    fn drop(&mut self) {
        let clients = Pin::new(futures::executor::block_on(self.clients.lock()));

        for (_, client) in clients.iter() {
            _ = futures::executor::block_on(client.disconnect());
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Default, Clone)]
pub struct ConnectionSshConfig<'a> {
    pub host: ValueString<'a>,
    pub port: Value<u16>,
    pub user: ValueString<'a>,
    pub password: ValueString<'a>,
    pub key: ValueString<'a>,
    pub keyfile: ValueString<'a>,
}

impl<'a> ConnectionSshConfig<'a> {
    fn extend<'b>(self) -> ConnectionSshConfig<'b> {
        ConnectionSshConfig {
            host: self.host.extend(),
            port: self.port,
            user: self.user.extend(),
            password: self.password.extend(),
            key: self.key.extend(),
            keyfile: self.keyfile.extend(),
        }
    }
}

#[async_trait]
impl Connection for ConnectionSsh {
    const NAME: &'static str = "ssh";
    type Config<'a> = ConnectionSshConfig<'a>;
    type Reader = SftpReader;
    type Writer = SftpWriter;

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
        V: AsRef<str> + Send + Sync + 'b,
    {
        _ = env;

        let client = self.get_client(config).await?;
        let result = client.execute(cmd, env).await?;
        Ok(result)
    }

    /// Return a reader to read a remote file
    async fn read<'a>(&self, config: &Self::Config<'a>, path: &str) -> Result<Self::Reader> {
        let client = self.get_client(config).await?;
        Ok(SftpReader::new(&client.handle, path).await?)
    }

    /// Return a writer to write a remote file
    async fn write<'a>(
        &self,
        config: &Self::Config<'a>,
        path: &str,
        mode: u32,
        overwrite: bool,
    ) -> Result<Self::Writer> {
        let client = self.get_client(config).await?;
        Ok(SftpWriter::new(&client.handle, path, mode, overwrite).await?)
    }

    /// Delete a file
    async fn delete<'a>(&self, config: &Self::Config<'a>, path: &str) -> Result<()> {
        let client = self.get_client(config).await?;
        let client = SftpClient::new(client.handle.channel_open_session().await?).await?;

        if let rusftp::Message::Status(status) = client
            .send(
                rusftp::Remove {
                    path: rusftp::Path(path.to_owned().into()),
                }
                .into(),
            )
            .await
        {
            if status.code == StatusCode::Ok as u32 {
                Ok(())
            } else {
                Err(status.into())
            }
        } else {
            Err(StatusCode::BadMessage
                .to_status("Bad Response".into())
                .into())
        }
    }

    /// Validate the state is valid
    async fn validate<'a>(
        &self,
        diags: &mut Diagnostics,
        attr_path: AttributePath,
        config: &Self::Config<'a>,
    ) -> Option<()> {
        match &config.host {
            Value::Value(host) => {
                if host.is_empty() {
                    diags.error_short("`hostname` cannot be empty", attr_path.attribute("host"));
                    return None;
                }
            }
            Value::Null => {
                diags.error_short("`hostname` cannot be null", attr_path.attribute("host"));
                return None;
            }
            Value::Unknown => (),
        }
        Some(())
    }

    fn schema() -> HashMap<String, Attribute> {
        map! {
            "host" => Attribute {
                attr_type: AttributeType::String,
                description: Description::plain("Hostname to connect to"),
                constraint: AttributeConstraint::Required,
                ..Default::default()
            },
            "port" => Attribute {
                attr_type: AttributeType::Number,
                description: Description::plain("Port to connect to"),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
            "user" => Attribute {
                attr_type: AttributeType::String,
                description: Description::plain("User to connect with"),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
            "password" => Attribute {
                attr_type: AttributeType::String,
                description: Description::plain("Password or passphrase"),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
            "key" => Attribute {
                attr_type: AttributeType::String,
                description: Description::plain("Key"),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
            "keyfile" => Attribute {
                attr_type: AttributeType::String,
                description: Description::plain("Filename of the key"),
                constraint: AttributeConstraint::Optional,
                ..Default::default()
            },
        }
    }
}

impl std::fmt::Debug for ConnectionSsh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionSsh") /*.field("clients", &self.clients)*/
            .finish()
    }
}
