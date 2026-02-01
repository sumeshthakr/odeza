//! Window Management
//!
//! Cross-platform window creation and event handling.

use std::sync::Arc;

use glam::UVec2;
use parking_lot::RwLock;

use crate::PlatformResult;

/// Window configuration
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Initial width in pixels
    pub width: u32,
    /// Initial height in pixels
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to start in fullscreen mode
    pub fullscreen: bool,
    /// Whether VSync is enabled
    pub vsync: bool,
    /// Target refresh rate (0 for unlimited)
    pub target_fps: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("Odeza Engine"),
            width: 1280,
            height: 720,
            resizable: true,
            fullscreen: false,
            vsync: true,
            target_fps: 60,
        }
    }
}

impl WindowConfig {
    /// Create a mobile-optimized window config
    pub fn mobile() -> Self {
        Self {
            title: String::from("Odeza"),
            width: 0, // Auto-detect
            height: 0,
            resizable: false,
            fullscreen: true,
            vsync: true,
            target_fps: 60,
        }
    }

    /// Create a desktop window config
    pub fn desktop() -> Self {
        Self::default()
    }
}

/// Window events
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Window was resized
    Resized { width: u32, height: u32 },
    /// Window was moved
    Moved { x: i32, y: i32 },
    /// Window close was requested
    CloseRequested,
    /// Window gained focus
    Focused,
    /// Window lost focus
    Unfocused,
    /// Window was minimized
    Minimized,
    /// Window was restored from minimized state
    Restored,
    /// Window scale factor changed (DPI)
    ScaleFactorChanged { scale_factor: f64 },
    /// Redraw was requested
    RedrawRequested,
}

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Current width in physical pixels
    pub width: u32,
    /// Current height in physical pixels
    pub height: u32,
    /// Scale factor (DPI scaling)
    pub scale_factor: f64,
    /// Whether the window is focused
    pub focused: bool,
    /// Whether the window is minimized
    pub minimized: bool,
    /// Whether the window is fullscreen
    pub fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            scale_factor: 1.0,
            focused: true,
            minimized: false,
            fullscreen: false,
        }
    }
}

/// Platform window abstraction
pub struct Window {
    /// Window configuration
    config: WindowConfig,
    /// Current window state
    state: Arc<RwLock<WindowState>>,
    /// Pending events
    events: Arc<RwLock<Vec<WindowEvent>>>,
    /// Whether the window should close
    should_close: Arc<RwLock<bool>>,
}

impl Window {
    /// Create a new window with the given configuration
    pub fn new(config: WindowConfig) -> PlatformResult<Self> {
        let state = WindowState {
            width: config.width,
            height: config.height,
            scale_factor: 1.0,
            focused: true,
            minimized: false,
            fullscreen: config.fullscreen,
        };

        Ok(Self {
            config,
            state: Arc::new(RwLock::new(state)),
            events: Arc::new(RwLock::new(Vec::new())),
            should_close: Arc::new(RwLock::new(false)),
        })
    }

    /// Get the window configuration
    pub fn config(&self) -> &WindowConfig {
        &self.config
    }

    /// Get the current window state
    pub fn state(&self) -> WindowState {
        self.state.read().clone()
    }

    /// Get the window size
    pub fn size(&self) -> UVec2 {
        let state = self.state.read();
        UVec2::new(state.width, state.height)
    }

    /// Get the window aspect ratio
    pub fn aspect_ratio(&self) -> f32 {
        let state = self.state.read();
        state.width as f32 / state.height.max(1) as f32
    }

    /// Check if the window should close
    pub fn should_close(&self) -> bool {
        *self.should_close.read()
    }

    /// Request window close
    pub fn request_close(&self) {
        *self.should_close.write() = true;
    }

    /// Poll for pending window events
    pub fn poll_events(&self) -> Vec<WindowEvent> {
        let mut events = self.events.write();
        std::mem::take(&mut *events)
    }

    /// Push a window event (called by platform layer)
    pub fn push_event(&self, event: WindowEvent) {
        // Update state based on event
        match &event {
            WindowEvent::Resized { width, height } => {
                let mut state = self.state.write();
                state.width = *width;
                state.height = *height;
            }
            WindowEvent::Focused => {
                self.state.write().focused = true;
            }
            WindowEvent::Unfocused => {
                self.state.write().focused = false;
            }
            WindowEvent::Minimized => {
                self.state.write().minimized = true;
            }
            WindowEvent::Restored => {
                self.state.write().minimized = false;
            }
            WindowEvent::ScaleFactorChanged { scale_factor } => {
                self.state.write().scale_factor = *scale_factor;
            }
            WindowEvent::CloseRequested => {
                *self.should_close.write() = true;
            }
            _ => {}
        }

        self.events.write().push(event);
    }

    /// Set fullscreen mode
    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.state.write().fullscreen = fullscreen;
    }

    /// Check if window is focused
    pub fn is_focused(&self) -> bool {
        self.state.read().focused
    }

    /// Check if window is minimized
    pub fn is_minimized(&self) -> bool {
        self.state.read().minimized
    }

    /// Get the scale factor
    pub fn scale_factor(&self) -> f64 {
        self.state.read().scale_factor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.resizable);
    }

    #[test]
    fn test_window_creation() {
        let window = Window::new(WindowConfig::default()).unwrap();
        assert!(!window.should_close());
        assert_eq!(window.size(), UVec2::new(1280, 720));
    }

    #[test]
    fn test_window_events() {
        let window = Window::new(WindowConfig::default()).unwrap();
        
        window.push_event(WindowEvent::Resized { width: 1920, height: 1080 });
        
        let events = window.poll_events();
        assert_eq!(events.len(), 1);
        
        let state = window.state();
        assert_eq!(state.width, 1920);
        assert_eq!(state.height, 1080);
    }

    #[test]
    fn test_window_close() {
        let window = Window::new(WindowConfig::default()).unwrap();
        assert!(!window.should_close());
        
        window.request_close();
        assert!(window.should_close());
    }
}
