//! Post-Processing Effects
//!
//! Temporal anti-aliasing, bloom, tone mapping, and other effects.

use glam::Vec3;

/// TAA (Temporal Anti-Aliasing) settings
#[derive(Debug, Clone)]
pub struct TaaSettings {
    /// Enable TAA
    pub enabled: bool,
    /// Jitter scale
    pub jitter_scale: f32,
    /// History blend factor
    pub blend_factor: f32,
    /// Motion vector scale
    pub motion_scale: f32,
    /// Rejection threshold
    pub rejection_threshold: f32,
}

impl Default for TaaSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            jitter_scale: 1.0,
            blend_factor: 0.9,
            motion_scale: 1.0,
            rejection_threshold: 0.1,
        }
    }
}

/// Bloom settings
#[derive(Debug, Clone)]
pub struct BloomSettings {
    /// Enable bloom
    pub enabled: bool,
    /// Bloom intensity
    pub intensity: f32,
    /// Bloom threshold
    pub threshold: f32,
    /// Soft knee
    pub soft_knee: f32,
    /// Number of blur passes
    pub blur_passes: u32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.5,
            threshold: 1.0,
            soft_knee: 0.5,
            blur_passes: 5,
        }
    }
}

/// Tone mapping operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneMapper {
    #[default]
    ACES,
    Reinhard,
    Uncharted2,
    AgX,
    Neutral,
}

/// Exposure settings
#[derive(Debug, Clone)]
pub struct ExposureSettings {
    /// Enable auto exposure
    pub auto_exposure: bool,
    /// Manual exposure value (EV)
    pub exposure_ev: f32,
    /// Minimum exposure
    pub min_exposure: f32,
    /// Maximum exposure
    pub max_exposure: f32,
    /// Adaptation speed
    pub adaptation_speed: f32,
}

impl Default for ExposureSettings {
    fn default() -> Self {
        Self {
            auto_exposure: true,
            exposure_ev: 0.0,
            min_exposure: -4.0,
            max_exposure: 4.0,
            adaptation_speed: 1.0,
        }
    }
}

/// Color grading settings
#[derive(Debug, Clone)]
pub struct ColorGradingSettings {
    /// Enable color grading
    pub enabled: bool,
    /// Saturation adjustment
    pub saturation: f32,
    /// Contrast adjustment
    pub contrast: f32,
    /// Color filter (tint)
    pub color_filter: Vec3,
    /// Shadow color
    pub shadows: Vec3,
    /// Midtone color
    pub midtones: Vec3,
    /// Highlight color
    pub highlights: Vec3,
}

impl Default for ColorGradingSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            saturation: 1.0,
            contrast: 1.0,
            color_filter: Vec3::ONE,
            shadows: Vec3::ONE,
            midtones: Vec3::ONE,
            highlights: Vec3::ONE,
        }
    }
}

/// Depth of field settings
#[derive(Debug, Clone)]
pub struct DepthOfFieldSettings {
    /// Enable depth of field
    pub enabled: bool,
    /// Focus distance
    pub focus_distance: f32,
    /// Aperture (f-stop)
    pub aperture: f32,
    /// Max blur radius
    pub max_blur: f32,
    /// Enable bokeh
    pub bokeh: bool,
}

impl Default for DepthOfFieldSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            focus_distance: 10.0,
            aperture: 2.8,
            max_blur: 16.0,
            bokeh: false,
        }
    }
}

/// Complete post-processing stack settings
#[derive(Debug, Clone, Default)]
pub struct PostProcess {
    /// TAA settings
    pub taa: TaaSettings,
    /// Bloom settings
    pub bloom: BloomSettings,
    /// Tone mapper
    pub tone_mapper: ToneMapper,
    /// Exposure settings
    pub exposure: ExposureSettings,
    /// Color grading
    pub color_grading: ColorGradingSettings,
    /// Depth of field
    pub depth_of_field: DepthOfFieldSettings,
    /// Enable vignette
    pub vignette: bool,
    /// Vignette intensity
    pub vignette_intensity: f32,
}

impl PostProcess {
    /// Create mobile-optimized settings
    pub fn mobile() -> Self {
        Self {
            taa: TaaSettings::default(),
            bloom: BloomSettings {
                blur_passes: 3,
                ..Default::default()
            },
            depth_of_field: DepthOfFieldSettings {
                enabled: false,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create high-quality settings
    pub fn high_quality() -> Self {
        Self {
            taa: TaaSettings::default(),
            bloom: BloomSettings {
                blur_passes: 6,
                ..Default::default()
            },
            depth_of_field: DepthOfFieldSettings {
                enabled: true,
                bokeh: true,
                ..Default::default()
            },
            vignette: true,
            vignette_intensity: 0.3,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_process_defaults() {
        let pp = PostProcess::default();
        assert!(pp.taa.enabled);
        assert!(pp.bloom.enabled);
    }

    #[test]
    fn test_mobile_settings() {
        let pp = PostProcess::mobile();
        assert!(!pp.depth_of_field.enabled);
        assert_eq!(pp.bloom.blur_passes, 3);
    }
}
