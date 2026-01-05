//! Chunk status and management utilities

use glam::{IVec2, Vec2};

use super::chunk_manager::ChunkManager;
use super::{CHUNK_SIZE, Chunk};

/// Chunk status query and management utilities
pub struct ChunkStatus;

impl ChunkStatus {
    /// Check if a chunk needs CA update based on dirty state of itself and neighbors
    /// Returns true if this chunk or any of its 8 neighbors have dirty_rect set or simulation_active
    pub fn needs_ca_update(chunk_manager: &ChunkManager, pos: IVec2) -> bool {
        // Check the chunk itself
        if let Some(chunk) = chunk_manager.chunks.get(&pos)
            && (chunk.dirty_rect.is_some() || chunk.is_simulation_active())
        {
            return true;
        }

        // Check all 8 neighbors - materials can flow in from any direction
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let neighbor_pos = IVec2::new(pos.x + dx, pos.y + dy);
                if let Some(neighbor) = chunk_manager.chunks.get(&neighbor_pos)
                    && (neighbor.dirty_rect.is_some() || neighbor.is_simulation_active())
                {
                    return true;
                }
            }
        }

        false
    }

    /// Update active chunks: remove distant chunks and re-activate nearby loaded chunks
    /// Returns the number of chunks added to the active list
    pub fn update_active_chunks(
        chunk_manager: &mut ChunkManager,
        player_position: Vec2,
        active_chunk_radius: i32,
    ) -> usize {
        let player_chunk_x = (player_position.x as i32).div_euclid(CHUNK_SIZE as i32);
        let player_chunk_y = (player_position.y as i32).div_euclid(CHUNK_SIZE as i32);

        // 1. Remove distant chunks from active list
        chunk_manager.active_chunks.retain(|pos| {
            let dist_x = (pos.x - player_chunk_x).abs();
            let dist_y = (pos.y - player_chunk_y).abs();
            dist_x <= active_chunk_radius && dist_y <= active_chunk_radius
        });

        // 2. Add nearby loaded chunks that aren't currently active
        let mut added_count = 0;
        for cy in (player_chunk_y - active_chunk_radius)..=(player_chunk_y + active_chunk_radius) {
            for cx in
                (player_chunk_x - active_chunk_radius)..=(player_chunk_x + active_chunk_radius)
            {
                let pos = IVec2::new(cx, cy);

                // If chunk is loaded but not active, add it to active list
                if chunk_manager.chunks.contains_key(&pos)
                    && !chunk_manager.active_chunks.contains(&pos)
                {
                    chunk_manager.active_chunks.push(pos);
                    added_count += 1;

                    // Mark newly activated chunks for simulation so physics/chemistry runs
                    if let Some(chunk) = chunk_manager.chunks.get_mut(&pos) {
                        chunk.set_simulation_active(true);
                    }
                }
            }
        }

        added_count
    }

    /// Ensure chunks exist for rectangular area (pre-allocation)
    /// Creates empty chunks for the given world coordinate bounds if they don't exist
    /// Used by headless training to set up scenarios without full world generation
    pub fn ensure_chunks_for_area(
        chunk_manager: &mut ChunkManager,
        min_x: i32,
        min_y: i32,
        max_x: i32,
        max_y: i32,
    ) {
        let (min_chunk, _, _) = ChunkManager::world_to_chunk_coords(min_x, min_y);
        let (max_chunk, _, _) = ChunkManager::world_to_chunk_coords(max_x, max_y);

        for cy in min_chunk.y..=max_chunk.y {
            for cx in min_chunk.x..=max_chunk.x {
                let pos = IVec2::new(cx, cy);
                chunk_manager
                    .chunks
                    .entry(pos)
                    .or_insert_with(|| Chunk::new(cx, cy));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::chunk::DirtyRect;

    /// Create a test chunk manager with a single chunk at origin
    fn setup_single_chunk() -> ChunkManager {
        let mut manager = ChunkManager::new();
        manager.chunks.insert(IVec2::new(0, 0), Chunk::new(0, 0));
        manager
    }

    /// Create chunk manager with 3x3 grid of chunks centered at origin
    fn setup_3x3_chunks() -> ChunkManager {
        let mut manager = ChunkManager::new();
        for cy in -1..=1 {
            for cx in -1..=1 {
                manager.chunks.insert(IVec2::new(cx, cy), Chunk::new(cx, cy));
            }
        }
        manager
    }

    #[test]
    fn test_needs_ca_update_clean_chunk() {
        let manager = setup_single_chunk();

        // Clean chunk with no dirty rect should not need update
        let needs_update = ChunkStatus::needs_ca_update(&manager, IVec2::new(0, 0));
        assert!(!needs_update, "Clean chunk should not need CA update");
    }

    #[test]
    fn test_needs_ca_update_dirty_rect() {
        let mut manager = setup_single_chunk();

        // Mark chunk as having dirty rect
        if let Some(chunk) = manager.chunks.get_mut(&IVec2::new(0, 0)) {
            chunk.dirty_rect = Some(DirtyRect::new(10, 10));
        }

        let needs_update = ChunkStatus::needs_ca_update(&manager, IVec2::new(0, 0));
        assert!(needs_update, "Chunk with dirty rect should need CA update");
    }

    #[test]
    fn test_needs_ca_update_simulation_active() {
        let mut manager = setup_single_chunk();

        // Mark chunk as having active simulation
        if let Some(chunk) = manager.chunks.get_mut(&IVec2::new(0, 0)) {
            chunk.set_simulation_active(true);
        }

        let needs_update = ChunkStatus::needs_ca_update(&manager, IVec2::new(0, 0));
        assert!(needs_update, "Chunk with active simulation should need CA update");
    }

    #[test]
    fn test_needs_ca_update_dirty_neighbor() {
        let mut manager = setup_3x3_chunks();

        // Mark neighbor chunk as dirty
        if let Some(chunk) = manager.chunks.get_mut(&IVec2::new(1, 0)) {
            chunk.dirty_rect = Some(DirtyRect::new(10, 10));
        }

        // Center chunk should need update because neighbor is dirty
        let needs_update = ChunkStatus::needs_ca_update(&manager, IVec2::new(0, 0));
        assert!(needs_update, "Chunk should need update when neighbor is dirty");
    }

    #[test]
    fn test_needs_ca_update_missing_chunk() {
        let manager = ChunkManager::new(); // Empty

        // Missing chunk should not need update (nothing to update)
        let needs_update = ChunkStatus::needs_ca_update(&manager, IVec2::new(0, 0));
        assert!(!needs_update, "Missing chunk should not need CA update");
    }

    #[test]
    fn test_update_active_chunks_adds_nearby() {
        let mut manager = setup_3x3_chunks();

        // Player at center of chunk (0, 0)
        let player_pos = Vec2::new(32.0, 32.0);
        let active_radius = 1;

        let added = ChunkStatus::update_active_chunks(&mut manager, player_pos, active_radius);

        // All 9 chunks should be added to active list
        assert_eq!(added, 9, "Should add all 9 chunks within radius 1");
        assert_eq!(manager.active_chunks.len(), 9);
    }

    #[test]
    fn test_update_active_chunks_removes_distant() {
        let mut manager = setup_3x3_chunks();

        // Add all chunks as active initially
        for cy in -1..=1 {
            for cx in -1..=1 {
                manager.active_chunks.push(IVec2::new(cx, cy));
            }
        }

        // Player moves far away
        let player_pos = Vec2::new(10000.0, 10000.0);
        let active_radius = 1;

        ChunkStatus::update_active_chunks(&mut manager, player_pos, active_radius);

        // All old chunks should be removed
        assert_eq!(manager.active_chunks.len(), 0, "Distant chunks should be removed");
    }

    #[test]
    fn test_update_active_chunks_marks_simulation_active() {
        let mut manager = setup_3x3_chunks();

        let player_pos = Vec2::new(32.0, 32.0);
        let active_radius = 1;

        ChunkStatus::update_active_chunks(&mut manager, player_pos, active_radius);

        // Newly activated chunks should have simulation_active set
        for pos in &manager.active_chunks {
            let chunk = manager.chunks.get(pos).unwrap();
            assert!(chunk.is_simulation_active(), "Activated chunk should have simulation_active=true");
        }
    }

    #[test]
    fn test_update_active_chunks_no_duplicates() {
        let mut manager = setup_3x3_chunks();

        let player_pos = Vec2::new(32.0, 32.0);
        let active_radius = 1;

        // Run update twice
        ChunkStatus::update_active_chunks(&mut manager, player_pos, active_radius);
        let first_count = manager.active_chunks.len();

        ChunkStatus::update_active_chunks(&mut manager, player_pos, active_radius);
        let second_count = manager.active_chunks.len();

        assert_eq!(first_count, second_count, "Should not add duplicates");
    }

    #[test]
    fn test_ensure_chunks_for_area_single_chunk() {
        let mut manager = ChunkManager::new();

        // Area within single chunk
        ChunkStatus::ensure_chunks_for_area(&mut manager, 10, 10, 50, 50);

        assert_eq!(manager.chunks.len(), 1, "Should create 1 chunk");
        assert!(manager.chunks.contains_key(&IVec2::new(0, 0)));
    }

    #[test]
    fn test_ensure_chunks_for_area_multiple_chunks() {
        let mut manager = ChunkManager::new();

        // Area spanning 2x2 chunks
        ChunkStatus::ensure_chunks_for_area(&mut manager, 0, 0, 100, 100);

        assert_eq!(manager.chunks.len(), 4, "Should create 4 chunks (2x2)");
        assert!(manager.chunks.contains_key(&IVec2::new(0, 0)));
        assert!(manager.chunks.contains_key(&IVec2::new(1, 0)));
        assert!(manager.chunks.contains_key(&IVec2::new(0, 1)));
        assert!(manager.chunks.contains_key(&IVec2::new(1, 1)));
    }

    #[test]
    fn test_ensure_chunks_for_area_preserves_existing() {
        let mut manager = ChunkManager::new();

        // Create chunk with custom data
        let mut existing_chunk = Chunk::new(0, 0);
        existing_chunk.set_material(10, 10, 5); // Mark it
        manager.chunks.insert(IVec2::new(0, 0), existing_chunk);

        // Ensure chunks - should not overwrite existing
        ChunkStatus::ensure_chunks_for_area(&mut manager, 0, 0, 50, 50);

        let chunk = manager.chunks.get(&IVec2::new(0, 0)).unwrap();
        assert_eq!(chunk.get_material(10, 10), 5, "Existing chunk data should be preserved");
    }

    #[test]
    fn test_ensure_chunks_for_area_negative_coords() {
        let mut manager = ChunkManager::new();

        // Area in negative coordinates
        // CHUNK_SIZE=64, so -100..=-50 spans chunks -2 to -1 (2x2 = 4 chunks)
        ChunkStatus::ensure_chunks_for_area(&mut manager, -100, -100, -50, -50);

        assert_eq!(manager.chunks.len(), 4, "Should create 4 chunks in negative space (2x2)");
        assert!(manager.chunks.contains_key(&IVec2::new(-2, -2)));
        assert!(manager.chunks.contains_key(&IVec2::new(-1, -1)));
    }
}
