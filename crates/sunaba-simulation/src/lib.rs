//! Material simulation data and reactions for Sunaba
//!
//! This crate provides the foundational data types for material simulation:
//! - Material definitions (MaterialId, MaterialDef, Materials)
//! - Material types and tags (MaterialType, MaterialTag)
//! - Chemical reactions (Reaction, ReactionRegistry)
//! - Pixel types (Pixel, pixel_flags, CHUNK_SIZE)
//! - Texture variation for visual depth

mod materials;
mod pixel;
mod reactions;
pub mod texture_variation;

pub use materials::{MaterialDef, MaterialId, MaterialTag, MaterialType, Materials};
pub use pixel::{CHUNK_AREA, CHUNK_SIZE, Pixel, pixel_flags};
pub use reactions::{Reaction, ReactionRegistry};
pub use texture_variation::apply_texture_variation;
