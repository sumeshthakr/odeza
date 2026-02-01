//! Input Handling
//!
//! Cross-platform input handling for touch, gamepad, mouse, and keyboard.

use std::collections::{HashMap, HashSet};

use bitflags::bitflags;
use glam::Vec2;

/// Input event types
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Keyboard key pressed
    KeyPressed(KeyCode),
    /// Keyboard key released
    KeyReleased(KeyCode),
    /// Mouse button pressed
    MousePressed(MouseButton),
    /// Mouse button released
    MouseReleased(MouseButton),
    /// Mouse moved
    MouseMoved { x: f32, y: f32 },
    /// Mouse wheel scrolled
    MouseWheel { delta_x: f32, delta_y: f32 },
    /// Touch started
    TouchStarted { id: u64, x: f32, y: f32 },
    /// Touch moved
    TouchMoved { id: u64, x: f32, y: f32 },
    /// Touch ended
    TouchEnded { id: u64, x: f32, y: f32 },
    /// Touch cancelled
    TouchCancelled { id: u64 },
    /// Gamepad connected
    GamepadConnected { id: u32 },
    /// Gamepad disconnected
    GamepadDisconnected { id: u32 },
    /// Gamepad button pressed
    GamepadButtonPressed { id: u32, button: GamepadButton },
    /// Gamepad button released
    GamepadButtonReleased { id: u32, button: GamepadButton },
    /// Gamepad axis moved
    GamepadAxisMoved { id: u32, axis: GamepadAxis, value: f32 },
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Special keys
    Space, Enter, Escape, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown,
    
    // Arrow keys
    Left, Right, Up, Down,
    
    // Modifier keys
    LeftShift, RightShift,
    LeftControl, RightControl,
    LeftAlt, RightAlt,
    
    // Other
    Unknown,
}

/// Mouse buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Button4,
    Button5,
}

/// Gamepad buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    // Face buttons
    South,      // A / Cross
    East,       // B / Circle
    West,       // X / Square
    North,      // Y / Triangle
    
    // Shoulder buttons
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    
    // Special
    Select,     // Back / Share
    Start,      // Start / Options
    LeftStick,  // L3
    RightStick, // R3
    
    // D-Pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

/// Gamepad axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

bitflags! {
    /// Keyboard modifiers
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Modifiers: u8 {
        const SHIFT = 0b0001;
        const CONTROL = 0b0010;
        const ALT = 0b0100;
        const SUPER = 0b1000;
    }
}

/// Touch point information
#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    /// Touch ID
    pub id: u64,
    /// Position in screen coordinates
    pub position: Vec2,
    /// Previous position
    pub previous_position: Vec2,
    /// Touch phase
    pub phase: TouchPhase,
}

/// Touch phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Touch state for multi-touch handling
#[derive(Debug, Clone, Default)]
pub struct TouchState {
    /// Active touch points
    points: HashMap<u64, TouchPoint>,
}

impl TouchState {
    /// Create a new touch state
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all active touch points
    pub fn points(&self) -> impl Iterator<Item = &TouchPoint> {
        self.points.values()
    }

    /// Get a specific touch point
    pub fn get_point(&self, id: u64) -> Option<&TouchPoint> {
        self.points.get(&id)
    }

    /// Get the number of active touches
    pub fn touch_count(&self) -> usize {
        self.points.len()
    }

    /// Check if there are any active touches
    pub fn is_touching(&self) -> bool {
        !self.points.is_empty()
    }

    /// Handle a touch event
    pub fn handle_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::TouchStarted { id, x, y } => {
                let pos = Vec2::new(*x, *y);
                self.points.insert(*id, TouchPoint {
                    id: *id,
                    position: pos,
                    previous_position: pos,
                    phase: TouchPhase::Started,
                });
            }
            InputEvent::TouchMoved { id, x, y } => {
                if let Some(point) = self.points.get_mut(id) {
                    point.previous_position = point.position;
                    point.position = Vec2::new(*x, *y);
                    point.phase = TouchPhase::Moved;
                }
            }
            InputEvent::TouchEnded { id, x, y } => {
                if let Some(point) = self.points.get_mut(id) {
                    point.previous_position = point.position;
                    point.position = Vec2::new(*x, *y);
                    point.phase = TouchPhase::Ended;
                }
                // Note: Don't remove yet, let the game handle the ended state first
            }
            InputEvent::TouchCancelled { id } => {
                if let Some(point) = self.points.get_mut(id) {
                    point.phase = TouchPhase::Cancelled;
                }
            }
            _ => {}
        }
    }

    /// Clear ended/cancelled touches (call after processing)
    pub fn clear_ended(&mut self) {
        self.points.retain(|_, point| {
            point.phase != TouchPhase::Ended && point.phase != TouchPhase::Cancelled
        });
    }

    /// Calculate pinch scale between first two touch points
    pub fn pinch_scale(&self) -> Option<f32> {
        let points: Vec<_> = self.points.values().take(2).collect();
        if points.len() < 2 {
            return None;
        }

        let current_dist = (points[0].position - points[1].position).length();
        let previous_dist = (points[0].previous_position - points[1].previous_position).length();

        if previous_dist > 0.0 {
            Some(current_dist / previous_dist)
        } else {
            None
        }
    }
}

/// Gamepad state
#[derive(Debug, Clone)]
pub struct GamepadState {
    /// Gamepad ID
    pub id: u32,
    /// Whether the gamepad is connected
    pub connected: bool,
    /// Pressed buttons
    buttons: HashSet<GamepadButton>,
    /// Axis values
    axes: HashMap<GamepadAxis, f32>,
    /// Deadzone for analog sticks
    pub deadzone: f32,
}

impl GamepadState {
    /// Create a new gamepad state
    pub fn new(id: u32) -> Self {
        Self {
            id,
            connected: false,
            buttons: HashSet::new(),
            axes: HashMap::new(),
            deadzone: 0.15,
        }
    }

    /// Check if a button is pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.buttons.contains(&button)
    }

    /// Get an axis value (with deadzone applied)
    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        let value = self.axes.get(&axis).copied().unwrap_or(0.0);
        if value.abs() < self.deadzone {
            0.0
        } else {
            value
        }
    }

    /// Get the left stick as a vector
    pub fn left_stick(&self) -> Vec2 {
        Vec2::new(
            self.axis(GamepadAxis::LeftStickX),
            self.axis(GamepadAxis::LeftStickY),
        )
    }

    /// Get the right stick as a vector
    pub fn right_stick(&self) -> Vec2 {
        Vec2::new(
            self.axis(GamepadAxis::RightStickX),
            self.axis(GamepadAxis::RightStickY),
        )
    }

    /// Handle a gamepad event
    pub fn handle_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::GamepadConnected { id } if *id == self.id => {
                self.connected = true;
            }
            InputEvent::GamepadDisconnected { id } if *id == self.id => {
                self.connected = false;
                self.buttons.clear();
                self.axes.clear();
            }
            InputEvent::GamepadButtonPressed { id, button } if *id == self.id => {
                self.buttons.insert(*button);
            }
            InputEvent::GamepadButtonReleased { id, button } if *id == self.id => {
                self.buttons.remove(button);
            }
            InputEvent::GamepadAxisMoved { id, axis, value } if *id == self.id => {
                self.axes.insert(*axis, *value);
            }
            _ => {}
        }
    }
}

/// Complete input state
#[derive(Debug, Clone, Default)]
pub struct InputState {
    /// Currently pressed keys
    keys_pressed: HashSet<KeyCode>,
    /// Keys pressed this frame
    keys_just_pressed: HashSet<KeyCode>,
    /// Keys released this frame
    keys_just_released: HashSet<KeyCode>,
    /// Mouse buttons pressed
    mouse_buttons: HashSet<MouseButton>,
    /// Mouse buttons just pressed
    mouse_just_pressed: HashSet<MouseButton>,
    /// Mouse buttons just released
    mouse_just_released: HashSet<MouseButton>,
    /// Mouse position
    mouse_position: Vec2,
    /// Mouse delta this frame
    mouse_delta: Vec2,
    /// Mouse wheel delta
    mouse_wheel: Vec2,
    /// Current modifiers
    modifiers: Modifiers,
    /// Touch state
    touch: TouchState,
    /// Gamepad states
    gamepads: HashMap<u32, GamepadState>,
}

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    /// Check if a mouse button is pressed
    pub fn is_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(&button)
    }

    /// Check if a mouse button was just pressed
    pub fn is_mouse_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&button)
    }

    /// Get the mouse position
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Get the mouse movement delta
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    /// Get the mouse wheel delta
    pub fn mouse_wheel(&self) -> Vec2 {
        self.mouse_wheel
    }

    /// Get current modifiers
    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Get the touch state
    pub fn touch(&self) -> &TouchState {
        &self.touch
    }

    /// Get a gamepad state
    pub fn gamepad(&self, id: u32) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }

    /// Handle an input event
    pub fn handle_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyPressed(key) => {
                if self.keys_pressed.insert(*key) {
                    self.keys_just_pressed.insert(*key);
                }
                self.update_modifiers(*key, true);
            }
            InputEvent::KeyReleased(key) => {
                if self.keys_pressed.remove(key) {
                    self.keys_just_released.insert(*key);
                }
                self.update_modifiers(*key, false);
            }
            InputEvent::MousePressed(button) => {
                if self.mouse_buttons.insert(*button) {
                    self.mouse_just_pressed.insert(*button);
                }
            }
            InputEvent::MouseReleased(button) => {
                if self.mouse_buttons.remove(button) {
                    self.mouse_just_released.insert(*button);
                }
            }
            InputEvent::MouseMoved { x, y } => {
                let new_pos = Vec2::new(*x, *y);
                self.mouse_delta = new_pos - self.mouse_position;
                self.mouse_position = new_pos;
            }
            InputEvent::MouseWheel { delta_x, delta_y } => {
                self.mouse_wheel = Vec2::new(*delta_x, *delta_y);
            }
            InputEvent::GamepadConnected { id } => {
                self.gamepads.entry(*id).or_insert_with(|| GamepadState::new(*id));
                if let Some(gamepad) = self.gamepads.get_mut(id) {
                    gamepad.handle_event(event);
                }
            }
            _ => {
                // Handle touch events
                self.touch.handle_event(event);
                
                // Handle gamepad events
                for gamepad in self.gamepads.values_mut() {
                    gamepad.handle_event(event);
                }
            }
        }
    }

    fn update_modifiers(&mut self, key: KeyCode, pressed: bool) {
        let modifier = match key {
            KeyCode::LeftShift | KeyCode::RightShift => Modifiers::SHIFT,
            KeyCode::LeftControl | KeyCode::RightControl => Modifiers::CONTROL,
            KeyCode::LeftAlt | KeyCode::RightAlt => Modifiers::ALT,
            _ => return,
        };

        if pressed {
            self.modifiers |= modifier;
        } else {
            self.modifiers -= modifier;
        }
    }

    /// Clear per-frame state (call at the end of each frame)
    pub fn end_frame(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel = Vec2::ZERO;
        self.touch.clear_ended();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_input() {
        let mut input = InputState::new();
        
        input.handle_event(&InputEvent::KeyPressed(KeyCode::Space));
        assert!(input.is_key_pressed(KeyCode::Space));
        assert!(input.is_key_just_pressed(KeyCode::Space));
        
        input.end_frame();
        assert!(input.is_key_pressed(KeyCode::Space));
        assert!(!input.is_key_just_pressed(KeyCode::Space));
        
        input.handle_event(&InputEvent::KeyReleased(KeyCode::Space));
        assert!(!input.is_key_pressed(KeyCode::Space));
        assert!(input.is_key_just_released(KeyCode::Space));
    }

    #[test]
    fn test_mouse_input() {
        let mut input = InputState::new();
        
        input.handle_event(&InputEvent::MouseMoved { x: 100.0, y: 200.0 });
        assert_eq!(input.mouse_position(), Vec2::new(100.0, 200.0));
        
        input.handle_event(&InputEvent::MouseMoved { x: 110.0, y: 210.0 });
        assert_eq!(input.mouse_delta(), Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_touch_input() {
        let mut input = InputState::new();
        
        input.handle_event(&InputEvent::TouchStarted { id: 0, x: 50.0, y: 100.0 });
        assert!(input.touch().is_touching());
        assert_eq!(input.touch().touch_count(), 1);
        
        input.handle_event(&InputEvent::TouchEnded { id: 0, x: 50.0, y: 100.0 });
        input.touch.clear_ended();
        assert!(!input.touch().is_touching());
    }

    #[test]
    fn test_gamepad_input() {
        let mut input = InputState::new();
        
        input.handle_event(&InputEvent::GamepadConnected { id: 0 });
        assert!(input.gamepad(0).is_some());
        assert!(input.gamepad(0).unwrap().connected);
        
        input.handle_event(&InputEvent::GamepadButtonPressed { id: 0, button: GamepadButton::South });
        assert!(input.gamepad(0).unwrap().is_button_pressed(GamepadButton::South));
    }

    #[test]
    fn test_modifiers() {
        let mut input = InputState::new();
        
        input.handle_event(&InputEvent::KeyPressed(KeyCode::LeftShift));
        assert!(input.modifiers().contains(Modifiers::SHIFT));
        
        input.handle_event(&InputEvent::KeyPressed(KeyCode::LeftControl));
        assert!(input.modifiers().contains(Modifiers::SHIFT | Modifiers::CONTROL));
    }
}
