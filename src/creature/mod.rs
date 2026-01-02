//! Creature system - CPPN-NEAT morphology generation with neural control
//!
//! This module implements:
//! - CPPN-NEAT genomes for articulated morphology generation
//! - Graph Neural Networks for morphology-agnostic control
//! - GOAP behavior planning for high-level decision making
//! - Full world interaction (sensing, eating, mining, building)

#![allow(clippy::module_inception)]

use glam::Vec2;

pub mod behavior;
pub mod creature;
pub mod genome;
pub mod morphology;
pub mod neural;
pub mod sensors;
pub mod spawning;
pub mod world_interaction;

// Re-export main types for convenience
pub use creature::Creature;
pub use genome::CreatureGenome;
pub use morphology::{CreatureMorphology, MorphologyPhysics};
pub use spawning::CreatureManager;

/// Body part render data for a single body part
#[derive(Debug, Clone)]
pub struct BodyPartRenderData {
    pub position: Vec2,
    pub radius: f32,
    pub color: [u8; 4],
}

/// Render data for an entire creature
#[derive(Debug, Clone)]
pub struct CreatureRenderData {
    pub body_parts: Vec<BodyPartRenderData>,
}
