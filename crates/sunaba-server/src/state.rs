//! Global server state for SpacetimeDB module

use once_cell::sync::Lazy;
use std::sync::Mutex;

// ============================================================================
// Global Server State
// ============================================================================

/// Global server-side World instance (persists across reducer calls)
pub static SERVER_WORLD: Lazy<Mutex<Option<sunaba_core::world::World>>> =
    Lazy::new(|| Mutex::new(None));

// ============================================================================
// Helper Types
// ============================================================================

/// No-op stats implementation for server
pub struct NoOpStats;

impl sunaba_core::world::SimStats for NoOpStats {
    fn record_pixel_moved(&mut self) {}
    fn record_state_change(&mut self) {}
    fn record_reaction(&mut self) {}
}
