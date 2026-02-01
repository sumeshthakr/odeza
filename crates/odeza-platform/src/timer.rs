//! High-Resolution Timers
//!
//! Cross-platform timing utilities for profiling and telemetry.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::sync::atomic::{AtomicU64, Ordering};

/// High-resolution timestamp
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Get the current timestamp
    pub fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::ZERO)
                .as_nanos() as u64,
        )
    }

    /// Get timestamp as nanoseconds since epoch
    pub fn as_nanos(&self) -> u64 {
        self.0
    }

    /// Get timestamp as microseconds since epoch
    pub fn as_micros(&self) -> u64 {
        self.0 / 1000
    }

    /// Get timestamp as milliseconds since epoch
    pub fn as_millis(&self) -> u64 {
        self.0 / 1_000_000
    }

    /// Get timestamp as seconds since epoch
    pub fn as_secs(&self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }

    /// Get duration since this timestamp
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        Duration::from_nanos(now.0.saturating_sub(self.0))
    }
}

/// High-resolution timer for performance measurement
#[derive(Debug)]
pub struct HighResTimer {
    start: Instant,
    last_lap: Instant,
}

impl HighResTimer {
    /// Create and start a new timer
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last_lap: now,
        }
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        let now = Instant::now();
        self.start = now;
        self.last_lap = now;
    }

    /// Get elapsed time since timer start
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_millis(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    /// Get elapsed time in microseconds
    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Get elapsed time in nanoseconds
    pub fn elapsed_nanos(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }

    /// Get time since last lap and reset lap timer
    pub fn lap(&mut self) -> Duration {
        let now = Instant::now();
        let duration = now - self.last_lap;
        self.last_lap = now;
        duration
    }

    /// Get lap time in milliseconds
    pub fn lap_millis(&mut self) -> f64 {
        self.lap().as_secs_f64() * 1000.0
    }
}

impl Default for HighResTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scoped timer that records duration on drop
pub struct ScopedTimer<'a> {
    name: &'a str,
    start: Instant,
    callback: Option<Box<dyn FnMut(&str, Duration) + 'a>>,
}

impl<'a> ScopedTimer<'a> {
    /// Create a new scoped timer with a callback
    pub fn new<F>(name: &'a str, callback: F) -> Self
    where
        F: FnMut(&str, Duration) + 'a,
    {
        Self {
            name,
            start: Instant::now(),
            callback: Some(Box::new(callback)),
        }
    }

    /// Create a scoped timer that logs to tracing
    pub fn traced(name: &'a str) -> Self {
        Self {
            name,
            start: Instant::now(),
            callback: None,
        }
    }

    /// Get elapsed time so far
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for ScopedTimer<'_> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        if let Some(mut callback) = self.callback.take() {
            callback(self.name, duration);
        } else {
            // Log via tracing
            tracing::debug!(
                target: "timing",
                name = self.name,
                duration_us = duration.as_micros() as u64,
                "Timer completed"
            );
        }
    }
}

/// Rolling average timer for frame time tracking
#[derive(Debug)]
pub struct RollingTimer {
    samples: Vec<f64>,
    index: usize,
    count: usize,
}

impl RollingTimer {
    /// Create a new rolling timer with the given sample count
    pub fn new(sample_count: usize) -> Self {
        Self {
            samples: vec![0.0; sample_count.max(1)],
            index: 0,
            count: 0,
        }
    }

    /// Record a new sample (in seconds)
    pub fn record(&mut self, sample: f64) {
        self.samples[self.index] = sample;
        self.index = (self.index + 1) % self.samples.len();
        self.count = (self.count + 1).min(self.samples.len());
    }

    /// Record a new sample from a duration
    pub fn record_duration(&mut self, duration: Duration) {
        self.record(duration.as_secs_f64());
    }

    /// Get the average sample value
    pub fn average(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        self.samples.iter().take(self.count).sum::<f64>() / self.count as f64
    }

    /// Get the average as FPS
    pub fn average_fps(&self) -> f64 {
        let avg = self.average();
        if avg > 0.0 {
            1.0 / avg
        } else {
            0.0
        }
    }

    /// Get the minimum sample value
    pub fn min(&self) -> f64 {
        self.samples
            .iter()
            .take(self.count)
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get the maximum sample value
    pub fn max(&self) -> f64 {
        self.samples
            .iter()
            .take(self.count)
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get the sample count
    pub fn count(&self) -> usize {
        self.count
    }

    /// Reset all samples
    pub fn reset(&mut self) {
        self.samples.fill(0.0);
        self.index = 0;
        self.count = 0;
    }
}

/// Frame time tracker with statistics
#[derive(Debug)]
pub struct FrameTimer {
    last_frame: Instant,
    rolling: RollingTimer,
    frame_count: u64,
    total_time: f64,
}

impl FrameTimer {
    /// Create a new frame timer
    pub fn new() -> Self {
        Self {
            last_frame: Instant::now(),
            rolling: RollingTimer::new(60),
            frame_count: 0,
            total_time: 0.0,
        }
    }

    /// Mark the start of a new frame, returns delta time
    pub fn tick(&mut self) -> f64 {
        let now = Instant::now();
        let delta = (now - self.last_frame).as_secs_f64();
        self.last_frame = now;
        
        self.rolling.record(delta);
        self.frame_count += 1;
        self.total_time += delta;
        
        delta
    }

    /// Get the current FPS (rolling average)
    pub fn fps(&self) -> f64 {
        self.rolling.average_fps()
    }

    /// Get the average frame time in milliseconds
    pub fn average_frame_time_ms(&self) -> f64 {
        self.rolling.average() * 1000.0
    }

    /// Get the minimum frame time in milliseconds
    pub fn min_frame_time_ms(&self) -> f64 {
        self.rolling.min() * 1000.0
    }

    /// Get the maximum frame time in milliseconds
    pub fn max_frame_time_ms(&self) -> f64 {
        self.rolling.max() * 1000.0
    }

    /// Get total frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get total elapsed time
    pub fn total_time(&self) -> f64 {
        self.total_time
    }

    /// Get lifetime average FPS
    pub fn lifetime_fps(&self) -> f64 {
        if self.total_time > 0.0 {
            self.frame_count as f64 / self.total_time
        } else {
            0.0
        }
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance counter for tracking operation counts and timing
#[derive(Debug)]
pub struct PerfCounter {
    name: String,
    count: AtomicU64,
    total_nanos: AtomicU64,
}

impl PerfCounter {
    /// Create a new performance counter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            count: AtomicU64::new(0),
            total_nanos: AtomicU64::new(0),
        }
    }

    /// Record a timed operation
    pub fn record(&self, duration: Duration) {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_nanos.fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get the counter name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the operation count
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Get total time in nanoseconds
    pub fn total_nanos(&self) -> u64 {
        self.total_nanos.load(Ordering::Relaxed)
    }

    /// Get average time per operation in nanoseconds
    pub fn average_nanos(&self) -> f64 {
        let count = self.count();
        if count > 0 {
            self.total_nanos() as f64 / count as f64
        } else {
            0.0
        }
    }

    /// Get average time per operation in milliseconds
    pub fn average_millis(&self) -> f64 {
        self.average_nanos() / 1_000_000.0
    }

    /// Reset the counter
    pub fn reset(&self) {
        self.count.store(0, Ordering::Relaxed);
        self.total_nanos.store(0, Ordering::Relaxed);
    }

    /// Time an operation and record it
    pub fn time<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        self.record(start.elapsed());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        let t1 = Timestamp::now();
        std::thread::sleep(Duration::from_millis(10));
        let t2 = Timestamp::now();
        
        assert!(t2 > t1);
        assert!(t1.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_high_res_timer() {
        let mut timer = HighResTimer::new();
        std::thread::sleep(Duration::from_millis(10));
        
        assert!(timer.elapsed() >= Duration::from_millis(10));
        
        let lap = timer.lap();
        assert!(lap >= Duration::from_millis(10));
    }

    #[test]
    fn test_rolling_timer() {
        let mut timer = RollingTimer::new(5);
        
        timer.record(0.016);
        timer.record(0.017);
        timer.record(0.015);
        
        assert_eq!(timer.count(), 3);
        assert!((timer.average() - 0.016).abs() < 0.001);
        assert_eq!(timer.min(), 0.015);
        assert_eq!(timer.max(), 0.017);
    }

    #[test]
    fn test_frame_timer() {
        let mut timer = FrameTimer::new();
        
        for _ in 0..5 {
            std::thread::sleep(Duration::from_millis(16));
            timer.tick();
        }
        
        assert_eq!(timer.frame_count(), 5);
        assert!(timer.fps() > 0.0);
    }

    #[test]
    fn test_perf_counter() {
        let counter = PerfCounter::new("test");
        
        for _ in 0..10 {
            counter.time(|| {
                std::thread::sleep(Duration::from_micros(100));
            });
        }
        
        assert_eq!(counter.count(), 10);
        assert!(counter.average_nanos() > 0.0);
    }

    #[test]
    fn test_scoped_timer() {
        let mut recorded_duration = Duration::ZERO;
        
        {
            let _timer = ScopedTimer::new("test", |_name, duration| {
                recorded_duration = duration;
            });
            std::thread::sleep(Duration::from_millis(10));
        }
        
        assert!(recorded_duration >= Duration::from_millis(10));
    }
}
