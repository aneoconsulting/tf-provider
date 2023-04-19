use std::collections::HashMap;

use futures::Future;
use russh::client::Msg;
use russh::Channel;
use russh::ChannelMsg;
use tokio::sync::{mpsc, oneshot};

use crate::Init;
use crate::Message;
use crate::StatusCode;
use crate::Version;

pub struct SftpClient {
    commands: mpsc::UnboundedSender<(Message, oneshot::Sender<Message>)>,
}

impl SftpClient {
    pub async fn new(mut channel: Channel<Msg>) -> Result<Self, std::io::Error> {
        // Start SFTP subsystem
        match channel.request_subsystem(false, "sftp").await {
            Ok(_) => (),
            Err(russh::Error::IO(err)) => {
                return Err(err);
            }
            Err(err) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, err));
            }
        }

        // Init SFTP handshake
        let init_message = Message::Init(Init {
            version: 3,
            extensions: Default::default(),
        });
        let init_frame = init_message.encode(0).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Could not initialize SFTP")
        })?;
        channel.data(init_frame.as_ref()).await.map_err(|e| {
            if let russh::Error::IO(io_err) = e {
                io_err
            } else {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            }
        })?;

        // Check handshake response
        match channel.wait().await {
            Some(ChannelMsg::Data { data }) => {
                let mut buf = data.as_ref();
                match Message::decode(&mut buf) {
                    // Valid response: continue
                    Ok((
                        _,
                        Message::Version(Version {
                            version: 3,
                            extensions: _,
                        }),
                    )) => (),

                    // Invalid responses: abort
                    Ok((_, Message::Version(_))) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Invalid sftp version",
                        ));
                    }
                    Ok(_) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Bad SFTP init",
                        ));
                    }
                    Err(_) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Could not encode SFTP init",
                        ));
                    }
                }
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to start SFTP subsystem",
                ));
            }
        }

        let (tx, mut rx) = mpsc::unbounded_channel::<(Message, oneshot::Sender<Message>)>();

        tokio::spawn(async move {
            let mut onflight = HashMap::<u32, oneshot::Sender<Message>>::new();
            let mut id = 0u32;

            loop {
                tokio::select! {
                    // New request to send
                    request = rx.recv() => {
                        let Some((message, tx)) = request else {
                            _ = channel.close().await;
                            break;
                        };

                        id += 1;
                        //eprintln!("Request #{id}: {message:?}");
                        match message.encode(id) {
                            Ok(frame) => {
                                if let Err(err) = channel.data(frame.as_ref()).await {
                                    _ = tx.send(err.into());
                                } else {
                                    onflight.insert(id, tx);
                                }
                            }
                            Err(_) => {
                                _ = tx.send(
                                    StatusCode::BadMessage.to_message("Could not encode message".into()),
                                );
                            }
                        }
                    },

                    // New response received
                    response = channel.wait() => {
                        let Some(ChannelMsg::Data { data }) = response else {
                            rx.close();
                            break;
                        };

                        let mut buf = data.as_ref();
                        match Message::decode(&mut buf) {
                            Ok((id, message)) => {
                                //eprintln!("Response #{id}: {message:?}");
                                if let Some(tx) = onflight.remove(&id) {
                                    _ = tx.send(message);
                                } else {
                                    eprintln!("SFTP Error: Received a reply with an invalid id");
                                }
                            }
                            Err(_) => {
                                eprintln!("SFTP Error: Could not decode server frame");
                            }
                        }
                    },
                }
            }
        });

        Ok(Self { commands: tx })
    }

    pub async fn send(&self, request: Message) -> Message {
        let (tx, rx) = oneshot::channel();

        if self.commands.send((request, tx)).is_err() {
            StatusCode::Failure.to_message("Could not send request to SFTP client".into())
        } else {
            rx.await.unwrap_or(
                StatusCode::Failure.to_message("Could get reply from SFTP client".into()),
            )
        }
    }

    pub fn close(&self) -> impl Future<Output = ()> + '_ {
        self.commands.closed()
    }
}
