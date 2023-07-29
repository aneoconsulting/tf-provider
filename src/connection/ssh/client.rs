use std::sync::Arc;

use crate::connection::ExecutionResult;
use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use russh::client::{Config, Handle, Handler};
use tf_provider::Value;
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{error::SendError, Sender},
};

use super::ConnectionSshConfig;

pub(super) struct Client {
    pub(super) handle: Handle<ClientHandler>,
}

impl Client {
    pub(super) async fn connect<'a>(config: &ConnectionSshConfig<'a>) -> Result<Self> {
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

    pub(super) async fn execute<'a, I, K, V>(
        &self,
        command: &str,
        dir: &str,
        env: I,
    ) -> Result<ExecutionResult>
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
            // Helper to read input
            send(&tx, "newline='\n'\nread_stdin() {\nvalue=\nwhile IFS= read -r line; do\nvalue=\"$value$line$newline\"\ndone\nvalue=\"$value$line\"\n}\n").await?;

            // Change dir
            send(&tx, "read_stdin << '__!@#$END_OF_WORKDIR$#@!__'\n").await?;
            if !dir.is_empty() {
                send(&tx, dir).await?;
            }
            send(&tx, "\n__!@#$END_OF_WORKDIR$#@!__\nvalue=\"${value%%?}\"\n[ -z \"$value\" ] || cd \"$value\"\n").await?;

            // Export env
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

            // Execute command
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

    pub(super) async fn disconnect(&self) -> Result<()> {
        self.handle
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await
            .map_err(Into::into)
    }
}

#[derive(Clone)]
pub(super) struct ClientHandler {}

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
