//! Audio Backend
//!
//! Cross-platform audio abstraction with spatial audio support.

use std::sync::Arc;
use parking_lot::RwLock;
use glam::Vec3;

/// Audio backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioBackend {
    /// Platform-native audio (AAudio on Android, AVAudioEngine on iOS)
    Native,
    /// OpenAL-based audio
    OpenAL,
    /// Web Audio API
    WebAudio,
    /// Null audio (no sound)
    Null,
}

impl AudioBackend {
    /// Get the recommended backend for the current platform
    pub fn recommended() -> Self {
        #[cfg(target_os = "android")]
        return Self::Native;
        
        #[cfg(target_os = "ios")]
        return Self::Native;
        
        #[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
        return Self::Native;
        
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        return Self::Null;
    }
}

/// Audio sample format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// 16-bit signed integer
    I16,
    /// 32-bit floating point
    F32,
}

/// Audio channel layout
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Surround51,
    Surround71,
}

impl ChannelLayout {
    /// Get the number of channels
    pub fn channel_count(&self) -> usize {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
            Self::Surround51 => 6,
            Self::Surround71 => 8,
        }
    }
}

/// Audio configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Channel layout
    pub channels: ChannelLayout,
    /// Sample format
    pub format: SampleFormat,
    /// Buffer size in samples
    pub buffer_size: usize,
    /// Enable spatial audio
    pub spatial_audio: bool,
    /// Enable HRTF (Head-Related Transfer Function) for binaural audio
    pub hrtf_enabled: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: ChannelLayout::Stereo,
            format: SampleFormat::F32,
            buffer_size: 512,
            spatial_audio: true,
            hrtf_enabled: false,
        }
    }
}

/// Audio source properties
#[derive(Debug, Clone)]
pub struct AudioSourceProperties {
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pitch multiplier
    pub pitch: f32,
    /// Whether the source loops
    pub looping: bool,
    /// Position in 3D space (for spatial audio)
    pub position: Vec3,
    /// Velocity (for Doppler effect)
    pub velocity: Vec3,
    /// Maximum distance for attenuation
    pub max_distance: f32,
    /// Reference distance for attenuation
    pub reference_distance: f32,
    /// Rolloff factor
    pub rolloff: f32,
}

impl Default for AudioSourceProperties {
    fn default() -> Self {
        Self {
            volume: 1.0,
            pitch: 1.0,
            looping: false,
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            max_distance: 100.0,
            reference_distance: 1.0,
            rolloff: 1.0,
        }
    }
}

/// Audio listener properties
#[derive(Debug, Clone)]
pub struct AudioListenerProperties {
    /// Position in 3D space
    pub position: Vec3,
    /// Forward direction
    pub forward: Vec3,
    /// Up direction
    pub up: Vec3,
    /// Velocity (for Doppler effect)
    pub velocity: Vec3,
    /// Master volume
    pub volume: f32,
}

impl Default for AudioListenerProperties {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            velocity: Vec3::ZERO,
            volume: 1.0,
        }
    }
}

/// Audio bus for mixing and effects
#[derive(Debug, Clone)]
pub struct AudioBus {
    /// Bus name
    pub name: String,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Muted state
    pub muted: bool,
    /// Parent bus (for hierarchical mixing)
    pub parent: Option<String>,
}

impl AudioBus {
    /// Create a new audio bus
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: 1.0,
            muted: false,
            parent: None,
        }
    }

    /// Create a bus with a parent
    pub fn with_parent(name: impl Into<String>, parent: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            volume: 1.0,
            muted: false,
            parent: Some(parent.into()),
        }
    }
}

/// Audio system state
#[derive(Debug)]
pub struct AudioSystem {
    config: AudioConfig,
    backend: AudioBackend,
    listener: Arc<RwLock<AudioListenerProperties>>,
    master_volume: f32,
    paused: bool,
}

impl AudioSystem {
    /// Create a new audio system
    pub fn new(config: AudioConfig) -> Self {
        Self {
            config,
            backend: AudioBackend::recommended(),
            listener: Arc::new(RwLock::new(AudioListenerProperties::default())),
            master_volume: 1.0,
            paused: false,
        }
    }

    /// Get the audio configuration
    pub fn config(&self) -> &AudioConfig {
        &self.config
    }

    /// Get the audio backend
    pub fn backend(&self) -> AudioBackend {
        self.backend
    }

    /// Get the master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    /// Set the master volume
    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    /// Check if the audio system is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Pause all audio
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume all audio
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Update the listener properties
    pub fn set_listener(&self, properties: AudioListenerProperties) {
        *self.listener.write() = properties;
    }

    /// Get the listener properties
    pub fn listener(&self) -> AudioListenerProperties {
        self.listener.read().clone()
    }

    /// Handle mobile focus loss (pause audio)
    pub fn on_focus_lost(&mut self) {
        self.pause();
    }

    /// Handle mobile focus gain (resume audio)
    pub fn on_focus_gained(&mut self) {
        self.resume();
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new(AudioConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, ChannelLayout::Stereo);
    }

    #[test]
    fn test_channel_layout_count() {
        assert_eq!(ChannelLayout::Mono.channel_count(), 1);
        assert_eq!(ChannelLayout::Stereo.channel_count(), 2);
        assert_eq!(ChannelLayout::Surround51.channel_count(), 6);
        assert_eq!(ChannelLayout::Surround71.channel_count(), 8);
    }

    #[test]
    fn test_audio_system() {
        let mut system = AudioSystem::new(AudioConfig::default());
        
        assert_eq!(system.master_volume(), 1.0);
        
        system.set_master_volume(0.5);
        assert_eq!(system.master_volume(), 0.5);
        
        assert!(!system.is_paused());
        system.pause();
        assert!(system.is_paused());
        system.resume();
        assert!(!system.is_paused());
    }

    #[test]
    fn test_audio_listener() {
        let system = AudioSystem::new(AudioConfig::default());
        
        let listener = AudioListenerProperties {
            position: Vec3::new(1.0, 2.0, 3.0),
            ..Default::default()
        };
        
        system.set_listener(listener.clone());
        
        let retrieved = system.listener();
        assert_eq!(retrieved.position, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_audio_bus() {
        let master = AudioBus::new("Master");
        assert_eq!(master.name, "Master");
        assert_eq!(master.volume, 1.0);
        
        let sfx = AudioBus::with_parent("SFX", "Master");
        assert_eq!(sfx.parent, Some(String::from("Master")));
    }
}
