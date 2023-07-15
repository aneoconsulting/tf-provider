use std::{future::Future, io::ErrorKind, pin::Pin, sync::Arc};

use anyhow::{anyhow, Result};
use rusftp_client::{FileAttrs, PFlags, Path, SftpClient, StatusCode};
use russh::client::Handle;
use tokio::io::AsyncWrite;

use super::ClientHandler;

pub struct SftpWriter {
    client: Arc<SftpClient>,
    handle: rusftp_client::Handle,
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

        let mut pflags = (PFlags::WRITE as u32) | (PFlags::CREATE as u32);
        if overwrite {
            pflags |= PFlags::TRUNCATE as u32;
        } else {
            eprintln!("Check if file already exists");
            // Check if file exist in case the EXCLUDE flag is not taken into account
            match client
                .send(rusftp_client::Message::LStat {
                    path: Path(filename.to_owned().into()),
                })
                .await
            {
                rusftp_client::Message::Attrs { .. } => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "File already exists",
                    )
                    .into());
                }
                rusftp_client::Message::Status(status) => {
                    if status.code != StatusCode::NoSuchFile as u32 {
                        return Err(std::io::Error::from(status).into());
                    }
                }
                _ => {
                    return Err(anyhow!("Bad Message"));
                }
            }
            pflags |= PFlags::EXCLUDE as u32;
        }

        let handle = match client
            .send(rusftp_client::Message::Open {
                filename: Path(filename.to_owned().into()),
                pflags,
                attrs: FileAttrs {
                    perms: Some(mode),
                    ..Default::default()
                },
            })
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

        Ok(SftpWriter {
            client: Arc::new(client),
            handle: rusftp_client::MessageHandle(handle),
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
            let data = rusftp_client::MessageData(buf[0..length].to_owned().into());
            self.request.get_or_insert(Box::pin(async move {
                match client
                    .send(rusftp_client::Message::Write {
                        handle,
                        offset,
                        data,
                    })
                    .await
                {
                    rusftp_client::Message::Status(status) => {
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
        let close = self.client.close();
        tokio::pin!(close);
        close.poll(cx).map(|_| Ok(()))
    }
}
