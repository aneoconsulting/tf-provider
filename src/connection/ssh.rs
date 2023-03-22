use std::{collections::HashMap, io::Write, sync::Arc};

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use russh::client::{Config, Handle, Handler};
use serde::{Deserialize, Serialize};
use tf_provider::{
    map, Attribute, AttributeConstraint, AttributePath, AttributeType, Description, Diagnostics,
    Value,
};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Default, Clone)]
pub struct ConnectionSSH {
    pub host: Value<String>,
    pub port: Value<u16>,
    pub user: Value<String>,
    pub password: Value<String>,
    pub key: Value<String>,
    pub keyfile: Value<String>,
}

#[async_trait]
impl Connection for ConnectionSSH {
    const NAME: &'static str = "ssh";

    async fn execute<'a, I, K, V>(&'a self, cmd: &'a str, env: I) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = (K, V)> + Send + Sync,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        _ = env;
        let client = Client::connect(self).await?;
        let result = client.execute(cmd).await?;
        client.disconnect().await?;
        Ok(result)
    }

    /// Validate the state is valid
    async fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) -> Option<()> {
        match &self.host {
            Value::Value(host) => {
                if host.is_empty() {
                    diags.error_short(
                        "`hostname` cannot be empty",
                        attr_path.clone().attribute("host"),
                    );
                    return None;
                }
            }
            Value::Null => {
                diags.error_short(
                    "`hostname` cannot be null",
                    attr_path.clone().attribute("host"),
                );
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

struct Client {
    handle: Handle<ClientHandler>,
}

impl Client {
    async fn connect(config: &ConnectionSSH) -> Result<Self> {
        let russh_config = Arc::new(Config::default());
        let hostname = config.host.as_ref().map_or("", String::as_str);
        let port = config.port.unwrap_or_default();
        let port = if port == 0 { 22 } else { port };
        let client_handler = ClientHandler {};

        let mut handle =
            russh::client::connect(russh_config, (hostname, port), client_handler).await?;

        let password = config.password.as_ref_option().map(String::as_str);

        let private_key = match (&config.key, &config.keyfile) {
            (Value::Value(_), Value::Value(_)) => {
                return Err(anyhow!("Both private key and private key file were given"));
            }
            (Value::Value(key), _) => Some(russh_keys::decode_secret_key(key.as_str(), password)?),
            (_, Value::Value(keyfile)) => {
                Some(russh_keys::load_secret_key(keyfile.as_str(), password)?)
            }
            _ => None,
        };

        let username = config.user.as_ref().map_or("", String::as_str);
        let username = if username.is_empty() {
            "root"
        } else {
            username
        };

        let authenticated = match (private_key, password) {
            (Some(private_key), _) => {
                handle
                    .authenticate_publickey(username, Arc::new(private_key))
                    .await?
            }
            (None, Some(password)) => handle.authenticate_password(username, password).await?,
            (None, None) => handle.authenticate_none(username).await?,
        };

        if !authenticated {
            return Err(anyhow!("Authentication failure"));
        }

        Ok(Client { handle })
    }

    async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut channel = self.handle.channel_open_session().await?;
        channel.exec(true, command).await?;

        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { ref data } => stdout.write_all(data)?,
                russh::ChannelMsg::ExtendedData { ref data, ext } => {
                    _ = ext;
                    stderr.write_all(data)?;
                }
                russh::ChannelMsg::ExitStatus { exit_status } => {
                    return Ok(ExecutionResult {
                        status: exit_status as i32,
                        stdout: String::from_utf8(stdout)?,
                        stderr: String::from_utf8(stderr)?,
                    });
                }
                russh::ChannelMsg::ExitSignal {
                    signal_name,
                    core_dumped,
                    error_message,
                    lang_tag,
                } => {
                    _ = core_dumped;
                    _ = lang_tag;
                    return Err(anyhow!(
                        "Exit signal received {signal_name:?}: {error_message}"
                    ));
                }
                _ => (),
            }
        }

        Err(anyhow!("Command did not exit"))
    }

    async fn disconnect(&self) -> Result<()> {
        self.handle
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
struct ClientHandler {}

#[async_trait]
impl Handler for ClientHandler {
    type Error = Error;

    async fn check_server_key(
        self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<(Self, bool), Self::Error> {
        Ok((self, true))
    }
}
