//! Lighting System
//!
//! Clustered forward (Forward+) lighting with tiered quality.

use glam::{Vec3, Mat4};
use smallvec::SmallVec;

/// Light types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightType {
    /// Directional light (sun)
    Directional,
    /// Point light
    Point,
    /// Spot light
    Spot,
    /// Area light (for high-end)
    Area,
}

/// Light shadow settings
#[derive(Debug, Clone)]
pub struct ShadowSettings {
    /// Enable shadows
    pub enabled: bool,
    /// Shadow map resolution
    pub resolution: u32,
    /// Shadow bias
    pub bias: f32,
    /// Normal bias
    pub normal_bias: f32,
    /// Near plane
    pub near: f32,
    /// Far plane
    pub far: f32,
    /// Number of cascades (for directional)
    pub cascades: u32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            resolution: 2048,
            bias: 0.001,
            normal_bias: 0.01,
            near: 0.1,
            far: 100.0,
            cascades: 4,
        }
    }
}

/// Light component
#[derive(Debug, Clone)]
pub struct Light {
    /// Light type
    pub light_type: LightType,
    /// Light color
    pub color: Vec3,
    /// Light intensity
    pub intensity: f32,
    /// Range (for point/spot)
    pub range: f32,
    /// Inner cone angle in radians (for spot)
    pub inner_cone_angle: f32,
    /// Outer cone angle in radians (for spot)
    pub outer_cone_angle: f32,
    /// Shadow settings
    pub shadows: ShadowSettings,
    /// Whether the light affects volumetrics
    pub volumetric: bool,
    /// Light layer mask
    pub layer_mask: u32,
}

impl Light {
    /// Create a directional light
    pub fn directional(color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional,
            color,
            intensity,
            range: f32::INFINITY,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            shadows: ShadowSettings::default(),
            volumetric: true,
            layer_mask: u32::MAX,
        }
    }

    /// Create a point light
    pub fn point(color: Vec3, intensity: f32, range: f32) -> Self {
        Self {
            light_type: LightType::Point,
            color,
            intensity,
            range,
            inner_cone_angle: 0.0,
            outer_cone_angle: 0.0,
            shadows: ShadowSettings {
                enabled: false,
                resolution: 512,
                ..Default::default()
            },
            volumetric: false,
            layer_mask: u32::MAX,
        }
    }

    /// Create a spot light
    pub fn spot(color: Vec3, intensity: f32, range: f32, inner_angle: f32, outer_angle: f32) -> Self {
        Self {
            light_type: LightType::Spot,
            color,
            intensity,
            range,
            inner_cone_angle: inner_angle,
            outer_cone_angle: outer_angle,
            shadows: ShadowSettings {
                enabled: false,
                resolution: 1024,
                ..Default::default()
            },
            volumetric: false,
            layer_mask: u32::MAX,
        }
    }

    /// Get the effective color (color * intensity)
    pub fn effective_color(&self) -> Vec3 {
        self.color * self.intensity
    }
}

impl Default for Light {
    fn default() -> Self {
        Self::point(Vec3::ONE, 1.0, 10.0)
    }
}

/// Light cluster for Forward+ rendering
#[derive(Debug, Clone, Default)]
pub struct LightCluster {
    /// Indices of lights affecting this cluster
    pub light_indices: SmallVec<[u16; 32]>,
}

/// Lighting system configuration
#[derive(Debug, Clone)]
pub struct LightingConfig {
    /// Maximum lights per cluster
    pub max_lights_per_cluster: u32,
    /// Maximum volumetric lights
    pub max_volumetric_lights: u32,
    /// Cluster grid dimensions
    pub cluster_grid: [u32; 3],
    /// Enable clustered lighting
    pub clustered: bool,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            max_lights_per_cluster: 32,
            max_volumetric_lights: 4,
            cluster_grid: [16, 8, 24],
            clustered: true,
        }
    }
}

/// Lighting system
pub struct LightingSystem {
    config: LightingConfig,
    /// Light data buffer
    lights: Vec<LightData>,
    /// Cluster grid
    clusters: Vec<LightCluster>,
    /// Directional light (sun)
    sun: Option<Light>,
    /// Ambient color
    ambient_color: Vec3,
    /// Ambient intensity
    ambient_intensity: f32,
}

/// GPU-ready light data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct LightData {
    pub position: [f32; 4],
    pub direction: [f32; 4],
    pub color: [f32; 4],
    pub params: [f32; 4], // range, inner_angle, outer_angle, type
}

impl LightingSystem {
    /// Create a new lighting system
    pub fn new(config: LightingConfig) -> Self {
        let cluster_count = (config.cluster_grid[0] * config.cluster_grid[1] * config.cluster_grid[2]) as usize;
        
        Self {
            config,
            lights: Vec::new(),
            clusters: vec![LightCluster::default(); cluster_count],
            sun: None,
            ambient_color: Vec3::new(0.1, 0.1, 0.15),
            ambient_intensity: 1.0,
        }
    }

    /// Set the sun (main directional light)
    pub fn set_sun(&mut self, light: Light) {
        assert_eq!(light.light_type, LightType::Directional);
        self.sun = Some(light);
    }

    /// Get the sun
    pub fn sun(&self) -> Option<&Light> {
        self.sun.as_ref()
    }

    /// Set ambient lighting
    pub fn set_ambient(&mut self, color: Vec3, intensity: f32) {
        self.ambient_color = color;
        self.ambient_intensity = intensity;
    }

    /// Get ambient color with intensity
    pub fn ambient(&self) -> Vec3 {
        self.ambient_color * self.ambient_intensity
    }

    /// Get the configuration
    pub fn config(&self) -> &LightingConfig {
        &self.config
    }

    /// Get the number of active lights
    pub fn light_count(&self) -> usize {
        self.lights.len()
    }

    /// Clear all lights for the frame
    pub fn clear(&mut self) {
        self.lights.clear();
        for cluster in &mut self.clusters {
            cluster.light_indices.clear();
        }
    }

    /// Add a light for this frame
    pub fn add_light(&mut self, light: &Light, position: Vec3, direction: Vec3) {
        if self.lights.len() >= 1024 {
            return; // Hard limit
        }

        let light_data = LightData {
            position: [position.x, position.y, position.z, 1.0],
            direction: [direction.x, direction.y, direction.z, 0.0],
            color: [light.color.x * light.intensity, light.color.y * light.intensity, light.color.z * light.intensity, 1.0],
            params: [
                light.range,
                light.inner_cone_angle,
                light.outer_cone_angle,
                light.light_type as u32 as f32,
            ],
        };

        self.lights.push(light_data);
    }

    /// Build the light clusters
    pub fn build_clusters(&mut self, _view_proj: Mat4) {
        // Clear existing cluster data
        for cluster in &mut self.clusters {
            cluster.light_indices.clear();
        }

        // Simple implementation - would do frustum/AABB intersection in production
        for (light_idx, _light) in self.lights.iter().enumerate() {
            // Assign to relevant clusters based on position and range
            // This is a placeholder - real implementation would do proper culling
            let light_idx = light_idx as u16;
            if light_idx < u16::MAX {
                for cluster in &mut self.clusters {
                    if cluster.light_indices.len() < self.config.max_lights_per_cluster as usize {
                        cluster.light_indices.push(light_idx);
                    }
                }
            }
        }
    }
}

impl Default for LightingSystem {
    fn default() -> Self {
        Self::new(LightingConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_light_creation() {
        let sun = Light::directional(Vec3::ONE, 10.0);
        assert_eq!(sun.light_type, LightType::Directional);
        assert_eq!(sun.intensity, 10.0);
        
        let point = Light::point(Vec3::new(1.0, 0.5, 0.0), 5.0, 20.0);
        assert_eq!(point.light_type, LightType::Point);
        assert_eq!(point.range, 20.0);
    }

    #[test]
    fn test_effective_color() {
        let light = Light::point(Vec3::new(1.0, 0.0, 0.0), 2.0, 10.0);
        let effective = light.effective_color();
        assert_eq!(effective, Vec3::new(2.0, 0.0, 0.0));
    }

    #[test]
    fn test_lighting_system() {
        let mut system = LightingSystem::new(LightingConfig::default());
        
        system.set_sun(Light::directional(Vec3::ONE, 5.0));
        assert!(system.sun().is_some());
        
        system.set_ambient(Vec3::new(0.1, 0.1, 0.2), 1.0);
        assert_eq!(system.ambient(), Vec3::new(0.1, 0.1, 0.2));
    }

    #[test]
    fn test_light_addition() {
        let mut system = LightingSystem::new(LightingConfig::default());
        
        let light = Light::point(Vec3::ONE, 5.0, 10.0);
        system.add_light(&light, Vec3::new(1.0, 2.0, 3.0), Vec3::NEG_Z);
        
        assert_eq!(system.light_count(), 1);
        
        system.clear();
        assert_eq!(system.light_count(), 0);
    }
}
