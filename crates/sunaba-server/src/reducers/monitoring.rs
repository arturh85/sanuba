//! Multiplayer monitoring and metrics reducers

use spacetimedb::ReducerContext;

// ============================================================================
// Multiplayer Metrics & Monitoring
// ============================================================================

/// Ping-pong reducer for latency measurement
/// Client sends timestamp, measures round-trip time on response
#[spacetimedb::reducer]
pub fn request_ping(ctx: &ReducerContext, client_timestamp_ms: u64) {
    log::trace!("Ping from {:?} at {}ms", ctx.sender, client_timestamp_ms);
    // Immediate response - client calculates RTT = now - client_timestamp_ms
}
