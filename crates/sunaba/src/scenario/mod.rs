//! Scenario-based testing and control system
//!
//! This module provides a RON-based scenario scripting system for automated testing
//! and remote control of the game. It enables:
//! - High-level commands (MovePlayerTo, MineCircle, PlaceMaterial)
//! - Low-level input simulation (SimulateKey, SimulateMouseClick)
//! - State verification (MaterialCount, PlayerPosition, etc.)
//! - Headless execution with JSON results
//!
//! ## Example Usage
//!
//! ```bash
//! # Run a scenario from file
//! cargo run --features headless -- --test-scenario scenarios/test_mining.ron
//!
//! # Run all scenarios
//! just test-scenario-all
//! ```
//!
//! ## RON Scenario Format
//!
//! ```ron
//! (
//!     name: "Mining Test",
//!     description: "Verify mining mechanics",
//!     setup: [(type: "TeleportPlayer", x: 0.0, y: 100.0)],
//!     actions: [(type: "MineCircle", center_x: 0, center_y: 25, radius: 10)],
//!     verify: [(type: "RegionEmpty", region: (type: "Circle", center_x: 0, center_y: 25, radius: 8))],
//! )
//! ```

pub mod actions;
pub mod definition;
pub mod executor;
pub mod results;
pub mod validated_types;
pub mod verification;

pub use actions::{MouseButton, ScenarioAction};
pub use definition::ScenarioDefinition;
pub use executor::{ScenarioExecutor, ScenarioExecutorConfig};
pub use results::ExecutionReport;
pub use validated_types::{
    CreatureArchetype, SimulatedKey, ValidatedHealth, ValidatedHunger, ValidatedMaterialId,
    ValidatedRadius, ValidatedSlotIndex,
};
pub use verification::{Region, VerificationCondition, VerificationResult};
