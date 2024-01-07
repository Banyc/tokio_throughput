use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

use strict_num::NormalizedF64;

pub fn gauge(alpha: NormalizedF64) -> (GaugeHandle, Gauge) {
    let bytes = Arc::new(AtomicU64::new(0));
    let handle = GaugeHandle::new(alpha, Arc::clone(&bytes));
    let gauge = Gauge::new(bytes);
    (handle, gauge)
}

#[derive(Debug)]
pub struct GaugeHandle {
    alpha: NormalizedF64,
    prev: Option<Instant>,
    thruput: f64,
    bytes: Arc<AtomicU64>,
    total_bytes: u64,
}
impl GaugeHandle {
    fn new(alpha: NormalizedF64, bytes: Arc<AtomicU64>) -> Self {
        Self {
            alpha,
            prev: None,
            thruput: 0.0,
            bytes,
            total_bytes: 0,
        }
    }

    pub fn update(&mut self, now: Instant) {
        let d = now.duration_since(self.prev.unwrap_or(now));
        self.prev = Some(now);
        let d = d.as_secs_f64();
        if d == 0.0 {
            return;
        }
        let bytes = self.bytes.swap(0, Ordering::Relaxed);
        self.total_bytes += bytes;
        let thruput = bytes as f64 / d;
        self.thruput = self.alpha.get() * thruput + (1. - self.alpha.get()) * self.thruput;
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
