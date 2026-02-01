//! # Odeza Editor
//!
//! Editor application for the Odeza engine with Unreal-like workflow.
//!
//! ## Features
//! - Viewport (scene view, game view)
//! - Outliner (scene hierarchy)
//! - Inspector (component editing)
//! - Content Browser
//! - Console/Log
//! - Profiler (CPU/GPU/memory/IO)
//! - Build/Packaging
//! - Material Editor
//! - Animation Graph Editor

use std::path::PathBuf;

use glam::Vec3;

/// Editor pane types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaneType {
    Viewport,
    Outliner,
    Inspector,
    ContentBrowser,
    Console,
    Profiler,
    MaterialEditor,
    AnimationGraph,
    BuildWindow,
}

/// Viewport configuration
#[derive(Debug, Clone)]
pub struct ViewportConfig {
    /// Camera position
    pub camera_position: Vec3,
    /// Camera target
    pub camera_target: Vec3,
    /// Field of view in degrees
    pub fov: f32,
    /// Near clip plane
    pub near: f32,
    /// Far clip plane
    pub far: f32,
    /// Grid visible
    pub show_grid: bool,
    /// Gizmos visible
    pub show_gizmos: bool,
}

impl Default for ViewportConfig {
    fn default() -> Self {
        Self {
            camera_position: Vec3::new(5.0, 5.0, 5.0),
            camera_target: Vec3::ZERO,
            fov: 60.0,
            near: 0.1,
            far: 10000.0,
            show_grid: true,
            show_gizmos: true,
        }
    }
}

/// Gizmo mode for transform manipulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
    Scale,
}

/// Gizmo space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GizmoSpace {
    #[default]
    World,
    Local,
}

/// Debug view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DebugView {
    #[default]
    None,
    Wireframe,
    Overdraw,
    LightingOnly,
    RayTracingDebug,
    VirtualTextureTiles,
    StreamingCells,
}

/// Editor project
#[derive(Debug, Clone)]
pub struct EditorProject {
    pub name: String,
    pub path: PathBuf,
    pub target_platforms: Vec<TargetPlatform>,
}

/// Target platforms for build
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    Android,
    Ios,
    Windows,
    Linux,
}

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub platform: TargetPlatform,
    pub configuration: BuildConfiguration,
    pub output_path: PathBuf,
    pub compress_assets: bool,
    pub strip_debug: bool,
}

/// Build configuration type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildConfiguration {
    Debug,
    #[default]
    Development,
    Shipping,
}

/// Editor state
pub struct Editor {
    pub project: Option<EditorProject>,
    pub viewport_config: ViewportConfig,
    pub gizmo_mode: GizmoMode,
    pub gizmo_space: GizmoSpace,
    pub debug_view: DebugView,
    pub selected_entities: Vec<u64>,
    pub is_playing: bool,
    pub is_paused: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            project: None,
            viewport_config: ViewportConfig::default(),
            gizmo_mode: GizmoMode::Translate,
            gizmo_space: GizmoSpace::World,
            debug_view: DebugView::None,
            selected_entities: Vec::new(),
            is_playing: false,
            is_paused: false,
        }
    }

    /// Open a project
    pub fn open_project(&mut self, path: PathBuf) -> anyhow::Result<()> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unnamed")
            .to_string();

        self.project = Some(EditorProject {
            name,
            path,
            target_platforms: vec![
                TargetPlatform::Android,
                TargetPlatform::Ios,
                TargetPlatform::Windows,
            ],
        });

        Ok(())
    }

    /// Start play mode
    pub fn play(&mut self) {
        self.is_playing = true;
        self.is_paused = false;
    }

    /// Pause play mode
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Stop play mode
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
    }

    /// Step one frame in paused mode
    pub fn step(&mut self) {
        if self.is_paused {
            // Step one frame
        }
    }

    /// Select an entity
    pub fn select(&mut self, entity_id: u64) {
        self.selected_entities.clear();
        self.selected_entities.push(entity_id);
    }

    /// Add to selection
    pub fn add_to_selection(&mut self, entity_id: u64) {
        if !self.selected_entities.contains(&entity_id) {
            self.selected_entities.push(entity_id);
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = Editor::new();
        assert!(!editor.is_playing);
        assert!(editor.selected_entities.is_empty());
    }

    #[test]
    fn test_play_mode() {
        let mut editor = Editor::new();
        
        editor.play();
        assert!(editor.is_playing);
        assert!(!editor.is_paused);
        
        editor.pause();
        assert!(editor.is_paused);
        
        editor.stop();
        assert!(!editor.is_playing);
    }

    #[test]
    fn test_selection() {
        let mut editor = Editor::new();
        
        editor.select(1);
        assert_eq!(editor.selected_entities.len(), 1);
        
        editor.add_to_selection(2);
        assert_eq!(editor.selected_entities.len(), 2);
        
        editor.clear_selection();
        assert!(editor.selected_entities.is_empty());
    }
}
