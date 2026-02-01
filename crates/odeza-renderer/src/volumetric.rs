//! Volumetric Lighting and Fog
//!
//! Froxel-based volumetric effects with temporal reprojection.

use glam::Vec3;

/// Volumetric configuration
#[derive(Debug, Clone)]
pub struct VolumetricConfig {
    /// Enable volumetrics
    pub enabled: bool,
    /// Froxel grid dimensions
    pub grid_size: [u32; 3],
    /// Maximum distance for volumetrics
    pub max_distance: f32,
    /// Temporal reprojection enabled
    pub temporal_reprojection: bool,
    /// Update rate (1 = every frame, 2 = half rate, etc)
    pub update_rate: u32,
    /// Max lights affecting volume
    pub max_volumetric_lights: u32,
}

impl Default for VolumetricConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            grid_size: [160, 90, 64],
            max_distance: 100.0,
            temporal_reprojection: true,
            update_rate: 1,
            max_volumetric_lights: 4,
        }
    }
}

impl VolumetricConfig {
    /// Mobile-optimized settings
    pub fn mobile() -> Self {
        Self {
            enabled: true,
            grid_size: [80, 45, 32],
            max_distance: 50.0,
            temporal_reprojection: true,
            update_rate: 2, // Half rate
            max_volumetric_lights: 2,
        }
    }

    /// High quality settings
    pub fn high_quality() -> Self {
        Self {
            enabled: true,
            grid_size: [320, 180, 128],
            max_distance: 200.0,
            temporal_reprojection: true,
            update_rate: 1,
            max_volumetric_lights: 8,
        }
    }
}

/// Global height fog settings
#[derive(Debug, Clone)]
pub struct HeightFog {
    /// Enable height fog
    pub enabled: bool,
    /// Fog color
    pub color: Vec3,
    /// Fog density
    pub density: f32,
    /// Height falloff
    pub height_falloff: f32,
    /// Start height
    pub start_height: f32,
    /// Maximum opacity
    pub max_opacity: f32,
}

impl Default for HeightFog {
    fn default() -> Self {
        Self {
            enabled: true,
            color: Vec3::new(0.5, 0.6, 0.7),
            density: 0.02,
            height_falloff: 0.2,
            start_height: 0.0,
            max_opacity: 1.0,
        }
    }
}

/// Local volumetric volume (box or sphere)
#[derive(Debug, Clone)]
pub struct VolumetricVolume {
    /// Volume type
    pub volume_type: VolumeType,
    /// Volume color/scattering
    pub color: Vec3,
    /// Density
    pub density: f32,
    /// Blend mode
    pub blend_mode: VolumeBlendMode,
    /// Priority (higher = processed later)
    pub priority: i32,
}

/// Volume shape types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    Box,
    Sphere,
    Capsule,
}

/// How volumes blend with global fog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VolumeBlendMode {
    #[default]
    Additive,
    Override,
    Multiply,
}

impl Default for VolumetricVolume {
    fn default() -> Self {
        Self {
            volume_type: VolumeType::Box,
            color: Vec3::ONE,
            density: 0.1,
            blend_mode: VolumeBlendMode::Additive,
            priority: 0,
        }
    }
}

/// Volumetric shadow quality
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumetricShadowQuality {
    Off,
    Low,
    Medium,
    High,
}

impl VolumetricShadowQuality {
    /// Get shadow ray count
    pub fn ray_count(&self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Low => 4,
            Self::Medium => 8,
            Self::High => 16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volumetric_config() {
        let config = VolumetricConfig::default();
        assert!(config.enabled);
        assert!(config.temporal_reprojection);
    }

    #[test]
    fn test_mobile_volumetrics() {
        let config = VolumetricConfig::mobile();
        assert_eq!(config.update_rate, 2);
        assert_eq!(config.max_volumetric_lights, 2);
    }

    #[test]
    fn test_height_fog() {
        let fog = HeightFog::default();
        assert!(fog.enabled);
        assert!(fog.density > 0.0);
    }
}
