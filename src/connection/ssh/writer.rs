use std::{future::Future, io::ErrorKind, pin::Pin, sync::Arc};

use anyhow::{anyhow, Result};
use rusftp::{pflags, SftpClient, StatusCode};
use russh::client::Handle;
use tokio::io::AsyncWrite;

use super::ClientHandler;

pub struct SftpWriter {
    client: Arc<SftpClient>,
    handle: rusftp::Handle,
    offset: u64,
    eof: bool,
    request: Option<Pin<Box<dyn Future<Output = std::io::Result<usize>> + Send>>>,
}

impl SftpWriter {
    pub(super) async fn new(
        handle: &Handle<ClientHandler>,
        filename: &str,
        mode: u32,
        overwrite: bool,
    ) -> Result<Self> {
        let client = SftpClient::new(handle.channel_open_session().await?).await?;

        let mut flags = pflags::WRITE | pflags::CREATE;
        if overwrite {
            flags |= pflags::TRUNCATE;
        } else {
            eprintln!("Check if file already exists");
            // Check if file exist in case the EXCLUDE flag is not taken into account
            match client
                .send(
                    rusftp::LStat {
                        path: rusftp::Path(filename.to_owned().into()),
                    }
                    .into(),
                )
                .await
            {
                rusftp::Message::Attrs(_) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "File already exists",
                    )
                    .into());
                }
                rusftp::Message::Status(status) => {
                    if status.code != StatusCode::NoSuchFile as u32 {
                        return Err(std::io::Error::from(status).into());
                    }
                }
                _ => {
                    return Err(anyhow!("Bad Message"));
                }
            }
            flags |= pflags::EXCLUDE;
        }

        let handle = match client
            .send(
                rusftp::Open {
                    filename: rusftp::Path(filename.to_owned().into()),
                    pflags: flags,
                    attrs: rusftp::Attrs {
                        perms: Some(mode),
                        ..Default::default()
                    },
                }
                .into(),
            )
            .await
        {
            rusftp::Message::Status(status) => {
                return Err(std::io::Error::from(status).into());
            }
            rusftp::Message::Handle(h) => h,
            _ => {
                return Err(anyhow!("Bad reply"));
            }
        };

        Ok(SftpWriter {
            client: Arc::new(client),
            handle,
            offset: 0,
            eof: false,
            request: None,
        })
    }
}

impl AsyncWrite for SftpWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::result::Result<usize, std::io::Error>> {
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
            let length = buf.len().min(32768); // read at most 32K
            let data = rusftp::Data(buf[0..length].to_owned().into());
            self.request.get_or_insert(Box::pin(async move {
                match client
                    .send(
                        rusftp::Write {
                            handle,
                            offset,
                            data,
                        }
                        .into(),
                    )
                    .await
                {
                    rusftp::Message::Status(status) => {
                        if status.code == StatusCode::Ok as u32 {
                            Ok(length)
                        } else {
                            Err(std::io::Error::from(status))
                        }
                    }
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Bad reply",
                    )),
                }
            }))
        };

        match request.as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(len)) => {
                self.request = None;
                self.offset += len as u64;
                std::task::Poll::Ready(Ok(len))
            }
            std::task::Poll::Ready(Err(err)) => {
                if err.kind() == ErrorKind::UnexpectedEof {
                    self.eof = true;
                }
                std::task::Poll::Ready(Err(err))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        if self.eof {
            return std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "EOF",
            )));
        }

        let Some(request) = &mut self.request else {
            return std::task::Poll::Ready(Ok(()));
        };

        match request.as_mut().poll(cx) {
            std::task::Poll::Ready(Ok(len)) => {
                self.request = None;
                self.offset += len as u64;
                std::task::Poll::Ready(Ok(()))
            }
            std::task::Poll::Ready(Err(err)) => {
                if err.kind() == ErrorKind::UnexpectedEof {
                    self.eof = true;
                }
                std::task::Poll::Ready(Err(err))
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), std::io::Error>> {
        let close = self.client.stop();
        tokio::pin!(close);
        close.poll(cx).map(|_| Ok(()))
    }
}
