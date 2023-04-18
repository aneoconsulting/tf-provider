use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use crate::connection::{Connection, ExecutionResult};
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use bytes::Bytes;
use rusftp_client::{Message, Read, SftpClient, StatusCode};
use russh::client::{Config, Handle, Handler};
use serde::{Deserialize, Serialize};
use tf_provider::{
    map, Attribute, AttributeConstraint, AttributePath, AttributeType, Description, Diagnostics,
    Value, ValueString,
};
use tokio::{
    io::{AsyncRead, AsyncWriteExt},
    sync::{
        mpsc::{error::SendError, Sender},
        Mutex,
    },
};

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
    type Reader = SftpReader; // TODO: implements proper read/writer
    type Writer = tokio::fs::File; // TODO: implements proper read/writer

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
        _config: &Self::Config<'a>,
        _path: &str,
        _mode: u32,
        _overwrite: bool,
    ) -> Result<Self::Writer> {
        todo!()
    }

    /// Delete a file
    async fn delete<'a>(&self, config: &Self::Config<'a>, path: &str) -> Result<()> {
        let client = self.get_client(config).await?;
        let client = SftpClient::new(client.handle.channel_open_session().await?).await?;

        if let Message::Status(status) = client
            .send(rusftp_client::Message::Remove(path.to_owned().into()))
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

    async fn execute<'a, I, K, V>(&self, command: &str, env: I) -> Result<ExecutionResult>
    where
        I: IntoIterator<Item = (&'a K, &'a V)> + Send + Sync + 'a,
        I::IntoIter: Send + Sync + 'a,
        K: AsRef<str> + Send + Sync + 'a,
        V: AsRef<str> + Send + Sync + 'a,
    {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let mut channel = self.handle.channel_open_session().await?;

        channel.exec(false, "/bin/sh").await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let send = async {
            async fn send<'a>(
                tx: &Sender<&'a str>,
                msg: &'a str,
            ) -> Result<(), SendError<&'a str>> {
                tx.send(msg).await
            }
            send(&tx, "newline='\n'\nread_stdin() {\nvalue=\nwhile IFS= read -r line; do\nvalue=\"$value$line$newline\"\ndone\nvalue=\"$value$line\"\n}\n").await?;
            for (name, value) in env {
                let value = value.as_ref();
                send(&tx, "read_stdin << '__!@#$END_OF_VARIABLE$#@!__'\n").await?;
                if !value.is_empty() {
                    send(&tx, value).await?;
                }
                send(&tx, "\n__!@#$END_OF_VARIABLE$#@!__\nexport ").await?;
                send(&tx, name.as_ref()).await?;
                send(&tx, "=\"${value%%?}\"\n").await?;
            }
            send(
                &tx,
                "exec /usr/bin/env bash << '__!@#$END_OF_SCRIPT$#@!__'\n",
            )
            .await?;
            if !command.is_empty() {
                send(&tx, command).await?;
            }
            send(&tx, "\n__!@#$END_OF_SCRIPT$#@!__\n").await?;
            send(&tx, "").await?; // EOF
            Result::<(), SendError<&'a str>>::Ok(())
        };

        let receive = async {
            let mut status = None;
            loop {
                tokio::select! {
                    Some(data) = rx.recv() => {
                        if data.is_empty() {
                            channel.eof().await?;
                        } else {
                            channel.data(data.as_bytes()).await?;
                        }
                    },
                    msg = channel.wait() => {
                        let Some(msg) = msg else {
                            if let Some(status) = status {
                                return Ok(ExecutionResult {
                                    status,
                                    stdout: String::from_utf8(stdout)?,
                                    stderr: String::from_utf8(stderr)?,
                                })
                            } else {
                                return Err(anyhow!("channel closed without exit code"));
                            }
                        };
                        match msg {
                            russh::ChannelMsg::Data { ref data } => stdout.write_all(data).await?,
                            russh::ChannelMsg::ExtendedData { ref data, ext } => {
                                _ = ext;
                                stderr.write_all(data).await?;
                            }
                            russh::ChannelMsg::ExitStatus { exit_status } => {
                                status = Some(exit_status as i32);
                            }
                            russh::ChannelMsg::ExitSignal {
                                signal_name,
                                core_dumped: _,
                                error_message,
                                lang_tag: _,
                            } => {
                                return Err(anyhow!(
                                    "Exit signal received {signal_name:?}: {error_message}"
                                ));
                            }
                            _ => (),
                        }
                    }
                }
            }
        };

        let (send_result, receive_result) = tokio::join!(send, receive);
        channel.close().await?;
        send_result.map_err(|err| anyhow!("{err}"))?;
        receive_result
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
pub struct SftpReader {
    client: Arc<SftpClient>,
    handle: rusftp_client::Handle,
    offset: u64,
    eof: bool,
    request: Option<Pin<Box<dyn Future<Output = std::io::Result<Bytes>> + Send>>>,
}

impl SftpReader {
    async fn new(handle: &Handle<ClientHandler>, filename: &str) -> Result<Self> {
        let client = SftpClient::new(handle.channel_open_session().await?).await?;

        let handle = match client
            .send(rusftp_client::Message::Open(rusftp_client::Open {
                filename: filename.to_owned().into(),
                pflags: rusftp_client::PFlags::READ as u32,
                attrs: Default::default(),
            }))
            .await
        {
            rusftp_client::Message::Status(status) => {
                return Err(std::io::Error::from(status).into());
            }
            rusftp_client::Message::Handle(h) => h,
            _ => {
                return Err(anyhow!("Bad reply"));
            }
        };

        Ok(SftpReader {
            client: Arc::new(client),
            handle,
            offset: 0,
            eof: false,
            request: None,
        })
    }
}

impl AsyncRead for SftpReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        if self.eof {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF",
            )));
        }
        let request = if let Some(request) = &mut self.request {
            request
        } else {
            let client = self.client.clone();
            let handle = self.handle.clone();
            let offset = self.offset;
            let length = buf.remaining().min(32768) as u32; // read at most 32K
            self.request.get_or_insert(Box::pin(async move {
                match client
                    .send(rusftp_client::Message::Read(Read {
                        handle,
                        offset,
                        length,
                    }))
                    .await
                {
                    rusftp_client::Message::Status(status) => {
                        if status.code == StatusCode::Eof as u32 {
                            Ok(Bytes::default())
                        } else {
                            Err(std::io::Error::from(status))
                        }
                    }
                    rusftp_client::Message::Data(data) => Ok(data),
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Bad reply",
                    )),
                }
            }))
        };

        match request.as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(data)) => {
                if data.is_empty() {
                    self.eof = true;
                    self.request = None;
                    std::task::Poll::Ready(Ok(()))
                } else {
                    buf.put_slice(&data);
                    self.request = None;
                    self.offset += data.len() as u64;
                    std::task::Poll::Ready(Ok(()))
                }
            }
            std::task::Poll::Ready(Err(err)) => std::task::Poll::Ready(Err(err)),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
