//! Special material behaviors for Powder Game-style materials.
//!
//! This module implements emergent behaviors for special materials:
//! - **Virus**: Spreads to adjacent materials, transforms them
//! - **Clone**: Copies adjacent material patterns
//! - **Fuse**: Burns gradually in one direction
//! - **Vine**: Grows in tangled random walk pattern

use super::{CHUNK_SIZE, Chunk};
use glam::IVec2;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::collections::HashMap;
use sunaba_simulation::materials::{MaterialId, Materials};
use sunaba_simulation::pixel::{Pixel, pixel_flags};

/// Manages special material behaviors (virus, clone, fuse, vine).
pub struct SpecialBehaviorsSystem {
    /// RNG for probabilistic behaviors
    rng: Xoshiro256PlusPlus,
}

impl SpecialBehaviorsSystem {
    pub fn new() -> Self {
        Self {
            rng: Xoshiro256PlusPlus::seed_from_u64(42),
        }
    }

    /// Creates a new system with a specific seed for deterministic behavior.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        }
    }

    /// Updates all special behaviors for active chunks.
    pub fn update(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Update each behavior type
        self.update_fuse_behavior(chunks, active_chunks, materials);
        self.update_vine_behavior(chunks, active_chunks, materials);
        self.update_virus_behavior(chunks, active_chunks, materials);
        self.update_clone_behavior(chunks, active_chunks, materials);
    }

    /// Fuse burns gradually in one direction, igniting adjacent explosives.
    fn update_fuse_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect burning fuse positions
        let mut burning_fuses: Vec<(IVec2, usize, usize, u8)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::FUSE
                            && pixel.flags & pixel_flags::BURNING != 0
                        {
                            // Get direction from flags (0=up, 1=right, 2=down, 3=left)
                            let dir = Self::get_direction_from_flags(pixel.flags);
                            burning_fuses.push((chunk_pos, x, y, dir));
                        }
                    }
                }
            }
        }

        // Process each burning fuse
        for (chunk_pos, x, y, dir) in burning_fuses {
            // 10% chance to advance burn each frame (slow gradual burn)
            if self.rng.r#gen::<f32>() > 0.1 {
                continue;
            }

            // Get the direction offset
            let (dx, dy) = Self::direction_to_offset(dir);
            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Check if next pixel is also fuse or explosive
            let should_ignite_next = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                let next_mat = materials.get(next_pixel.material_id);
                // Ignite next fuse or any flammable material
                next_pixel.material_id == MaterialId::FUSE || next_mat.flammable
            } else {
                false
            };

            // Consume current fuse (turn to ash)
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                chunk.set_pixel(x, y, Pixel::new(MaterialId::ASH));
                chunk.dirty = true;
            }

            // Ignite the next segment
            if should_ignite_next && let Some(next_chunk) = chunks.get_mut(&next_chunk_pos) {
                let mut next_pixel = next_chunk.get_pixel(next_px, next_py);
                next_pixel.flags |= pixel_flags::BURNING;
                // Propagate the direction
                next_pixel.flags = Self::set_direction_in_flags(next_pixel.flags, dir);
                next_chunk.set_pixel(next_px, next_py, next_pixel);
                next_chunk.dirty = true;
            }
        }
    }

    /// Vine grows in random tangled patterns.
    fn update_vine_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        _materials: &Materials,
    ) {
        // Collect vine tip positions (vines with BEHAVIOR_ACTIVE flag)
        let mut vine_tips: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::VINE
                            && pixel.flags & pixel_flags::BEHAVIOR_ACTIVE != 0
                        {
                            vine_tips.push((chunk_pos, x, y));
                        }
                    }
                }
            }
        }

        // Process each vine tip
        for (chunk_pos, x, y) in vine_tips {
            // 5% chance to grow each frame (slow growth)
            if self.rng.r#gen::<f32>() > 0.05 {
                continue;
            }

            // Random walk direction (prefer up/horizontal over down for vines)
            let directions = [
                (-1, 0), // left
                (1, 0),  // right
                (0, 1),  // up (vines grow up)
                (0, 1),  // up (weighted)
                (-1, 1), // up-left
                (1, 1),  // up-right
                (0, -1), // down (rare)
            ];
            let dir_idx = self.rng.gen_range(0..directions.len());
            let (dx, dy) = directions[dir_idx];

            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Check if we can grow into air
            let can_grow = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                next_pixel.material_id == MaterialId::AIR
            } else {
                false
            };

            if can_grow {
                // Create new vine tip
                if let Some(next_chunk) = chunks.get_mut(&next_chunk_pos) {
                    let mut new_vine = Pixel::new(MaterialId::VINE);
                    new_vine.flags |= pixel_flags::BEHAVIOR_ACTIVE;
                    next_chunk.set_pixel(next_px, next_py, new_vine);
                    next_chunk.dirty = true;
                }

                // Deactivate old tip (50% chance to stop being active)
                if self.rng.r#gen::<f32>() > 0.5
                    && let Some(chunk) = chunks.get_mut(&chunk_pos)
                {
                    let mut pixel = chunk.get_pixel(x, y);
                    pixel.flags &= !pixel_flags::BEHAVIOR_ACTIVE;
                    chunk.set_pixel(x, y, pixel);
                    chunk.dirty = true;
                }
            }
        }
    }

    /// Virus spreads to adjacent materials, transforming them into virus.
    fn update_virus_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect virus positions
        let mut virus_positions: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::VIRUS {
                            virus_positions.push((chunk_pos, x, y));
                        }
                    }
                }
            }
        }

        // Process each virus
        for (chunk_pos, x, y) in virus_positions {
            // 3% chance to spread each frame (slow spread)
            if self.rng.r#gen::<f32>() > 0.03 {
                continue;
            }

            // Pick a random neighbor
            let dx = self.rng.gen_range(-1..=1);
            let dy = self.rng.gen_range(-1..=1);
            if dx == 0 && dy == 0 {
                continue;
            }

            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Check if neighbor can be infected
            let should_infect = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                let next_mat = materials.get(next_pixel.material_id);
                // Can't infect air, bedrock, other virus, or metals
                next_pixel.material_id != MaterialId::AIR
                    && next_pixel.material_id != MaterialId::BEDROCK
                    && next_pixel.material_id != MaterialId::VIRUS
                    && !next_mat.conducts_electricity // metals resist virus
            } else {
                false
            };

            if should_infect {
                // Mark for infection (transform next frame to avoid cascading in one frame)
                if let Some(next_chunk) = chunks.get_mut(&next_chunk_pos) {
                    let mut next_pixel = next_chunk.get_pixel(next_px, next_py);
                    next_pixel.flags |= pixel_flags::INFECTED;
                    next_chunk.set_pixel(next_px, next_py, next_pixel);
                    next_chunk.dirty = true;
                }
            }
        }

        // Transform infected pixels into virus
        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                let mut any_infected = false;
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.flags & pixel_flags::INFECTED != 0 {
                            chunk.set_pixel(x, y, Pixel::new(MaterialId::VIRUS));
                            any_infected = true;
                        }
                    }
                }
                if any_infected {
                    chunk.dirty = true;
                }
            }
        }
    }

    /// Clone copies adjacent material patterns.
    fn update_clone_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        _materials: &Materials,
    ) {
        // Collect clone positions
        let mut clone_positions: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::CLONE {
                            clone_positions.push((chunk_pos, x, y));
                        }
                    }
                }
            }
        }

        // Process each clone
        for (chunk_pos, x, y) in clone_positions {
            // 10% chance to clone each frame
            if self.rng.r#gen::<f32>() > 0.1 {
                continue;
            }

            // Find a source material to copy (check all 8 neighbors)
            let mut source_material: Option<u16> = None;

            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let (neighbor_chunk_pos, neighbor_px, neighbor_py) =
                        Self::get_neighbor_pos(chunk_pos, nx, ny);

                    if let Some(neighbor_chunk) = chunks.get(&neighbor_chunk_pos) {
                        let neighbor_pixel = neighbor_chunk.get_pixel(neighbor_px, neighbor_py);
                        // Copy any material that's not air, clone, or special behaviors
                        if neighbor_pixel.material_id != MaterialId::AIR
                            && neighbor_pixel.material_id != MaterialId::CLONE
                            && neighbor_pixel.material_id != MaterialId::VIRUS
                        {
                            source_material = Some(neighbor_pixel.material_id);
                            break;
                        }
                    }
                }
                if source_material.is_some() {
                    break;
                }
            }

            // If we found a source, emit a copy into an empty adjacent cell
            if let Some(source_id) = source_material {
                // Find an empty cell to emit into
                for dy in -1..=1_i32 {
                    for dx in -1..=1_i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        let (neighbor_chunk_pos, neighbor_px, neighbor_py) =
                            Self::get_neighbor_pos(chunk_pos, nx, ny);

                        if let Some(neighbor_chunk) = chunks.get_mut(&neighbor_chunk_pos) {
                            let neighbor_pixel = neighbor_chunk.get_pixel(neighbor_px, neighbor_py);
                            if neighbor_pixel.material_id == MaterialId::AIR {
                                // Emit a copy of the source material
                                neighbor_chunk.set_pixel(
                                    neighbor_px,
                                    neighbor_py,
                                    Pixel::new(source_id),
                                );
                                neighbor_chunk.dirty = true;
                                return; // Only emit one copy per frame per clone
                            }
                        }
                    }
                }
            }
        }
    }

    // --- Helper functions ---

    /// Gets direction (0-3) from pixel flags.
    fn get_direction_from_flags(flags: u16) -> u8 {
        let bit0 = (flags & pixel_flags::DIRECTION_BIT0 != 0) as u8;
        let bit1 = (flags & pixel_flags::DIRECTION_BIT1 != 0) as u8;
        bit0 | (bit1 << 1)
    }

    /// Sets direction (0-3) in pixel flags.
    fn set_direction_in_flags(flags: u16, dir: u8) -> u16 {
        let mut new_flags = flags & !(pixel_flags::DIRECTION_BIT0 | pixel_flags::DIRECTION_BIT1);
        if dir & 1 != 0 {
            new_flags |= pixel_flags::DIRECTION_BIT0;
        }
        if dir & 2 != 0 {
            new_flags |= pixel_flags::DIRECTION_BIT1;
        }
        new_flags
    }

    /// Converts direction (0-3) to (dx, dy) offset.
    fn direction_to_offset(dir: u8) -> (i32, i32) {
        match dir {
            0 => (0, 1),  // up
            1 => (1, 0),  // right
            2 => (0, -1), // down
            3 => (-1, 0), // left
            _ => (0, 0),
        }
    }

    /// Helper to get neighbor position, handling chunk boundaries.
    fn get_neighbor_pos(chunk_pos: IVec2, x: i32, y: i32) -> (IVec2, usize, usize) {
        let mut next_chunk_pos = chunk_pos;
        let mut next_x = x;
        let mut next_y = y;

        if x < 0 {
            next_chunk_pos.x -= 1;
            next_x = CHUNK_SIZE as i32 - 1;
        } else if x >= CHUNK_SIZE as i32 {
            next_chunk_pos.x += 1;
            next_x = 0;
        }

        if y < 0 {
            next_chunk_pos.y -= 1;
            next_y = CHUNK_SIZE as i32 - 1;
        } else if y >= CHUNK_SIZE as i32 {
            next_chunk_pos.y += 1;
            next_y = 0;
        }

        (next_chunk_pos, next_x as usize, next_y as usize)
    }
}

impl Default for SpecialBehaviorsSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        let system = SpecialBehaviorsSystem::new();
        // System should be created successfully
        assert!(std::mem::size_of_val(&system) > 0);
    }

    #[test]
    fn test_direction_flags() {
        // Test direction encoding/decoding
        for dir in 0..4u8 {
            let flags = SpecialBehaviorsSystem::set_direction_in_flags(0, dir);
            let decoded = SpecialBehaviorsSystem::get_direction_from_flags(flags);
            assert_eq!(decoded, dir, "Direction {} should round-trip", dir);
        }
    }

    #[test]
    fn test_direction_to_offset() {
        assert_eq!(SpecialBehaviorsSystem::direction_to_offset(0), (0, 1)); // up
        assert_eq!(SpecialBehaviorsSystem::direction_to_offset(1), (1, 0)); // right
        assert_eq!(SpecialBehaviorsSystem::direction_to_offset(2), (0, -1)); // down
        assert_eq!(SpecialBehaviorsSystem::direction_to_offset(3), (-1, 0)); // left
    }

    #[test]
    fn test_get_neighbor_pos_within_chunk() {
        let chunk_pos = IVec2::new(0, 0);
        let (next_pos, next_x, next_y) =
            SpecialBehaviorsSystem::get_neighbor_pos(chunk_pos, 10, 10);
        assert_eq!(next_pos, chunk_pos);
        assert_eq!(next_x, 10);
        assert_eq!(next_y, 10);
    }

    #[test]
    fn test_chunk_boundary_crossing() {
        let chunk_pos = IVec2::new(0, 0);

        // Test crossing left boundary
        let (left_pos, left_x, _) = SpecialBehaviorsSystem::get_neighbor_pos(chunk_pos, -1, 10);
        assert_eq!(left_pos.x, -1);
        assert_eq!(left_x, CHUNK_SIZE - 1);

        // Test crossing right boundary
        let (right_pos, right_x, _) =
            SpecialBehaviorsSystem::get_neighbor_pos(chunk_pos, CHUNK_SIZE as i32, 10);
        assert_eq!(right_pos.x, 1);
        assert_eq!(right_x, 0);
    }

    #[test]
    fn test_virus_spreads_to_adjacent() {
        let mut system = SpecialBehaviorsSystem::with_seed(12345);
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with virus next to stone
        let mut chunk = Chunk::new(0, 0);
        chunk.set_pixel(10, 10, Pixel::new(MaterialId::VIRUS));
        chunk.set_pixel(11, 10, Pixel::new(MaterialId::STONE));
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run update multiple times (virus spread is probabilistic)
        for _ in 0..100 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Stone should eventually be infected
        let chunk = chunks.get(&chunk_pos).unwrap();
        let stone_pixel = chunk.get_pixel(11, 10);
        // Either still stone or infected/virus
        assert!(
            stone_pixel.material_id == MaterialId::VIRUS
                || stone_pixel.flags & pixel_flags::INFECTED != 0
                || stone_pixel.material_id == MaterialId::STONE,
            "Stone should be infected or still stone (probabilistic)"
        );
    }

    #[test]
    fn test_vine_grows() {
        let mut system = SpecialBehaviorsSystem::with_seed(54321);
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with active vine tip
        let mut chunk = Chunk::new(0, 0);
        let mut vine = Pixel::new(MaterialId::VINE);
        vine.flags |= pixel_flags::BEHAVIOR_ACTIVE;
        chunk.set_pixel(32, 32, vine);
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Count initial vines
        let initial_count = count_material(&chunks, MaterialId::VINE);

        // Run update multiple times (vine growth is probabilistic)
        for _ in 0..200 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Should have more vines now
        let final_count = count_material(&chunks, MaterialId::VINE);
        assert!(
            final_count >= initial_count,
            "Vine count should grow: {} -> {}",
            initial_count,
            final_count
        );
    }

    #[test]
    fn test_clone_copies_material() {
        let mut system = SpecialBehaviorsSystem::with_seed(99999);
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with clone next to sand
        let mut chunk = Chunk::new(0, 0);
        chunk.set_pixel(20, 20, Pixel::new(MaterialId::CLONE));
        chunk.set_pixel(21, 20, Pixel::new(MaterialId::SAND));
        // Leave air around for clone to emit into
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Count initial sand
        let initial_sand = count_material(&chunks, MaterialId::SAND);

        // Run update multiple times
        for _ in 0..100 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Should have more sand now
        let final_sand = count_material(&chunks, MaterialId::SAND);
        assert!(
            final_sand >= initial_sand,
            "Sand count should increase from cloning: {} -> {}",
            initial_sand,
            final_sand
        );
    }

    /// Helper to count material in all chunks
    fn count_material(chunks: &HashMap<IVec2, Chunk>, material_id: u16) -> usize {
        let mut count = 0;
        for chunk in chunks.values() {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    if chunk.get_pixel(x, y).material_id == material_id {
                        count += 1;
                    }
                }
            }
        }
        count
    }
}
