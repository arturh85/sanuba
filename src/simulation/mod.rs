//! Simulation systems - materials, reactions, temperature, pressure

pub mod light;
mod materials;
pub mod mining;
pub mod reactions;
pub mod regeneration;
pub mod state_changes;
pub mod structural;
pub mod temperature;

pub use light::LightPropagation;
pub use materials::{MaterialDef, MaterialId, MaterialTag, MaterialType, Materials};
pub use reactions::{Reaction, ReactionRegistry};
pub use regeneration::RegenerationSystem;
pub use state_changes::StateChangeSystem;
pub use structural::StructuralIntegritySystem;
pub use temperature::{add_heat_at_pixel, get_temperature_at_pixel, TemperatureSimulator};
