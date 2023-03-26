use std::{borrow::BorrowMut, pin::Pin, task::Poll};

use crypto::digest::Digest;
use tokio::io::{AsyncRead, AsyncWrite};

pub(super) struct HashingStream<D, I> {
    pub(super) digest: D,
    pub(super) inner: I,
}

impl<D: Digest + Unpin, I: AsyncRead + Unpin> AsyncRead for HashingStream<D, I> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let poll = Pin::new(self.inner.borrow_mut()).poll_read(cx, buf);

        if let Poll::Ready(Ok(_)) = poll {
            self.digest.input(buf.filled())
        }

        poll
    }
}

impl<D: Digest + Unpin, I: AsyncWrite + Unpin> AsyncWrite for HashingStream<D, I> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let poll = Pin::new(self.inner.borrow_mut()).poll_write(cx, buf);

        if let Poll::Ready(Ok(written)) = poll {
            self.digest.input(&buf[0..written])
        }

        poll
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(self.inner.borrow_mut()).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(self.inner.borrow_mut()).poll_shutdown(cx)
    }
}
