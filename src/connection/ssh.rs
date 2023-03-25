use std::{collections::HashMap, io::Write, pin::Pin, sync::Arc};

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use russh::client::{Config, Handle, Handler};
use serde::{Deserialize, Serialize};
use tf_provider::{
    map, Attribute, AttributeConstraint, AttributePath, AttributeType, Description, Diagnostics,
    Value, ValueString,
};
use tokio::sync::{mpsc::error::SendError, Mutex};

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

    async fn execute<'a, I, K, V>(
        &self,
        config: &Self::Config<'a>,
        cmd: &str,
        env: I,
    ) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = (K, V)> + Send + Sync,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        _ = env;

        let client = self.get_client(config).await?;
        let result = client.execute(cmd, env).await?;
        Ok(result)
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

impl std::fmt::Debug for ConnectionSsh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnectionSsh") /*.field("clients", &self.clients)*/
            .finish()
    }
}

struct Client {
    handle: Handle<ClientHandler>,
}

impl Client {
    async fn connect<'a>(config: &ConnectionSshConfig<'a>) -> Result<Self> {
        let russh_config = Arc::new(Config::default());
        let hostname = config.host.as_str();
        let port = config.port.unwrap_or_default();
        let port = if port == 0 { 22 } else { port };
        let client_handler = ClientHandler {};

        let mut handle =
            russh::client::connect(russh_config, (hostname, port), client_handler).await?;

        let password = config.password.as_str();
        let password = if password.is_empty() {
            None
        } else {
            Some(password)
        };

        let private_key = match (&config.key, &config.keyfile) {
            (Value::Value(_), Value::Value(_)) => {
                return Err(anyhow!("Both private key and private key file were given"));
            }
            (Value::Value(key), _) => Some(russh_keys::decode_secret_key(key.as_ref(), password)?),
            (_, Value::Value(keyfile)) => {
                Some(russh_keys::load_secret_key(keyfile.as_ref(), password)?)
            }
            _ => None,
        };

        let username = config.user.as_str();
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

    async fn execute<I, K, V>(&self, command: &str, env: I) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = (K, V)> + Send + Sync,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut channel = self.handle.channel_open_session().await?;
        channel.exec(true, "bash -").await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);
        let command = command.to_string();

        let env: Vec<(_, _)> = env
            .into_iter()
            .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
            .collect();

        let send = tokio::spawn(async move {
            tx.send("set -ex\nnewline='\n'\nread_stdin() {\nvalue=\nwhile IFS= read -r line; do\nvalue=\"$value$line$newline\"\ndone < /dev/stdin\nvalue=\"$value$line\"\n}\n".to_string()).await?;
            for (name, value) in env {
                tx.send("read_stdin << '__!@#$END_OF_VARIABLE$#@!__'\n".to_string())
                    .await?;
                if !value.is_empty() {
                    tx.send(value).await?;
                }
                //tx.send(String::default()).await?;
                tx.send(format!(
                    "\n__!@#$END_OF_VARIABLE$#@!__\nexport {name}=\"${{value%%?}}\"\n"
                ))
                .await?;
            }
            tx.send(command).await?;
            tx.send(String::default()).await?;
            Result::<(), SendError<String>>::Ok(())
        });

        loop {
            tokio::select! {
                Some(data) = rx.recv() => {
                    eprintln!("{}", data);
                    if data.is_empty() {
                        channel.eof().await?;
                    } else {
                        channel.data(data.as_bytes()).await?;
                    }
                },
                Some(msg) = channel.wait() => {
                    match msg {
                        russh::ChannelMsg::Data { ref data } => stdout.write_all(data)?,
                        russh::ChannelMsg::ExtendedData { ref data, ext } => {
                            _ = ext;
                            stderr.write_all(data)?;
                        }
                        russh::ChannelMsg::ExitStatus { exit_status } => {
                            send.await??;
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
            }
        }
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
