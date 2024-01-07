use std::{pin::Pin, task::Poll};

use tokio::io::AsyncWrite;

use crate::Gauge;

pub struct WriteHalf<W> {
    w: W,
    gauge: Gauge,
}
impl<W> WriteHalf<W> {
    pub fn new(w: W, gauge: Gauge) -> Self {
        Self { w, gauge }
    }
}
impl<W: AsyncWrite + Unpin> AsyncWrite for WriteHalf<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let ready = Pin::new(&mut self.w).poll_write(cx, buf);
        if let Poll::Ready(Ok(bytes)) = ready {
            self.gauge.update(bytes as u64);
        }
        ready
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.w).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.w).poll_shutdown(cx)
    }
}
