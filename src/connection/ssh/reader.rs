use std::{future::Future, pin::Pin, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use rusftp::{SftpClient, StatusCode};
use russh::client::Handle;
use tokio::io::AsyncRead;

use super::ClientHandler;

pub struct SftpReader {
    client: Arc<SftpClient>,
    handle: rusftp::Handle,
    offset: u64,
    eof: bool,
    request: Option<Pin<Box<dyn Future<Output = std::io::Result<Bytes>> + Send>>>,
}

impl SftpReader {
    pub(super) async fn new(handle: &Handle<ClientHandler>, filename: &str) -> Result<Self> {
        let client = SftpClient::new(handle.channel_open_session().await?).await?;

        let handle = client
            .open(rusftp::Open {
                filename: rusftp::Path(filename.to_owned().into()),
                pflags: rusftp::pflags::READ,
                attrs: Default::default(),
            })
            .await?;

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
                    .read(rusftp::Read {
                        handle,
                        offset,
                        length,
                    })
                    .await
                {
                    Ok(data) => Ok(data.0),
                    Err(status) => {
                        if status.code == StatusCode::Eof as u32 {
                            Ok(Bytes::default())
                        } else {
                            Err(std::io::Error::from(status))
                        }
                    }
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
