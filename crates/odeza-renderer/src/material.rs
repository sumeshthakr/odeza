//! Material System
//!
//! PBR-based material system with LOD support.

use glam::{Vec3, Vec4};
use std::collections::HashMap;

/// Material property types
#[derive(Debug, Clone)]
pub enum MaterialProperty {
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Texture(TextureBinding),
    Bool(bool),
}

/// Texture binding for materials
#[derive(Debug, Clone)]
pub struct TextureBinding {
    /// Texture asset ID
    pub texture_id: u64,
    /// UV channel
    pub uv_channel: u32,
    /// Tiling
    pub tiling: [f32; 2],
    /// Offset
    pub offset: [f32; 2],
}

impl Default for TextureBinding {
    fn default() -> Self {
        Self {
            texture_id: 0,
            uv_channel: 0,
            tiling: [1.0, 1.0],
            offset: [0.0, 0.0],
        }
    }
}

/// PBR material parameters
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    /// Base color (albedo)
    pub base_color: Vec4,
    /// Base color texture
    pub base_color_texture: Option<TextureBinding>,
    
    /// Metallic factor
    pub metallic: f32,
    /// Roughness factor
    pub roughness: f32,
    /// Metallic-roughness texture
    pub metallic_roughness_texture: Option<TextureBinding>,
    
    /// Normal map
    pub normal_texture: Option<TextureBinding>,
    /// Normal map scale
    pub normal_scale: f32,
    
    /// Occlusion texture
    pub occlusion_texture: Option<TextureBinding>,
    /// Occlusion strength
    pub occlusion_strength: f32,
    
    /// Emissive color
    pub emissive: Vec3,
    /// Emissive texture
    pub emissive_texture: Option<TextureBinding>,
    
    /// Alpha mode
    pub alpha_mode: AlphaMode,
    /// Alpha cutoff (for masked mode)
    pub alpha_cutoff: f32,
    
    /// Double-sided rendering
    pub double_sided: bool,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            metallic_roughness_texture: None,
            normal_texture: None,
            normal_scale: 1.0,
            occlusion_texture: None,
            occlusion_strength: 1.0,
            emissive: Vec3::ZERO,
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
        }
    }
}

/// Alpha blending modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask,
    Blend,
}

/// Material definition
#[derive(Debug, Clone)]
pub struct Material {
    /// Material name
    pub name: String,
    /// Material ID
    pub id: u64,
    /// Shader type
    pub shader: ShaderType,
    /// Material properties
    pub properties: HashMap<String, MaterialProperty>,
    /// Render queue priority
    pub render_queue: i32,
    /// Material complexity (for LOD/budget)
    pub complexity: MaterialComplexity,
}

impl Material {
    /// Create a new material
    pub fn new(name: impl Into<String>, shader: ShaderType) -> Self {
        Self {
            name: name.into(),
            id: 0,
            shader,
            properties: HashMap::new(),
            render_queue: 0,
            complexity: MaterialComplexity::Medium,
        }
    }

    /// Set a property
    pub fn set_property(&mut self, name: impl Into<String>, value: MaterialProperty) {
        self.properties.insert(name.into(), value);
    }

    /// Get a property
    pub fn get_property(&self, name: &str) -> Option<&MaterialProperty> {
        self.properties.get(name)
    }

    /// Create a PBR material
    pub fn pbr(name: impl Into<String>, params: PbrMaterial) -> Self {
        let mut mat = Self::new(name, ShaderType::Pbr);
        
        mat.set_property("base_color", MaterialProperty::Vec4(params.base_color.into()));
        mat.set_property("metallic", MaterialProperty::Float(params.metallic));
        mat.set_property("roughness", MaterialProperty::Float(params.roughness));
        mat.set_property("normal_scale", MaterialProperty::Float(params.normal_scale));
        mat.set_property("emissive", MaterialProperty::Vec3(params.emissive.into()));
        mat.set_property("alpha_cutoff", MaterialProperty::Float(params.alpha_cutoff));
        mat.set_property("double_sided", MaterialProperty::Bool(params.double_sided));
        
        if let Some(tex) = params.base_color_texture {
            mat.set_property("base_color_texture", MaterialProperty::Texture(tex));
        }
        if let Some(tex) = params.metallic_roughness_texture {
            mat.set_property("metallic_roughness_texture", MaterialProperty::Texture(tex));
        }
        if let Some(tex) = params.normal_texture {
            mat.set_property("normal_texture", MaterialProperty::Texture(tex));
        }
        
        mat
    }
}

/// Shader types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderType {
    /// Standard PBR shader
    Pbr,
    /// Unlit shader
    Unlit,
    /// Custom shader
    Custom(u32),
}

/// Material complexity for performance budgeting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialComplexity {
    /// Simple material (few textures, basic math)
    Low,
    /// Standard material
    Medium,
    /// Complex material (many textures, complex math)
    High,
    /// Very complex (procedural, many layers)
    VeryHigh,
}

impl MaterialComplexity {
    /// Get the cost multiplier
    pub fn cost_multiplier(&self) -> f32 {
        match self {
            Self::Low => 0.5,
            Self::Medium => 1.0,
            Self::High => 2.0,
            Self::VeryHigh => 4.0,
        }
    }
}

/// Material instance with per-instance overrides
#[derive(Debug, Clone)]
pub struct MaterialInstance {
    /// Base material ID
    pub base_material: u64,
    /// Instance-specific property overrides
    pub overrides: HashMap<String, MaterialProperty>,
}

impl MaterialInstance {
    /// Create a new instance from a base material
    pub fn new(base_material: u64) -> Self {
        Self {
            base_material,
            overrides: HashMap::new(),
        }
    }

    /// Override a property
    pub fn set_override(&mut self, name: impl Into<String>, value: MaterialProperty) {
        self.overrides.insert(name.into(), value);
    }

    /// Get an override
    pub fn get_override(&self, name: &str) -> Option<&MaterialProperty> {
        self.overrides.get(name)
    }

    /// Clear all overrides
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbr_material_default() {
        let pbr = PbrMaterial::default();
        assert_eq!(pbr.base_color, Vec4::ONE);
        assert_eq!(pbr.metallic, 0.0);
        assert_eq!(pbr.roughness, 0.5);
    }

    #[test]
    fn test_material_creation() {
        let mat = Material::new("TestMat", ShaderType::Pbr);
        assert_eq!(mat.name, "TestMat");
        assert_eq!(mat.shader, ShaderType::Pbr);
    }

    #[test]
    fn test_material_properties() {
        let mut mat = Material::new("TestMat", ShaderType::Pbr);
        mat.set_property("roughness", MaterialProperty::Float(0.8));
        
        let prop = mat.get_property("roughness").unwrap();
        match prop {
            MaterialProperty::Float(v) => assert_eq!(*v, 0.8),
            _ => panic!("Wrong property type"),
        }
    }

    #[test]
    fn test_material_instance() {
        let mut instance = MaterialInstance::new(42);
        instance.set_override("roughness", MaterialProperty::Float(0.2));
        
        assert!(instance.get_override("roughness").is_some());
        assert!(instance.get_override("metallic").is_none());
    }
}
