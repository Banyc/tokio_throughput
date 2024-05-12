use std::{
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use strict_num::NormalizedF64;

const PERIOD_DURATION: Duration = Duration::from_secs(1);
const NUM_PERIODS: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(16) };
const ALPHA: NormalizedF64 =
    unsafe { NormalizedF64::new_unchecked(2. / (1 + NUM_PERIODS.get()) as f64) };

pub fn gauge() -> (GaugeHandle, Gauge) {
    let bytes = Arc::new(AtomicU64::new(0));
    let handle = GaugeHandle::new(Arc::clone(&bytes));
    let gauge = Gauge::new(bytes);
    (handle, gauge)
}

#[derive(Debug)]
pub struct GaugeHandle {
    prev: Option<Instant>,
    thruput: f64,
    bytes: Arc<AtomicU64>,
    total_bytes: u64,
}
impl GaugeHandle {
    fn new(bytes: Arc<AtomicU64>) -> Self {
        Self {
            prev: None,
            thruput: 0.0,
            bytes,
            total_bytes: 0,
        }
    }

    /// Update `thruput` using EMA and `total_bytes` using summation.
    pub fn update(&mut self, now: Instant) {
        let d = now.duration_since(*self.prev.get_or_insert(now));
        if d < PERIOD_DURATION {
            return;
        }

        self.prev = Some(now);
        let bytes = self.bytes.swap(0, Ordering::Relaxed);
        self.total_bytes += bytes;

        // Update `thruput`
        let thruput = bytes as f64 / d.as_secs_f64();
        if d > PERIOD_DURATION.mul_f64(NUM_PERIODS.get() as f64) {
            self.thruput = thruput;
            return;
        }
        let periods = (d.as_secs_f64() / PERIOD_DURATION.as_secs_f64()) as usize;
        for _ in 0..periods {
            self.thruput = ALPHA.get() * thruput + (1. - ALPHA.get()) * self.thruput;
        }
    }

    pub fn thruput(&self) -> f64 {
        self.thruput
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

#[derive(Debug)]
pub struct Gauge {
    bytes: Arc<AtomicU64>,
}
impl Gauge {
    fn new(bytes: Arc<AtomicU64>) -> Self {
        Self { bytes }
    }

    pub fn update(&self, bytes: u64) {
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision() {
        let (mut handle, gauge) = gauge();
        const N: usize = 2 << 10;
        const BYTES: u64 = 2 << 10;
        let mut now = Instant::now();
        for _ in 0..N {
            gauge.update(BYTES);
            handle.update(now);
            now += Duration::from_secs(1);
        }
        assert!((handle.thruput() - BYTES as f64).abs() < 0.1);
    }
}
