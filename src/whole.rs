use std::{pin::Pin, task::Poll};

use tokio::io::{AsyncRead, AsyncWrite};

use crate::Gauge;

pub struct ReadGauge(pub Gauge);
pub struct WriteGauge(pub Gauge);

pub struct WholeStream<S> {
    s: S,
    r: Gauge,
    w: Gauge,
}
impl<S> WholeStream<S> {
    pub fn new(s: S, r: ReadGauge, w: WriteGauge) -> Self {
        Self { s, r: r.0, w: w.0 }
    }
}
impl<S: AsyncRead + Unpin> AsyncRead for WholeStream<S> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let ready = Pin::new(&mut self.s).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = ready {
            self.r.update(buf.filled().len() as u64);
        }
        ready
    }
}
impl<S: AsyncWrite + Unpin> AsyncWrite for WholeStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let ready = Pin::new(&mut self.s).poll_write(cx, buf);
        if let Poll::Ready(Ok(bytes)) = ready {
            self.w.update(bytes as u64);
        }
        ready
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.s).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.s).poll_shutdown(cx)
    }
}
