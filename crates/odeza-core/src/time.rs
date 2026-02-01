//! Time Management
//!
//! Time-step model supporting:
//! - Variable render step
//! - Fixed-step simulation
//! - Deterministic-friendly hooks for networking/replays

use std::time::{Duration, Instant};

/// Delta time wrapper for type safety
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeltaTime(pub f64);

impl DeltaTime {
    /// Create a new delta time from seconds
    pub fn from_secs(secs: f64) -> Self {
        Self(secs)
    }

    /// Create a new delta time from milliseconds
    pub fn from_millis(millis: f64) -> Self {
        Self(millis / 1000.0)
    }

    /// Get the delta time in seconds
    pub fn as_secs(&self) -> f64 {
        self.0
    }

    /// Get the delta time in milliseconds
    pub fn as_millis(&self) -> f64 {
        self.0 * 1000.0
    }

    /// Get the delta time as f32 (commonly needed for graphics)
    pub fn as_secs_f32(&self) -> f32 {
        self.0 as f32
    }
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}

impl From<Duration> for DeltaTime {
    fn from(duration: Duration) -> Self {
        Self(duration.as_secs_f64())
    }
}

/// Fixed time step configuration
#[derive(Debug, Clone, Copy)]
pub struct FixedTimeStep {
    /// Fixed timestep in seconds
    pub step: f64,
    /// Maximum number of fixed updates per frame (to prevent spiral of death)
    pub max_updates: u32,
}

impl Default for FixedTimeStep {
    fn default() -> Self {
        Self {
            step: 1.0 / 60.0, // 60 Hz
            max_updates: 8,
        }
    }
}

impl FixedTimeStep {
    /// Create a new fixed time step with the given frequency
    pub fn from_hz(hz: f64) -> Self {
        Self {
            step: 1.0 / hz,
            max_updates: 8,
        }
    }

    /// Create a new fixed time step with the given step size
    pub fn from_step(step: f64) -> Self {
        Self {
            step,
            max_updates: 8,
        }
    }

    /// Set the maximum number of updates per frame
    pub fn with_max_updates(mut self, max: u32) -> Self {
        self.max_updates = max;
        self
    }
}

/// Time manager for tracking frame timing and fixed updates
pub struct TimeManager {
    /// Time when the engine started
    start_time: Instant,
    /// Time of the last frame
    last_frame_time: Instant,
    /// Total elapsed time since start
    total_time: f64,
    /// Delta time of the last frame
    delta_time: f64,
    /// Accumulated time for fixed updates
    fixed_accumulator: f64,
    /// Frame count
    frame_count: u64,
    /// Fixed update count
    fixed_update_count: u64,
    /// Current frame rate (smoothed)
    fps: f64,
    /// Frame time history for FPS smoothing
    frame_times: [f64; 60],
    /// Current index in frame time history
    frame_time_index: usize,
    /// Time scale (for slow motion or fast forward)
    time_scale: f64,
    /// Whether the game is paused
    paused: bool,
}

impl TimeManager {
    /// Create a new time manager
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame_time: now,
            total_time: 0.0,
            delta_time: 1.0 / 60.0,
            fixed_accumulator: 0.0,
            frame_count: 0,
            fixed_update_count: 0,
            fps: 60.0,
            frame_times: [1.0 / 60.0; 60],
            frame_time_index: 0,
            time_scale: 1.0,
            paused: false,
        }
    }

    /// Update the time manager for a new frame
    pub fn update(&mut self, delta_time: f64) {
        let now = Instant::now();
        
        // Clamp delta time to prevent extreme values
        let clamped_dt = delta_time.min(0.25).max(0.0001);
        
        // Apply time scale
        let scaled_dt = if self.paused { 0.0 } else { clamped_dt * self.time_scale };
        
        self.delta_time = scaled_dt;
        self.total_time += scaled_dt;
        self.fixed_accumulator += scaled_dt;
        self.frame_count += 1;
        
        // Update FPS calculation
        self.frame_times[self.frame_time_index] = delta_time;
        self.frame_time_index = (self.frame_time_index + 1) % self.frame_times.len();
        
        let avg_frame_time: f64 = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        self.fps = if avg_frame_time > 0.0 { 1.0 / avg_frame_time } else { 0.0 };
        
        self.last_frame_time = now;
    }

    /// Check if a fixed update should run
    pub fn should_run_fixed_update(&self, fixed_step: f64) -> bool {
        self.fixed_accumulator >= fixed_step
    }

    /// Consume time for a fixed update
    pub fn consume_fixed_update(&mut self, fixed_step: f64) {
        self.fixed_accumulator -= fixed_step;
        self.fixed_update_count += 1;
    }

    /// Get the number of fixed updates needed this frame
    pub fn fixed_updates_needed(&self, config: &FixedTimeStep) -> u32 {
        let updates = (self.fixed_accumulator / config.step) as u32;
        updates.min(config.max_updates)
    }

    /// Get the interpolation factor for rendering between fixed updates
    pub fn fixed_interpolation(&self, fixed_step: f64) -> f64 {
        (self.fixed_accumulator / fixed_step).clamp(0.0, 1.0)
    }

    /// Get the delta time for the current frame
    pub fn delta_time(&self) -> DeltaTime {
        DeltaTime(self.delta_time)
    }

    /// Get the raw delta time (unscaled)
    pub fn raw_delta_time(&self) -> f64 {
        self.delta_time / self.time_scale.max(0.0001)
    }

    /// Get the total elapsed time
    pub fn total_time(&self) -> f64 {
        self.total_time
    }

    /// Get the frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the fixed update count
    pub fn fixed_update_count(&self) -> u64 {
        self.fixed_update_count
    }

    /// Get the current FPS (smoothed)
    pub fn fps(&self) -> f64 {
        self.fps
    }

    /// Get the time scale
    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    /// Set the time scale
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.max(0.0);
    }

    /// Check if the game is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause the game
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the game
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Get time since engine start (wall clock)
    pub fn time_since_start(&self) -> Duration {
        self.start_time.elapsed()
    }
}

impl Default for TimeManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Stopwatch for measuring elapsed time
#[derive(Debug, Clone)]
pub struct Stopwatch {
    start: Instant,
    elapsed: Duration,
    running: bool,
}

impl Stopwatch {
    /// Create and start a new stopwatch
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Duration::ZERO,
            running: true,
        }
    }

    /// Create a stopped stopwatch
    pub fn stopped() -> Self {
        Self {
            start: Instant::now(),
            elapsed: Duration::ZERO,
            running: false,
        }
    }

    /// Start or resume the stopwatch
    pub fn start(&mut self) {
        if !self.running {
            self.start = Instant::now();
            self.running = true;
        }
    }

    /// Stop the stopwatch
    pub fn stop(&mut self) {
        if self.running {
            self.elapsed += self.start.elapsed();
            self.running = false;
        }
    }

    /// Reset the stopwatch
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.start = Instant::now();
    }

    /// Get the elapsed time
    pub fn elapsed(&self) -> Duration {
        if self.running {
            self.elapsed + self.start.elapsed()
        } else {
            self.elapsed
        }
    }

    /// Get the elapsed time in seconds
    pub fn elapsed_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    /// Check if the stopwatch is running
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer that triggers after a specified duration
#[derive(Debug, Clone)]
pub struct Timer {
    duration: Duration,
    elapsed: Duration,
    repeating: bool,
    finished: bool,
}

impl Timer {
    /// Create a new one-shot timer
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating: false,
            finished: false,
        }
    }

    /// Create a new repeating timer
    pub fn repeating(duration: Duration) -> Self {
        Self {
            duration,
            elapsed: Duration::ZERO,
            repeating: true,
            finished: false,
        }
    }

    /// Update the timer with delta time
    pub fn tick(&mut self, delta: Duration) -> bool {
        if self.finished && !self.repeating {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
            }
            true
        } else {
            false
        }
    }

    /// Check if the timer has finished
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get the progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
    }

    /// Reset the timer
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.finished = false;
    }

    /// Get the remaining time
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_time() {
        let dt = DeltaTime::from_secs(0.016);
        assert!((dt.as_secs() - 0.016).abs() < 0.0001);
        assert!((dt.as_millis() - 16.0).abs() < 0.1);
    }

    #[test]
    fn test_fixed_time_step() {
        let fts = FixedTimeStep::from_hz(60.0);
        assert!((fts.step - 1.0 / 60.0).abs() < 0.0001);
    }

    #[test]
    fn test_time_manager() {
        let mut tm = TimeManager::new();
        
        tm.update(0.016);
        assert!(tm.frame_count() == 1);
        assert!(tm.delta_time().as_secs() > 0.0);
    }

    #[test]
    fn test_time_manager_pause() {
        let mut tm = TimeManager::new();
        
        tm.pause();
        assert!(tm.is_paused());
        
        let total_before = tm.total_time();
        tm.update(0.016);
        
        // Total time should not increase when paused
        assert_eq!(tm.total_time(), total_before);
        
        tm.resume();
        assert!(!tm.is_paused());
    }

    #[test]
    fn test_time_scale() {
        let mut tm = TimeManager::new();
        tm.set_time_scale(2.0);
        
        tm.update(0.016);
        
        // Delta time should be scaled
        assert!((tm.delta_time().as_secs() - 0.032).abs() < 0.001);
    }

    #[test]
    fn test_fixed_updates_needed() {
        let mut tm = TimeManager::new();
        let config = FixedTimeStep::from_hz(60.0);
        
        // Accumulate enough time for 3 fixed updates
        tm.update(config.step * 3.0);
        
        assert_eq!(tm.fixed_updates_needed(&config), 3);
    }

    #[test]
    fn test_stopwatch() {
        let mut sw = Stopwatch::new();
        assert!(sw.is_running());
        
        std::thread::sleep(Duration::from_millis(10));
        sw.stop();
        
        assert!(!sw.is_running());
        assert!(sw.elapsed() >= Duration::from_millis(10));
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new(Duration::from_millis(100));
        
        assert!(!timer.tick(Duration::from_millis(50)));
        assert!(!timer.is_finished());
        assert!((timer.progress() - 0.5).abs() < 0.01);
        
        assert!(timer.tick(Duration::from_millis(60)));
        assert!(timer.is_finished());
    }

    #[test]
    fn test_repeating_timer() {
        let mut timer = Timer::repeating(Duration::from_millis(100));
        
        assert!(timer.tick(Duration::from_millis(150)));
        assert!(!timer.is_finished()); // Repeating timer never "finishes"
        
        // Should have rolled over with 50ms remaining
        assert!(timer.remaining() < Duration::from_millis(60));
    }
}
