//! SpacetimeDB multiplayer server module for Sunaba
//!
//! This module provides multiplayer support with:
//! - World simulation (falling sand cellular automata)
//! - Server-side creature AI (neural network inference)
//! - Player synchronization
//!
//! Note: This is a minimal implementation to establish the structure.
//! Full functionality will be added incrementally.

mod encoding;
mod helpers;
mod reducers;
mod state;
mod tables;
mod world_access;

// Re-export tables for external access (if needed)
pub use tables::*;

// Re-export reducers (SpacetimeDB will discover them)
pub use reducers::*;

// Re-export utilities
pub use encoding::*;
pub use helpers::*;
pub use state::*;
pub use world_access::*;

// Note: WorldRng is automatically implemented for any rand::Rng via blanket impl in sunaba-core
// This includes SpacetimeDB's ctx.rng() which implements rand::Rng
