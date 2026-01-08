//! Player input state

use crate::simulation::MaterialId;
use serde::{Deserialize, Serialize};

#[cfg(feature = "client")]
use web_time::{Duration, Instant};

/// Tracks current input state for player control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputState {
    // Movement keys
    pub w_pressed: bool,
    pub a_pressed: bool,
    pub s_pressed: bool,
    pub d_pressed: bool,
    pub jump_pressed: bool, // Space bar for jumping

    // Material selection (1-9 map to material IDs)
    pub selected_material: u16,

    // Mouse state
    pub mouse_world_pos: Option<(i32, i32)>, // Converted to world coords
    pub left_mouse_pressed: bool,
    pub right_mouse_pressed: bool,
    pub prev_right_mouse_pressed: bool, // Previous frame's right mouse state

    // Zoom control
    pub zoom_delta: f32, // Zoom change this frame (1.0 = no change)

    // Dash mechanics
    pub shift_pressed: bool, // For Shift+A/D instant dash

    // Double-tap timing (expires after 300ms)
    #[cfg(feature = "client")]
    #[serde(skip)]
    pub a_last_tap: Option<Instant>,
    #[cfg(feature = "client")]
    #[serde(skip)]
    pub d_last_tap: Option<Instant>,

    // Double-tap flags (set when detected, cleared each frame)
    #[cfg(feature = "client")]
    #[serde(skip)]
    pub a_double_tap: bool,
    #[cfg(feature = "client")]
    #[serde(skip)]
    pub d_double_tap: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            w_pressed: false,
            a_pressed: false,
            s_pressed: false,
            d_pressed: false,
            jump_pressed: false,
            selected_material: MaterialId::SAND, // Start with sand
            mouse_world_pos: None,
            left_mouse_pressed: false,
            right_mouse_pressed: false,
            prev_right_mouse_pressed: false,
            zoom_delta: 1.0, // No change by default
            shift_pressed: false,
            #[cfg(feature = "client")]
            a_last_tap: None,
            #[cfg(feature = "client")]
            d_last_tap: None,
            #[cfg(feature = "client")]
            a_double_tap: false,
            #[cfg(feature = "client")]
            d_double_tap: false,
        }
    }

    /// Detect double-tap for a key
    ///
    /// Returns true if the key was pressed within 200ms of the last tap.
    /// Updates the last_tap timestamp and consumes it on double-tap.
    #[cfg(feature = "client")]
    pub fn detect_double_tap(
        last_tap: &mut Option<Instant>,
        pressed: bool,
        prev_pressed: bool,
    ) -> bool {
        // Key just pressed (rising edge)
        if pressed && !prev_pressed {
            let now = Instant::now();

            // Check if within 200ms window
            if let Some(last) = *last_tap
                && now.duration_since(last) < Duration::from_millis(200)
            {
                *last_tap = None; // Consume tap
                return true; // Double-tap detected!
            }

            // Single tap - record timestamp
            *last_tap = Some(now);
        }

        false
    }

    /// Expire old tap timestamps (call at end of frame)
    ///
    /// Taps older than 300ms are cleared to prevent stale double-taps.
    #[cfg(feature = "client")]
    pub fn expire_taps(&mut self) {
        let now = Instant::now();
        let expire_window = Duration::from_millis(300);

        if let Some(last) = self.a_last_tap
            && now.duration_since(last) > expire_window
        {
            self.a_last_tap = None;
        }

        if let Some(last) = self.d_last_tap
            && now.duration_since(last) > expire_window
        {
            self.d_last_tap = None;
        }
    }

    /// Clear per-frame flags (call at end of frame)
    ///
    /// Double-tap flags are only valid for one frame.
    #[cfg(feature = "client")]
    pub fn clear_frame_flags(&mut self) {
        self.a_double_tap = false;
        self.d_double_tap = false;
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_new() {
        let input = InputState::new();

        // All movement keys should be unpressed
        assert!(!input.w_pressed);
        assert!(!input.a_pressed);
        assert!(!input.s_pressed);
        assert!(!input.d_pressed);
        assert!(!input.jump_pressed);

        // Default material should be sand
        assert_eq!(input.selected_material, MaterialId::SAND);

        // Mouse state should be reset
        assert!(input.mouse_world_pos.is_none());
        assert!(!input.left_mouse_pressed);
        assert!(!input.right_mouse_pressed);
        assert!(!input.prev_right_mouse_pressed);

        // Zoom should be 1.0 (no change)
        assert_eq!(input.zoom_delta, 1.0);
    }

    #[test]
    fn test_input_state_default() {
        let input = InputState::default();
        let new_input = InputState::new();

        assert_eq!(input.selected_material, new_input.selected_material);
        assert_eq!(input.zoom_delta, new_input.zoom_delta);
    }

    #[test]
    fn test_input_state_modifiable() {
        let mut input = InputState::new();

        // Simulate pressing movement keys
        input.w_pressed = true;
        input.a_pressed = true;
        assert!(input.w_pressed);
        assert!(input.a_pressed);

        // Simulate mouse input
        input.mouse_world_pos = Some((100, 200));
        input.left_mouse_pressed = true;
        assert_eq!(input.mouse_world_pos, Some((100, 200)));
        assert!(input.left_mouse_pressed);

        // Change material selection
        input.selected_material = MaterialId::WATER;
        assert_eq!(input.selected_material, MaterialId::WATER);

        // Zoom
        input.zoom_delta = 1.5;
        assert_eq!(input.zoom_delta, 1.5);
    }
}
