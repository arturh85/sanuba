//! Kinematic falling chunks - simple physics without rapier2d
//!
//! Large debris falls as a unit with gravity, no rotation.
//! When it hits ground, it settles back into static pixels.
//!
//! This is WASM-compatible and used by both native game and SpacetimeDB server.

use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chunk of pixels falling as a unit
#[derive(Clone, Serialize, Deserialize)]
pub struct FallingChunk {
    /// Pixels relative to center, with their material IDs
    pub pixels: HashMap<IVec2, u16>,
    /// Center position in world space (floating point for smooth movement)
    pub center: Vec2,
    /// Vertical velocity (pixels per second, negative = falling)
    pub velocity_y: f32,
    /// Unique ID for tracking
    pub id: u64,
}

/// Render data for a falling chunk (used by renderer)
#[derive(Clone)]
pub struct ChunkRenderData {
    pub center: Vec2,
    pub pixels: HashMap<IVec2, u16>,
}

/// Manages all falling chunks
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct FallingChunkSystem {
    chunks: Vec<FallingChunk>,
    next_id: u64,
}

/// Trait for world collision queries (implemented by World)
pub trait WorldCollisionQuery {
    fn is_solid_at(&self, x: i32, y: i32) -> bool;
}

impl FallingChunkSystem {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            next_id: 0,
        }
    }

    /// Create a new falling chunk from a set of world positions with materials
    /// Returns the chunk ID
    pub fn create_chunk(&mut self, pixels: HashMap<IVec2, u16>) -> u64 {
        if pixels.is_empty() {
            return 0;
        }

        // Calculate center of mass
        let center = Self::calculate_center(&pixels);

        // Convert to relative positions
        let center_i = IVec2::new(center.x.round() as i32, center.y.round() as i32);
        let relative_pixels: HashMap<IVec2, u16> = pixels
            .into_iter()
            .map(|(pos, mat)| (pos - center_i, mat))
            .collect();

        let id = self.next_id;
        self.next_id += 1;

        log::info!(
            "FallingChunks: Created chunk {} with {} pixels at ({:.1}, {:.1})",
            id,
            relative_pixels.len(),
            center.x,
            center.y
        );

        self.chunks.push(FallingChunk {
            pixels: relative_pixels,
            center,
            velocity_y: 0.0,
            id,
        });

        id
    }

    /// Update all chunks with gravity, returns list of chunks that have settled
    pub fn update<W: WorldCollisionQuery>(&mut self, dt: f32, world: &W) -> Vec<FallingChunk> {
        const GRAVITY: f32 = -300.0; // pixels/s^2 (negative = down)
        const TERMINAL_VELOCITY: f32 = -500.0;
        const SETTLE_VELOCITY: f32 = -5.0; // Velocity threshold to consider settled

        let mut settled = Vec::new();
        let mut i = 0;

        while i < self.chunks.len() {
            let chunk = &mut self.chunks[i];

            // Apply gravity
            chunk.velocity_y = (chunk.velocity_y + GRAVITY * dt).max(TERMINAL_VELOCITY);

            // Calculate desired movement
            let delta_y = chunk.velocity_y * dt;

            // Check if we can move down
            if delta_y < 0.0 {
                let steps = (-delta_y).ceil() as i32;
                let mut moved = 0;

                for _ in 0..steps {
                    if Self::can_move_chunk(chunk, 0, -1, world) {
                        chunk.center.y -= 1.0;
                        moved += 1;
                    } else {
                        // Hit something - stop vertical velocity
                        chunk.velocity_y = 0.0;
                        break;
                    }
                }

                // If we couldn't move at all and velocity is low, settle
                if moved == 0 && chunk.velocity_y.abs() < SETTLE_VELOCITY.abs() {
                    log::info!(
                        "FallingChunks: Chunk {} settled at ({:.1}, {:.1})",
                        chunk.id,
                        chunk.center.x,
                        chunk.center.y
                    );
                    settled.push(self.chunks.remove(i));
                    continue;
                }
            }

            i += 1;
        }

        settled
    }

    /// Get all chunks for rendering
    pub fn get_render_data(&self) -> Vec<ChunkRenderData> {
        self.chunks
            .iter()
            .map(|c| ChunkRenderData {
                center: c.center,
                pixels: c.pixels.clone(),
            })
            .collect()
    }

    /// Get number of active falling chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Check if a chunk can move by (dx, dy) without collision
    fn can_move_chunk<W: WorldCollisionQuery>(
        chunk: &FallingChunk,
        dx: i32,
        dy: i32,
        world: &W,
    ) -> bool {
        let center_i = IVec2::new(chunk.center.x.round() as i32, chunk.center.y.round() as i32);

        for relative_pos in chunk.pixels.keys() {
            let new_world_pos = center_i + *relative_pos + IVec2::new(dx, dy);
            if world.is_solid_at(new_world_pos.x, new_world_pos.y) {
                return false;
            }
        }
        true
    }

    /// Calculate center of mass from pixel positions
    fn calculate_center(pixels: &HashMap<IVec2, u16>) -> Vec2 {
        if pixels.is_empty() {
            return Vec2::ZERO;
        }
        let sum: Vec2 = pixels
            .keys()
            .map(|p| Vec2::new(p.x as f32, p.y as f32))
            .sum();
        sum / pixels.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestWorld {
        solids: std::collections::HashSet<IVec2>,
    }

    impl WorldCollisionQuery for TestWorld {
        fn is_solid_at(&self, x: i32, y: i32) -> bool {
            self.solids.contains(&IVec2::new(x, y))
        }
    }

    #[test]
    fn test_create_chunk() {
        let mut system = FallingChunkSystem::new();
        let mut pixels = HashMap::new();
        pixels.insert(IVec2::new(0, 0), 1);
        pixels.insert(IVec2::new(1, 0), 1);
        pixels.insert(IVec2::new(0, 1), 1);
        pixels.insert(IVec2::new(1, 1), 1);

        let id = system.create_chunk(pixels);
        assert_eq!(id, 0);
        assert_eq!(system.chunk_count(), 1);
    }

    #[test]
    fn test_falling_chunk_settles() {
        let mut system = FallingChunkSystem::new();
        let mut pixels = HashMap::new();
        pixels.insert(IVec2::new(0, 10), 1);

        system.create_chunk(pixels);

        // Create ground at y=0
        let mut solids = std::collections::HashSet::new();
        for x in -10..10 {
            solids.insert(IVec2::new(x, 0));
        }
        let world = TestWorld { solids };

        // Simulate until settled
        let mut settled = Vec::new();
        for _ in 0..100 {
            settled.extend(system.update(0.016, &world));
            if system.chunk_count() == 0 {
                break;
            }
        }

        assert_eq!(settled.len(), 1);
        assert_eq!(system.chunk_count(), 0);
    }

    #[test]
    fn test_calculate_center() {
        let mut pixels = HashMap::new();
        pixels.insert(IVec2::new(0, 0), 1);
        pixels.insert(IVec2::new(2, 0), 1);
        pixels.insert(IVec2::new(0, 2), 1);
        pixels.insert(IVec2::new(2, 2), 1);

        let center = FallingChunkSystem::calculate_center(&pixels);
        assert!((center.x - 1.0).abs() < 0.01);
        assert!((center.y - 1.0).abs() < 0.01);
    }
}
