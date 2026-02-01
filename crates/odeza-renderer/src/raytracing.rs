//! Ray Tracing Support
//!
//! Tiered ray tracing effects with fallback chains.

/// Ray tracing capability level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtCapability {
    /// No ray tracing support
    None,
    /// Basic ray tracing (limited rays)
    Basic,
    /// Full ray tracing support
    Full,
}

/// Ray tracing effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtEffect {
    Reflections,
    Shadows,
    AmbientOcclusion,
    GlobalIllumination,
}

/// Ray tracing configuration
#[derive(Debug, Clone)]
pub struct RtConfig {
    /// Ray tracing capability
    pub capability: RtCapability,
    /// Enable RT reflections
    pub reflections: bool,
    /// Enable RT shadows
    pub shadows: bool,
    /// Enable RT ambient occlusion
    pub ambient_occlusion: bool,
    /// Enable RT global illumination
    pub global_illumination: bool,
    /// Max rays per pixel for reflections
    pub reflection_rays: u32,
    /// Max rays per pixel for shadows
    pub shadow_rays: u32,
    /// Max rays per pixel for AO
    pub ao_rays: u32,
    /// Denoiser strength
    pub denoise_strength: f32,
}

impl Default for RtConfig {
    fn default() -> Self {
        Self {
            capability: RtCapability::None,
            reflections: false,
            shadows: false,
            ambient_occlusion: false,
            global_illumination: false,
            reflection_rays: 1,
            shadow_rays: 1,
            ao_rays: 1,
            denoise_strength: 1.0,
        }
    }
}

impl RtConfig {
    /// Create config for mobile with RT support
    pub fn mobile_rt() -> Self {
        Self {
            capability: RtCapability::Basic,
            reflections: true,
            shadows: false,
            ambient_occlusion: false,
            global_illumination: false,
            reflection_rays: 1,
            shadow_rays: 1,
            ao_rays: 1,
            denoise_strength: 1.0,
        }
    }

    /// Create config for high-end with full RT
    pub fn high_end() -> Self {
        Self {
            capability: RtCapability::Full,
            reflections: true,
            shadows: true,
            ambient_occlusion: true,
            global_illumination: false,
            reflection_rays: 2,
            shadow_rays: 2,
            ao_rays: 4,
            denoise_strength: 0.8,
        }
    }
}

/// Denoiser settings for ray tracing
#[derive(Debug, Clone)]
pub struct DenoiserSettings {
    /// Enable denoiser
    pub enabled: bool,
    /// Temporal stability weight
    pub temporal_weight: f32,
    /// Spatial filter radius
    pub spatial_radius: u32,
    /// History rejection threshold
    pub rejection_threshold: f32,
    /// Roughness-aware filtering
    pub roughness_aware: bool,
}

impl Default for DenoiserSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            temporal_weight: 0.9,
            spatial_radius: 3,
            rejection_threshold: 0.1,
            roughness_aware: true,
        }
    }
}

/// Fallback chain for when RT is not available
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackMode {
    /// Use screen-space technique
    ScreenSpace,
    /// Use precomputed probes
    Probes,
    /// Use shadow maps
    ShadowMaps,
    /// Disabled
    Off,
}

/// Reflection fallback chain
pub fn reflection_fallback(rt_available: bool, ssr_available: bool) -> FallbackMode {
    if rt_available {
        FallbackMode::ScreenSpace // RT handled separately
    } else if ssr_available {
        FallbackMode::ScreenSpace
    } else {
        FallbackMode::Probes
    }
}

/// Shadow fallback chain  
pub fn shadow_fallback(rt_available: bool) -> FallbackMode {
    if rt_available {
        FallbackMode::ShadowMaps // RT handled separately
    } else {
        FallbackMode::ShadowMaps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rt_config_default() {
        let config = RtConfig::default();
        assert_eq!(config.capability, RtCapability::None);
        assert!(!config.reflections);
    }

    #[test]
    fn test_rt_fallbacks() {
        assert_eq!(reflection_fallback(false, true), FallbackMode::ScreenSpace);
        assert_eq!(reflection_fallback(false, false), FallbackMode::Probes);
    }
}
