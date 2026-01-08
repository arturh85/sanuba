//! Scenario actions - commands that can be executed in test scenarios

use serde::{Deserialize, Serialize};
use sunaba_core::entity::inventory::ItemStack;

use super::validated_types::{
    CreatureArchetype, SimulatedKey, ValidatedHealth, ValidatedHunger, ValidatedMaterialId,
    ValidatedRadius, ValidatedSlotIndex,
};
use super::verification::VerificationCondition;

/// Actions that can be performed in a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScenarioAction {
    // === HIGH-LEVEL GAME COMMANDS ===
    /// Teleport player to exact position
    TeleportPlayer { x: f32, y: f32 },

    /// Move player smoothly to target (simulates walking/flying)
    MovePlayerTo {
        x: f32,
        y: f32,
        /// Max time to spend moving (seconds, prevents infinite loops)
        timeout: f32,
    },

    /// Mine a circular area (uses debug_mine_circle)
    MineCircle {
        center_x: i32,
        center_y: i32,
        radius: ValidatedRadius,
    },

    /// Mine a rectangular area
    MineRect {
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
    },

    /// Place material in circular brush (uses place_material_debug)
    PlaceMaterial {
        x: i32,
        y: i32,
        material: ValidatedMaterialId,
        radius: ValidatedRadius,
    },

    /// Place material in rectangular area
    FillRect {
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
        material: ValidatedMaterialId,
    },

    /// Give item to player inventory
    GiveItem {
        item: ItemStack,
        slot: Option<ValidatedSlotIndex>,
    },

    /// Remove item from player inventory
    RemoveItem { slot: ValidatedSlotIndex },

    /// Set player health directly
    SetPlayerHealth { health: ValidatedHealth },

    /// Set player hunger directly
    SetPlayerHunger { hunger: ValidatedHunger },

    /// Spawn creature at position
    SpawnCreature {
        genome_type: CreatureArchetype,
        x: f32,
        y: f32,
    },

    /// Remove all creatures from world
    ClearCreatures,

    /// Set time of day (0.0-1.0, affects lighting)
    SetTimeOfDay { time: f32 },

    /// Load a demo level by ID (uses LevelManager)
    LoadLevel { level_id: usize },

    /// Set world seed and regenerate
    SetWorldSeed { seed: u64 },

    // === LOW-LEVEL INPUT SIMULATION ===
    /// Simulate key press for N frames (w, a, s, d, space)
    SimulateKey { key: SimulatedKey, frames: usize },

    /// Simulate mouse click at world coordinates
    SimulateMouseClick {
        world_x: i32,
        world_y: i32,
        button: MouseButton,
        frames: usize, // Hold for N frames
    },

    /// Simulate mouse movement to world coordinates
    SimulateMouseMove { world_x: i32, world_y: i32 },

    // === CONTROL FLOW ===
    /// Wait for N simulation frames
    WaitFrames { frames: usize },

    /// Wait until condition is met (max timeout)
    WaitUntil {
        condition: VerificationCondition,
        timeout_frames: usize,
    },

    /// Capture screenshot to file
    CaptureScreenshot {
        filename: String,
        width: Option<usize>,
        height: Option<usize>,
    },

    /// Log a message to console
    Log { message: String },

    /// Run nested actions in sequence
    Sequence { actions: Vec<ScenarioAction> },
}

/// Mouse button for click simulation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}
