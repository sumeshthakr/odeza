//! # Odeza Audio
//!
//! Audio system for the Odeza engine.
//!
//! ## Features
//! - Spatial audio with HRTF support
//! - Mixer graph with buses and effects
//! - Streaming audio for long assets
//! - Mobile focus/interrupt handling

pub use odeza_platform::audio::*;

/// Audio asset reference
#[derive(Debug, Clone)]
pub struct AudioClip {
    pub id: u64,
    pub name: String,
    pub duration: f32,
    pub sample_rate: u32,
    pub channels: u32,
    pub streaming: bool,
}

impl Default for AudioClip {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            duration: 0.0,
            sample_rate: 48000,
            channels: 2,
            streaming: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_clip() {
        let clip = AudioClip::default();
        assert_eq!(clip.sample_rate, 48000);
    }
}
