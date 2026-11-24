use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub(super) fn timed<T>(step: &str, work: impl FnOnce() -> T) -> T {
    let started = cfg!(debug_assertions).then(Instant::now);
    let result = work();
    if let Some(started) = started {
        eprintln!("[photart-import] {step}: {:.2?}", started.elapsed());
    }
    result
}

#[derive(Default)]
pub(super) struct WorkerTiming(AtomicU64);

impl WorkerTiming {
    pub(super) fn measure<T>(&self, work: impl FnOnce() -> T) -> T {
        let started = cfg!(debug_assertions).then(Instant::now);
        let result = work();
        if let Some(started) = started {
            self.0
                .fetch_add(started.elapsed().as_micros() as u64, Ordering::Relaxed);
        }
        result
    }

    pub(super) fn total(&self) -> Duration {
        Duration::from_micros(self.0.load(Ordering::Relaxed))
    }
}
