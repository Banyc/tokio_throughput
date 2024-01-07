use std::{pin::Pin, task::Poll};

use tokio::io::AsyncRead;

use crate::Gauge;

pub struct ReadHalf<R> {
    r: R,
    gauge: Gauge,
}
impl<R> ReadHalf<R> {
    pub fn new(r: R, gauge: Gauge) -> Self {
        Self { r, gauge }
    }
}
impl<R: AsyncRead + Unpin> AsyncRead for ReadHalf<R> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let ready = Pin::new(&mut self.r).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = ready {
            self.gauge.update(buf.filled().len() as u64);
        }
        ready
    }
}
