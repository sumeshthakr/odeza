//! # Odeza Platform
//!
//! Platform abstraction layer (HAL) for the Odeza game engine.
//!
//! This crate provides cross-platform abstractions for:
//! - **Window**: Window creation and management
//! - **Input**: Touch, gamepad, mouse, and keyboard input
//! - **FileSystem**: File I/O with async support
//! - **Threading**: Threading primitives and atomics
//! - **Timers**: High-resolution timers and telemetry
//! - **Audio**: Platform-native audio backend selection
//!
//! ## Supported Platforms
//! - Android (arm64)
//! - iOS (arm64)  
//! - Windows (x64)
//! - Linux (x64)

pub mod window;
pub mod input;
pub mod filesystem;
pub mod threading;
pub mod timer;
pub mod audio;

pub use window::{Window, WindowConfig, WindowEvent};
pub use input::{InputState, InputEvent, GamepadState, TouchState};
pub use filesystem::{FileSystem, FileHandle, FileMode};
pub use threading::{Thread, ThreadPool};
pub use timer::{HighResTimer, Timestamp};

use thiserror::Error;

/// Platform-specific errors
#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),
    
    #[error("File I/O error: {0}")]
    FileIO(#[from] std::io::Error),
    
    #[error("Graphics backend not supported: {0}")]
    GraphicsNotSupported(String),
    
    #[error("Audio backend error: {0}")]
    AudioError(String),
    
    #[error("Platform not supported: {0}")]
    NotSupported(String),
}

/// Result type for platform operations
pub type PlatformResult<T> = Result<T, PlatformError>;

/// Platform identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// Android (arm64)
    Android,
    /// iOS (arm64)
    Ios,
    /// Windows (x64)
    Windows,
    /// Linux (x64)
    Linux,
    /// macOS (for development)
    MacOs,
    /// Unknown platform
    Unknown,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        #[cfg(target_os = "android")]
        return Platform::Android;
        
        #[cfg(target_os = "ios")]
        return Platform::Ios;
        
        #[cfg(target_os = "windows")]
        return Platform::Windows;
        
        #[cfg(target_os = "linux")]
        return Platform::Linux;
        
        #[cfg(target_os = "macos")]
        return Platform::MacOs;
        
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "windows",
            target_os = "linux",
            target_os = "macos"
        )))]
        return Platform::Unknown;
    }

    /// Check if this is a mobile platform
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::Android | Platform::Ios)
    }

    /// Check if this is a desktop platform
    pub fn is_desktop(&self) -> bool {
        matches!(self, Platform::Windows | Platform::Linux | Platform::MacOs)
    }

    /// Check if touch input is the primary input method
    pub fn is_touch_primary(&self) -> bool {
        self.is_mobile()
    }

    /// Get the recommended graphics backend
    pub fn recommended_graphics_backend(&self) -> GraphicsBackend {
        match self {
            Platform::Android => GraphicsBackend::Vulkan,
            Platform::Ios => GraphicsBackend::Metal,
            Platform::Windows => GraphicsBackend::Vulkan,
            Platform::Linux => GraphicsBackend::Vulkan,
            Platform::MacOs => GraphicsBackend::Metal,
            Platform::Unknown => GraphicsBackend::OpenGL,
        }
    }
}

/// Graphics backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GraphicsBackend {
    /// Vulkan (Android, Windows, Linux)
    Vulkan,
    /// Metal (iOS, macOS)
    Metal,
    /// DirectX 12 (Windows)
    DirectX12,
    /// OpenGL ES (fallback for older devices)
    OpenGL,
    /// WebGPU (future web support)
    WebGPU,
}

impl GraphicsBackend {
    /// Check if ray tracing is potentially supported
    pub fn supports_raytracing(&self) -> bool {
        matches!(self, GraphicsBackend::Vulkan | GraphicsBackend::Metal | GraphicsBackend::DirectX12)
    }

    /// Get wgpu backend equivalent
    pub fn to_wgpu_backend(&self) -> wgpu::Backends {
        match self {
            GraphicsBackend::Vulkan => wgpu::Backends::VULKAN,
            GraphicsBackend::Metal => wgpu::Backends::METAL,
            GraphicsBackend::DirectX12 => wgpu::Backends::DX12,
            GraphicsBackend::OpenGL => wgpu::Backends::GL,
            GraphicsBackend::WebGPU => wgpu::Backends::BROWSER_WEBGPU,
        }
    }
}

/// Device capabilities detected at runtime
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    /// Platform
    pub platform: Platform,
    /// Graphics backend
    pub graphics_backend: GraphicsBackend,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// Total system RAM in bytes
    pub total_ram: u64,
    /// GPU name
    pub gpu_name: String,
    /// Whether ray tracing is available
    pub raytracing_available: bool,
    /// Whether hardware tessellation is available
    pub tessellation_available: bool,
    /// Maximum texture size
    pub max_texture_size: u32,
    /// Whether compute shaders are available
    pub compute_available: bool,
}

impl DeviceCapabilities {
    /// Detect device capabilities
    pub fn detect() -> Self {
        let platform = Platform::current();
        let cpu_cores = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        
        Self {
            platform,
            graphics_backend: platform.recommended_graphics_backend(),
            cpu_cores,
            total_ram: Self::detect_ram(),
            gpu_name: String::from("Unknown"),
            raytracing_available: false,
            tessellation_available: true,
            max_texture_size: 8192,
            compute_available: true,
        }
    }

    fn detect_ram() -> u64 {
        // Platform-specific RAM detection would go here
        // For now, return a reasonable default
        4 * 1024 * 1024 * 1024 // 4 GB
    }

    /// Get recommended performance tier based on capabilities
    pub fn recommended_tier(&self) -> odeza_core::PerformanceTier {
        use odeza_core::PerformanceTier;
        
        if self.platform.is_mobile() {
            if self.total_ram >= 8 * 1024 * 1024 * 1024 && self.raytracing_available {
                PerformanceTier::HighEnd
            } else {
                PerformanceTier::Mobile
            }
        } else {
            PerformanceTier::HandheldPC
        }
    }
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self::detect()
    }
}

use wgpu;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        // Should return a valid platform on any supported system
        assert!(matches!(
            platform,
            Platform::Android | Platform::Ios | Platform::Windows | 
            Platform::Linux | Platform::MacOs | Platform::Unknown
        ));
    }

    #[test]
    fn test_platform_characteristics() {
        assert!(Platform::Android.is_mobile());
        assert!(Platform::Ios.is_mobile());
        assert!(Platform::Windows.is_desktop());
        assert!(Platform::Linux.is_desktop());
    }

    #[test]
    fn test_graphics_backend() {
        let android_backend = Platform::Android.recommended_graphics_backend();
        assert_eq!(android_backend, GraphicsBackend::Vulkan);
        
        let ios_backend = Platform::Ios.recommended_graphics_backend();
        assert_eq!(ios_backend, GraphicsBackend::Metal);
    }

    #[test]
    fn test_device_capabilities() {
        let caps = DeviceCapabilities::detect();
        assert!(caps.cpu_cores >= 1);
        assert!(caps.max_texture_size >= 1024);
    }
}
