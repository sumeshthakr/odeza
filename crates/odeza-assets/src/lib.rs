//! # Odeza Assets
//!
//! Asset pipeline and database for the Odeza engine.
//!
//! ## Features
//! - Content-addressed asset IDs (hash-based)
//! - Dependency graph tracking
//! - Incremental cooking
//! - Platform-specific asset compilation

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ahash::AHashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Asset errors
#[derive(Error, Debug)]
pub enum AssetError {
    #[error("Asset not found: {0}")]
    NotFound(String),
    
    #[error("Import failed: {0}")]
    ImportFailed(String),
    
    #[error("Cook failed: {0}")]
    CookFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Result type for asset operations
pub type AssetResult<T> = Result<T, AssetError>;

/// Content-addressed asset ID (hash-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub u64);

impl AssetId {
    /// Create an asset ID from a hash
    pub fn from_hash(hash: u64) -> Self {
        Self(hash)
    }

    /// Create an asset ID from content bytes
    pub fn from_content(content: &[u8]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Create an asset ID from a path
    pub fn from_path(path: &Path) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Asset type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Texture,
    Mesh,
    Animation,
    Audio,
    Material,
    Shader,
    Prefab,
    Scene,
    Font,
    Script,
    Data,
}

impl AssetType {
    /// Get file extensions for this asset type
    pub fn extensions(&self) -> &[&str] {
        match self {
            Self::Texture => &["png", "jpg", "jpeg", "tga", "exr", "hdr"],
            Self::Mesh => &["gltf", "glb", "fbx", "obj"],
            Self::Animation => &["gltf", "glb", "fbx"],
            Self::Audio => &["wav", "ogg", "mp3", "flac"],
            Self::Material => &["omat"],
            Self::Shader => &["wgsl", "glsl", "hlsl"],
            Self::Prefab => &["opfb"],
            Self::Scene => &["oscn"],
            Self::Font => &["ttf", "otf"],
            Self::Script => &["wasm", "lua"],
            Self::Data => &["json", "toml", "yaml"],
        }
    }
}

/// Asset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    /// Asset ID
    pub id: AssetId,
    /// Asset name
    pub name: String,
    /// Asset type
    pub asset_type: AssetType,
    /// Source file path (relative to project)
    pub source_path: PathBuf,
    /// Content hash for change detection
    pub content_hash: u64,
    /// Dependencies (other asset IDs)
    pub dependencies: Vec<AssetId>,
    /// Last modification time
    pub modified_time: u64,
    /// Import settings hash
    pub settings_hash: u64,
}

/// Import settings for textures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureImportSettings {
    /// Generate mipmaps
    pub generate_mips: bool,
    /// Compression format
    pub compression: TextureCompression,
    /// sRGB color space
    pub srgb: bool,
    /// Max resolution (0 = unlimited)
    pub max_size: u32,
}

impl Default for TextureImportSettings {
    fn default() -> Self {
        Self {
            generate_mips: true,
            compression: TextureCompression::Auto,
            srgb: true,
            max_size: 4096,
        }
    }
}

/// Texture compression formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextureCompression {
    #[default]
    Auto,
    None,
    BC7,
    ASTC,
    ETC2,
}

/// Import settings for meshes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshImportSettings {
    /// Import scale
    pub scale: f32,
    /// Generate LODs
    pub generate_lods: bool,
    /// Number of LOD levels
    pub lod_count: u32,
    /// Calculate tangents
    pub calculate_tangents: bool,
    /// Optimize mesh
    pub optimize: bool,
}

impl Default for MeshImportSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            generate_lods: true,
            lod_count: 4,
            calculate_tangents: true,
            optimize: true,
        }
    }
}

/// Asset database for tracking all assets
pub struct AssetDatabase {
    /// All registered assets
    assets: RwLock<AHashMap<AssetId, AssetMeta>>,
    /// Path to ID mapping
    path_to_id: RwLock<AHashMap<PathBuf, AssetId>>,
    /// Dependency graph (asset -> dependents)
    dependents: RwLock<AHashMap<AssetId, HashSet<AssetId>>>,
    /// Project root path
    project_root: PathBuf,
    /// Cache directory
    cache_dir: PathBuf,
}

impl AssetDatabase {
    /// Create a new asset database
    pub fn new(project_root: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            assets: RwLock::new(AHashMap::new()),
            path_to_id: RwLock::new(AHashMap::new()),
            dependents: RwLock::new(AHashMap::new()),
            project_root,
            cache_dir,
        }
    }

    /// Get the project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Register an asset
    pub fn register(&self, meta: AssetMeta) {
        let id = meta.id;
        let path = meta.source_path.clone();
        let deps = meta.dependencies.clone();

        self.assets.write().insert(id, meta);
        self.path_to_id.write().insert(path, id);

        // Update dependency graph
        let mut dependents = self.dependents.write();
        for dep_id in deps {
            dependents.entry(dep_id).or_default().insert(id);
        }
    }

    /// Get asset metadata by ID
    pub fn get(&self, id: AssetId) -> Option<AssetMeta> {
        self.assets.read().get(&id).cloned()
    }

    /// Get asset ID by path
    pub fn get_id_by_path(&self, path: &Path) -> Option<AssetId> {
        self.path_to_id.read().get(path).copied()
    }

    /// Get all assets of a type
    pub fn get_by_type(&self, asset_type: AssetType) -> Vec<AssetMeta> {
        self.assets
            .read()
            .values()
            .filter(|m| m.asset_type == asset_type)
            .cloned()
            .collect()
    }

    /// Get assets that depend on the given asset
    pub fn get_dependents(&self, id: AssetId) -> Vec<AssetId> {
        self.dependents
            .read()
            .get(&id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if an asset needs recooking
    pub fn needs_recook(&self, id: AssetId) -> bool {
        let assets = self.assets.read();
        let Some(meta) = assets.get(&id) else {
            return true;
        };

        // Check if source file changed
        let source_path = self.project_root.join(&meta.source_path);
        if let Ok(file_meta) = std::fs::metadata(&source_path) {
            if let Ok(modified) = file_meta.modified() {
                let modified_time = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if modified_time > meta.modified_time {
                    return true;
                }
            }
        }

        // Check if any dependency changed
        for dep_id in &meta.dependencies {
            if self.needs_recook(*dep_id) {
                return true;
            }
        }

        false
    }

    /// Get total asset count
    pub fn asset_count(&self) -> usize {
        self.assets.read().len()
    }

    /// Clear the database
    pub fn clear(&self) {
        self.assets.write().clear();
        self.path_to_id.write().clear();
        self.dependents.write().clear();
    }
}

/// Asset cooker for platform-specific compilation
pub struct AssetCooker {
    /// Target platform
    pub platform: TargetPlatform,
    /// Output directory
    pub output_dir: PathBuf,
    /// Compression enabled
    pub compress: bool,
}

/// Target platforms for cooking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    Android,
    Ios,
    Windows,
    Linux,
}

impl TargetPlatform {
    /// Get preferred texture compression for platform
    pub fn texture_compression(&self) -> TextureCompression {
        match self {
            Self::Android => TextureCompression::ASTC,
            Self::Ios => TextureCompression::ASTC,
            Self::Windows => TextureCompression::BC7,
            Self::Linux => TextureCompression::BC7,
        }
    }
}

impl AssetCooker {
    /// Create a new asset cooker
    pub fn new(platform: TargetPlatform, output_dir: PathBuf) -> Self {
        Self {
            platform,
            output_dir,
            compress: true,
        }
    }

    /// Cook an asset
    pub fn cook(&self, _meta: &AssetMeta, _source_data: &[u8]) -> AssetResult<Vec<u8>> {
        // Placeholder - would do actual cooking here
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_id() {
        let id1 = AssetId::from_content(b"test content");
        let id2 = AssetId::from_content(b"test content");
        let id3 = AssetId::from_content(b"different content");
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_asset_database() {
        let db = AssetDatabase::new(PathBuf::from("."), PathBuf::from("./cache"));
        
        let meta = AssetMeta {
            id: AssetId(12345),
            name: String::from("Test"),
            asset_type: AssetType::Texture,
            source_path: PathBuf::from("textures/test.png"),
            content_hash: 0,
            dependencies: Vec::new(),
            modified_time: 0,
            settings_hash: 0,
        };
        
        db.register(meta.clone());
        
        assert_eq!(db.asset_count(), 1);
        
        let retrieved = db.get(AssetId(12345)).unwrap();
        assert_eq!(retrieved.name, "Test");
    }

    #[test]
    fn test_path_to_id() {
        let db = AssetDatabase::new(PathBuf::from("."), PathBuf::from("./cache"));
        
        let path = PathBuf::from("textures/test.png");
        let meta = AssetMeta {
            id: AssetId(99999),
            name: String::from("Test"),
            asset_type: AssetType::Texture,
            source_path: path.clone(),
            content_hash: 0,
            dependencies: Vec::new(),
            modified_time: 0,
            settings_hash: 0,
        };
        
        db.register(meta);
        
        let id = db.get_id_by_path(&path).unwrap();
        assert_eq!(id, AssetId(99999));
    }

    #[test]
    fn test_target_platform() {
        assert_eq!(TargetPlatform::Android.texture_compression(), TextureCompression::ASTC);
        assert_eq!(TargetPlatform::Windows.texture_compression(), TextureCompression::BC7);
    }
}
