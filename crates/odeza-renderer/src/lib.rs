//! # Odeza Renderer
//!
//! Hybrid AAA renderer with ray tracing support for mobile and handheld PC.
//!
//! ## Features
//! - Frame Graph (render graph) for pass scheduling
//! - Forward+ (Clustered Forward) baseline renderer
//! - PBR shading model
//! - Ray tracing effects (reflections, shadows, AO, GI)
//! - Volumetric lighting and fog
//! - Temporal upscaling (TAA/TAAU)
//! - Virtual texturing for 4K asset support

pub mod frame_graph;
pub mod material;
pub mod lighting;
pub mod post;
pub mod raytracing;
pub mod volumetric;
pub mod texture;
pub mod mesh;
pub mod pipeline;

pub use frame_graph::{FrameGraph, RenderPass, RenderResource};
pub use material::{Material, MaterialInstance, PbrMaterial};
pub use lighting::{Light, LightType, LightingSystem};
pub use post::{PostProcess, TaaSettings, BloomSettings};

use odeza_core::PerformanceTier;
use thiserror::Error;

/// Renderer errors
#[derive(Error, Debug)]
pub enum RendererError {
    #[error("GPU device creation failed: {0}")]
    DeviceCreation(String),
    
    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),
    
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),
    
    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),
    
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    
    #[error("Out of GPU memory")]
    OutOfMemory,
}

/// Result type for renderer operations
pub type RendererResult<T> = Result<T, RendererError>;

/// Renderer quality tier settings
#[derive(Debug, Clone)]
pub struct QualitySettings {
    /// Performance tier
    pub tier: PerformanceTier,
    /// Shadow quality
    pub shadow_quality: ShadowQuality,
    /// Reflection quality
    pub reflection_quality: ReflectionQuality,
    /// Volumetric quality
    pub volumetric_quality: VolumetricQuality,
    /// Enable ray tracing effects
    pub raytracing_enabled: bool,
    /// Render scale (1.0 = native)
    pub render_scale: f32,
    /// Enable temporal upscaling
    pub temporal_upscaling: bool,
    /// Enable dynamic resolution
    pub dynamic_resolution: bool,
    /// Target frame time in ms
    pub target_frame_time_ms: f32,
}

impl QualitySettings {
    /// Create settings for mobile tier
    pub fn mobile() -> Self {
        Self {
            tier: PerformanceTier::Mobile,
            shadow_quality: ShadowQuality::Medium,
            reflection_quality: ReflectionQuality::ScreenSpace,
            volumetric_quality: VolumetricQuality::Low,
            raytracing_enabled: false,
            render_scale: 0.75,
            temporal_upscaling: true,
            dynamic_resolution: true,
            target_frame_time_ms: 33.33, // 30 fps
        }
    }

    /// Create settings for high-end mobile tier
    pub fn high_end() -> Self {
        Self {
            tier: PerformanceTier::HighEnd,
            shadow_quality: ShadowQuality::High,
            reflection_quality: ReflectionQuality::Hybrid,
            volumetric_quality: VolumetricQuality::Medium,
            raytracing_enabled: true,
            render_scale: 0.85,
            temporal_upscaling: true,
            dynamic_resolution: true,
            target_frame_time_ms: 16.67, // 60 fps
        }
    }

    /// Create settings for handheld PC tier
    pub fn handheld_pc() -> Self {
        Self {
            tier: PerformanceTier::HandheldPC,
            shadow_quality: ShadowQuality::Ultra,
            reflection_quality: ReflectionQuality::RayTraced,
            volumetric_quality: VolumetricQuality::High,
            raytracing_enabled: true,
            render_scale: 1.0,
            temporal_upscaling: true,
            dynamic_resolution: true,
            target_frame_time_ms: 16.67, // 60 fps
        }
    }
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self::mobile()
    }
}

/// Shadow quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
    Ultra,
}

/// Reflection quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReflectionQuality {
    Off,
    /// Probe-based reflections only
    Probes,
    /// Screen-space reflections
    ScreenSpace,
    /// SSR with RT fallback
    Hybrid,
    /// Full ray-traced reflections
    RayTraced,
}

/// Volumetric quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumetricQuality {
    Off,
    Low,
    Medium,
    High,
}

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Quality settings
    pub quality: QualitySettings,
    /// Preferred graphics backend
    pub backend: wgpu::Backends,
    /// Present mode
    pub present_mode: wgpu::PresentMode,
    /// Maximum frames in flight
    pub max_frames_in_flight: u32,
    /// Enable validation layers (debug)
    pub validation: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            quality: QualitySettings::default(),
            backend: wgpu::Backends::all(),
            present_mode: wgpu::PresentMode::AutoVsync,
            max_frames_in_flight: 2,
            validation: cfg!(debug_assertions),
        }
    }
}

/// GPU timing information
#[derive(Debug, Clone, Default)]
pub struct GpuTiming {
    /// Total frame time on GPU (ms)
    pub frame_time_ms: f32,
    /// Per-pass timings
    pub pass_timings: Vec<(String, f32)>,
}

/// Renderer statistics
#[derive(Debug, Clone, Default)]
pub struct RendererStats {
    /// GPU timing
    pub gpu_timing: GpuTiming,
    /// Draw calls this frame
    pub draw_calls: u32,
    /// Triangles rendered
    pub triangles: u32,
    /// Textures bound
    pub textures_bound: u32,
    /// Current render scale
    pub current_render_scale: f32,
    /// VRAM usage in bytes
    pub vram_usage: u64,
}

/// Main renderer instance
pub struct Renderer {
    config: RendererConfig,
    stats: RendererStats,
    frame_number: u64,
}

impl Renderer {
    /// Create a new renderer
    pub fn new(config: RendererConfig) -> RendererResult<Self> {
        Ok(Self {
            config,
            stats: RendererStats::default(),
            frame_number: 0,
        })
    }

    /// Get the renderer configuration
    pub fn config(&self) -> &RendererConfig {
        &self.config
    }

    /// Get renderer statistics
    pub fn stats(&self) -> &RendererStats {
        &self.stats
    }

    /// Get the current frame number
    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Begin a new frame
    pub fn begin_frame(&mut self) {
        self.frame_number += 1;
        self.stats = RendererStats::default();
    }

    /// End the current frame
    pub fn end_frame(&mut self) {
        // Frame statistics would be collected here
    }

    /// Update quality settings
    pub fn set_quality(&mut self, quality: QualitySettings) {
        self.config.quality = quality;
    }

    /// Check if ray tracing is available
    pub fn raytracing_available(&self) -> bool {
        // Would check GPU capabilities
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_settings() {
        let mobile = QualitySettings::mobile();
        assert_eq!(mobile.tier, PerformanceTier::Mobile);
        assert!(!mobile.raytracing_enabled);
        
        let pc = QualitySettings::handheld_pc();
        assert_eq!(pc.tier, PerformanceTier::HandheldPC);
        assert!(pc.raytracing_enabled);
    }

    #[test]
    fn test_renderer_creation() {
        let renderer = Renderer::new(RendererConfig::default()).unwrap();
        assert_eq!(renderer.frame_number(), 0);
    }

    #[test]
    fn test_frame_progression() {
        let mut renderer = Renderer::new(RendererConfig::default()).unwrap();
        
        renderer.begin_frame();
        renderer.end_frame();
        
        assert_eq!(renderer.frame_number(), 1);
    }
}
