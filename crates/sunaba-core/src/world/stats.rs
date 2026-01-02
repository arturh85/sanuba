//! Simulation statistics collection trait

/// Trait for collecting simulation statistics
///
/// This allows sunaba-core to record stats without depending on the full
/// stats collection implementation in the main crate.
pub trait SimStats {
    /// Record that a pixel was moved during simulation
    fn record_pixel_moved(&mut self);

    /// Record that a state change occurred (e.g., melting, freezing)
    fn record_state_change(&mut self);

    /// Record that a chemical reaction occurred
    fn record_reaction(&mut self);
}

/// A no-op implementation for when stats collection is not needed
#[derive(Default)]
pub struct NoopStats;

impl SimStats for NoopStats {
    fn record_pixel_moved(&mut self) {}
    fn record_state_change(&mut self) {}
    fn record_reaction(&mut self) {}
}
