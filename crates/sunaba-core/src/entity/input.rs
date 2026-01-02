//! Player input state

use crate::simulation::MaterialId;

/// Tracks current input state for player control
#[derive(Debug, Clone)]
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
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}
