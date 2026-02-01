//! Mesh and Geometry
//!
//! Mesh representation with LOD support.

/// Mesh LOD level
#[derive(Debug, Clone)]
pub struct MeshLod {
    /// LOD index
    pub level: u32,
    /// Screen size threshold
    pub screen_size: f32,
    /// Vertex count
    pub vertex_count: u32,
    /// Triangle count
    pub triangle_count: u32,
}

/// Mesh data
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Mesh name
    pub name: String,
    /// LOD levels
    pub lods: Vec<MeshLod>,
    /// Bounding box min
    pub bounds_min: [f32; 3],
    /// Bounding box max
    pub bounds_max: [f32; 3],
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            name: String::new(),
            lods: Vec::new(),
            bounds_min: [0.0; 3],
            bounds_max: [0.0; 3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_default() {
        let mesh = Mesh::default();
        assert!(mesh.lods.is_empty());
    }
}
