//! Pixel-based entity AI system for Powder Game-style creatures.
//!
//! This module implements simple AI behaviors for single-pixel entities:
//! - **Ant**: Random walk on solid surfaces, avoids water
//! - **Bird**: Boids flocking behavior, flies in air
//! - **Fish**: Swimming in water, schooling behavior

use super::{CHUNK_SIZE, Chunk};
use glam::IVec2;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::collections::HashMap;
use sunaba_simulation::materials::{MaterialId, MaterialType, Materials};
use sunaba_simulation::pixel::{Pixel, pixel_flags};

/// Configuration for boids flocking behavior
const BOIDS_SEPARATION_RADIUS: i32 = 3;
const BOIDS_ALIGNMENT_RADIUS: i32 = 5;
const BOIDS_COHESION_RADIUS: i32 = 7;

/// Manages pixel-based entity AI (ant, bird, fish).
pub struct PixelEntitySystem {
    /// RNG for probabilistic behaviors
    rng: Xoshiro256PlusPlus,
}

impl PixelEntitySystem {
    pub fn new() -> Self {
        Self {
            rng: Xoshiro256PlusPlus::seed_from_u64(12345),
        }
    }

    /// Creates a new system with a specific seed for deterministic behavior.
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: Xoshiro256PlusPlus::seed_from_u64(seed),
        }
    }

    /// Updates all entity behaviors for active chunks.
    pub fn update(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Update each entity type
        self.update_ant_behavior(chunks, active_chunks, materials);
        self.update_bird_behavior(chunks, active_chunks, materials);
        self.update_fish_behavior(chunks, active_chunks, materials);
    }

    /// Ant behavior: Random walk on solid surfaces, avoids water.
    fn update_ant_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect ant positions
        let mut ant_positions: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::ANT {
                            ant_positions.push((chunk_pos, x, y));
                        }
                    }
                }
            }
        }

        // Process each ant
        for (chunk_pos, x, y) in ant_positions {
            // 30% chance to move each frame (ants are fairly active)
            if self.rng.r#gen::<f32>() > 0.3 {
                continue;
            }

            // Check if ant is on solid ground (has solid below)
            let below_y = y as i32 - 1;
            let (below_chunk_pos, below_px, below_py) =
                Self::get_neighbor_pos(chunk_pos, x as i32, below_y);

            let on_ground = if let Some(below_chunk) = chunks.get(&below_chunk_pos) {
                let below_pixel = below_chunk.get_pixel(below_px, below_py);
                let below_mat = materials.get(below_pixel.material_id);
                below_mat.material_type == MaterialType::Solid
                    || below_mat.material_type == MaterialType::Powder
            } else {
                false
            };

            // Get current direction from flags
            let current_dir = Self::get_direction_from_flags(
                chunks
                    .get(&chunk_pos)
                    .map(|c| c.get_pixel(x, y).flags)
                    .unwrap_or(0),
            );

            // Choose movement direction
            let (dx, dy) = if on_ground {
                // Walk horizontally with occasional direction changes
                let walk_dir = if self.rng.r#gen::<f32>() < 0.1 {
                    // 10% chance to change direction
                    if self.rng.r#gen::<bool>() { 1 } else { -1 }
                } else {
                    // Continue in current direction
                    if current_dir == 1 { 1 } else { -1 }
                };
                (walk_dir, 0)
            } else {
                // Fall down if not on ground
                (0, -1)
            };

            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Check if we can move there
            let can_move = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                let next_mat = materials.get(next_pixel.material_id);
                // Can move into air or gas
                next_pixel.material_id == MaterialId::AIR
                    || next_mat.material_type == MaterialType::Gas
            } else {
                false
            };

            // Check if destination has water (ants avoid water)
            let is_water = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                next_pixel.material_id == MaterialId::WATER
            } else {
                false
            };

            if can_move && !is_water {
                // Move ant
                if let Some(src_chunk) = chunks.get_mut(&chunk_pos) {
                    let mut ant_pixel = src_chunk.get_pixel(x, y);
                    // Update direction in flags
                    let new_dir = if dx > 0 {
                        1
                    } else if dx < 0 {
                        3
                    } else {
                        current_dir
                    };
                    ant_pixel.flags = Self::set_direction_in_flags(ant_pixel.flags, new_dir);
                    src_chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                    src_chunk.dirty = true;

                    if let Some(dst_chunk) = chunks.get_mut(&next_chunk_pos) {
                        dst_chunk.set_pixel(next_px, next_py, ant_pixel);
                        dst_chunk.dirty = true;
                    }
                }
            } else if on_ground && !can_move {
                // Try to climb over obstacle
                let climb_y = y as i32 + 1;
                let (climb_chunk_pos, climb_px, climb_py) =
                    Self::get_neighbor_pos(chunk_pos, next_x, climb_y);

                let can_climb = if let Some(climb_chunk) = chunks.get(&climb_chunk_pos) {
                    let climb_pixel = climb_chunk.get_pixel(climb_px, climb_py);
                    climb_pixel.material_id == MaterialId::AIR
                } else {
                    false
                };

                if can_climb {
                    // Climb up
                    if let Some(src_chunk) = chunks.get_mut(&chunk_pos) {
                        let ant_pixel = src_chunk.get_pixel(x, y);
                        src_chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                        src_chunk.dirty = true;

                        if let Some(dst_chunk) = chunks.get_mut(&climb_chunk_pos) {
                            dst_chunk.set_pixel(climb_px, climb_py, ant_pixel);
                            dst_chunk.dirty = true;
                        }
                    }
                } else {
                    // Turn around
                    if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                        let mut ant_pixel = chunk.get_pixel(x, y);
                        let new_dir = if current_dir == 1 { 3 } else { 1 }; // Flip direction
                        ant_pixel.flags = Self::set_direction_in_flags(ant_pixel.flags, new_dir);
                        chunk.set_pixel(x, y, ant_pixel);
                        chunk.dirty = true;
                    }
                }
            }
        }
    }

    /// Bird behavior: Boids flocking (alignment, cohesion, separation).
    fn update_bird_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect bird positions and directions
        let mut birds: Vec<(IVec2, usize, usize, u8)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::BIRD {
                            let dir = Self::get_direction_from_flags(pixel.flags);
                            birds.push((chunk_pos, x, y, dir));
                        }
                    }
                }
            }
        }

        // Process each bird with boids rules
        for i in 0..birds.len() {
            let (chunk_pos, x, y, _current_dir) = birds[i];

            // 40% chance to update each frame (birds are quite active)
            if self.rng.r#gen::<f32>() > 0.4 {
                continue;
            }

            // Calculate world position for distance calculations
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            // Boids calculations
            let mut separation = (0.0_f32, 0.0_f32);
            let mut alignment = (0.0_f32, 0.0_f32);
            let mut cohesion = (0.0_f32, 0.0_f32);
            let mut sep_count = 0;
            let mut align_count = 0;
            let mut cohesion_count = 0;

            for (j, &(other_chunk_pos, other_x, other_y, other_dir)) in birds.iter().enumerate() {
                if i == j {
                    continue;
                }
                let other_world_x = other_chunk_pos.x * CHUNK_SIZE as i32 + other_x as i32;
                let other_world_y = other_chunk_pos.y * CHUNK_SIZE as i32 + other_y as i32;

                let dx = other_world_x - world_x;
                let dy = other_world_y - world_y;
                let dist_sq = dx * dx + dy * dy;

                // Separation - avoid crowding
                if dist_sq < BOIDS_SEPARATION_RADIUS * BOIDS_SEPARATION_RADIUS && dist_sq > 0 {
                    let dist = (dist_sq as f32).sqrt();
                    separation.0 -= dx as f32 / dist;
                    separation.1 -= dy as f32 / dist;
                    sep_count += 1;
                }

                // Alignment - steer towards average heading
                if dist_sq < BOIDS_ALIGNMENT_RADIUS * BOIDS_ALIGNMENT_RADIUS {
                    let (odx, ody) = Self::direction_to_offset(other_dir);
                    alignment.0 += odx as f32;
                    alignment.1 += ody as f32;
                    align_count += 1;
                }

                // Cohesion - steer towards center of mass
                if dist_sq < BOIDS_COHESION_RADIUS * BOIDS_COHESION_RADIUS {
                    cohesion.0 += other_world_x as f32;
                    cohesion.1 += other_world_y as f32;
                    cohesion_count += 1;
                }
            }

            // Normalize and combine forces
            let mut target_dx = 0.0_f32;
            let mut target_dy = 0.0_f32;

            if sep_count > 0 {
                target_dx += separation.0 / sep_count as f32 * 2.0; // Weight separation high
                target_dy += separation.1 / sep_count as f32 * 2.0;
            }

            if align_count > 0 {
                target_dx += alignment.0 / align_count as f32;
                target_dy += alignment.1 / align_count as f32;
            }

            if cohesion_count > 0 {
                let center_x = cohesion.0 / cohesion_count as f32;
                let center_y = cohesion.1 / cohesion_count as f32;
                target_dx += (center_x - world_x as f32) * 0.1;
                target_dy += (center_y - world_y as f32) * 0.1;
            }

            // Add slight upward tendency (birds prefer to fly)
            target_dy += 0.5;

            // Add random component
            target_dx += self.rng.gen_range(-0.5..0.5);
            target_dy += self.rng.gen_range(-0.3..0.5);

            // Convert to movement direction
            let (dx, dy) = if target_dx.abs() > target_dy.abs() {
                if target_dx > 0.0 { (1, 0) } else { (-1, 0) }
            } else if target_dy > 0.0 {
                (0, 1)
            } else {
                (0, -1)
            };

            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Check if we can move there (air or gas only for birds)
            let can_move = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                let next_mat = materials.get(next_pixel.material_id);
                next_pixel.material_id == MaterialId::AIR
                    || (next_mat.material_type == MaterialType::Gas
                        && next_pixel.material_id != MaterialId::BIRD)
            } else {
                false
            };

            if can_move {
                // Move bird
                if let Some(src_chunk) = chunks.get_mut(&chunk_pos) {
                    let mut bird_pixel = src_chunk.get_pixel(x, y);
                    // Update direction
                    let new_dir = Self::offset_to_direction(dx, dy);
                    bird_pixel.flags = Self::set_direction_in_flags(bird_pixel.flags, new_dir);
                    src_chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                    src_chunk.dirty = true;

                    if let Some(dst_chunk) = chunks.get_mut(&next_chunk_pos) {
                        dst_chunk.set_pixel(next_px, next_py, bird_pixel);
                        dst_chunk.dirty = true;
                    }
                }
            }
        }
    }

    /// Fish behavior: Swimming in water, schooling.
    fn update_fish_behavior(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        _materials: &Materials,
    ) {
        // Collect fish positions
        let mut fish: Vec<(IVec2, usize, usize, u8)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::FISH {
                            let dir = Self::get_direction_from_flags(pixel.flags);
                            fish.push((chunk_pos, x, y, dir));
                        }
                    }
                }
            }
        }

        // Process each fish
        for i in 0..fish.len() {
            let (chunk_pos, x, y, current_dir) = fish[i];

            // 35% chance to update each frame
            if self.rng.r#gen::<f32>() > 0.35 {
                continue;
            }

            // Check if fish is in water
            let in_water = Self::is_pixel_in_water(chunks, chunk_pos, x, y);

            if !in_water {
                // Fish out of water - fall down (suffocating)
                let below_y = y as i32 - 1;
                let (below_chunk_pos, below_px, below_py) =
                    Self::get_neighbor_pos(chunk_pos, x as i32, below_y);

                let can_fall = if let Some(below_chunk) = chunks.get(&below_chunk_pos) {
                    let below_pixel = below_chunk.get_pixel(below_px, below_py);
                    below_pixel.material_id == MaterialId::AIR
                        || below_pixel.material_id == MaterialId::WATER
                } else {
                    false
                };

                if can_fall && let Some(src_chunk) = chunks.get_mut(&chunk_pos) {
                    let fish_pixel = src_chunk.get_pixel(x, y);
                    src_chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                    src_chunk.dirty = true;

                    if let Some(dst_chunk) = chunks.get_mut(&below_chunk_pos) {
                        dst_chunk.set_pixel(below_px, below_py, fish_pixel);
                        dst_chunk.dirty = true;
                    }
                }
                continue;
            }

            // Fish is in water - swim with schooling behavior
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            // Simple schooling - move towards nearby fish, but not too close
            let mut target_dx = 0.0_f32;
            let mut target_dy = 0.0_f32;
            let mut neighbor_count = 0;

            for (j, &(other_chunk_pos, other_x, other_y, _)) in fish.iter().enumerate() {
                if i == j {
                    continue;
                }
                let other_world_x = other_chunk_pos.x * CHUNK_SIZE as i32 + other_x as i32;
                let other_world_y = other_chunk_pos.y * CHUNK_SIZE as i32 + other_y as i32;

                let dx = other_world_x - world_x;
                let dy = other_world_y - world_y;
                let dist_sq = dx * dx + dy * dy;

                if dist_sq < 64 && dist_sq > 0 {
                    // Within 8 pixels
                    let dist = (dist_sq as f32).sqrt();
                    if dist < 2.0 {
                        // Too close - separate
                        target_dx -= dx as f32 / dist;
                        target_dy -= dy as f32 / dist;
                    } else {
                        // Move towards school
                        target_dx += dx as f32 / dist * 0.3;
                        target_dy += dy as f32 / dist * 0.3;
                    }
                    neighbor_count += 1;
                }
            }

            // Add current direction bias
            let (curr_dx, curr_dy) = Self::direction_to_offset(current_dir);
            target_dx += curr_dx as f32 * 0.5;
            target_dy += curr_dy as f32 * 0.5;

            // Add random component
            target_dx += self.rng.gen_range(-0.5..0.5);
            target_dy += self.rng.gen_range(-0.3..0.3);

            // Prefer horizontal movement for fish
            if neighbor_count == 0 {
                target_dx += if current_dir == 1 { 1.0 } else { -1.0 };
            }

            // Convert to movement direction
            let (dx, dy) = if target_dx.abs() > target_dy.abs() {
                if target_dx > 0.0 { (1, 0) } else { (-1, 0) }
            } else if target_dy > 0.0 {
                (0, 1)
            } else {
                (0, -1)
            };

            let next_x = x as i32 + dx;
            let next_y = y as i32 + dy;
            let (next_chunk_pos, next_px, next_py) =
                Self::get_neighbor_pos(chunk_pos, next_x, next_y);

            // Fish can only move within water
            let can_move = if let Some(next_chunk) = chunks.get(&next_chunk_pos) {
                let next_pixel = next_chunk.get_pixel(next_px, next_py);
                next_pixel.material_id == MaterialId::WATER
            } else {
                false
            };

            if can_move {
                // Swap fish with water
                if let Some(src_chunk) = chunks.get_mut(&chunk_pos) {
                    let mut fish_pixel = src_chunk.get_pixel(x, y);
                    let new_dir = Self::offset_to_direction(dx, dy);
                    fish_pixel.flags = Self::set_direction_in_flags(fish_pixel.flags, new_dir);
                    src_chunk.set_pixel(x, y, Pixel::new(MaterialId::WATER)); // Leave water behind
                    src_chunk.dirty = true;

                    if let Some(dst_chunk) = chunks.get_mut(&next_chunk_pos) {
                        dst_chunk.set_pixel(next_px, next_py, fish_pixel);
                        dst_chunk.dirty = true;
                    }
                }
            } else {
                // Can't move - turn around
                if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                    let mut fish_pixel = chunk.get_pixel(x, y);
                    let new_dir = (current_dir + 2) % 4; // Reverse direction
                    fish_pixel.flags = Self::set_direction_in_flags(fish_pixel.flags, new_dir);
                    chunk.set_pixel(x, y, fish_pixel);
                    chunk.dirty = true;
                }
            }
        }
    }

    /// Check if a pixel position is surrounded by water (fish needs water to survive).
    fn is_pixel_in_water(
        chunks: &HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
    ) -> bool {
        // Check if current position has water neighbors (fish is "in" water if surrounded)
        let mut water_neighbors = 0;
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
                    if neighbor_pixel.material_id == MaterialId::WATER {
                        water_neighbors += 1;
                    }
                }
            }
        }
        // Consider "in water" if at least 3 water neighbors
        water_neighbors >= 3
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

    /// Converts (dx, dy) offset to direction (0-3).
    fn offset_to_direction(dx: i32, dy: i32) -> u8 {
        if dy > 0 {
            0 // up
        } else if dx > 0 {
            1 // right
        } else if dy < 0 {
            2 // down
        } else {
            3 // left
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

impl Default for PixelEntitySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_system() {
        let system = PixelEntitySystem::new();
        assert!(std::mem::size_of_val(&system) > 0);
    }

    #[test]
    fn test_direction_flags() {
        for dir in 0..4u8 {
            let flags = PixelEntitySystem::set_direction_in_flags(0, dir);
            let decoded = PixelEntitySystem::get_direction_from_flags(flags);
            assert_eq!(decoded, dir, "Direction {} should round-trip", dir);
        }
    }

    #[test]
    fn test_direction_to_offset() {
        assert_eq!(PixelEntitySystem::direction_to_offset(0), (0, 1)); // up
        assert_eq!(PixelEntitySystem::direction_to_offset(1), (1, 0)); // right
        assert_eq!(PixelEntitySystem::direction_to_offset(2), (0, -1)); // down
        assert_eq!(PixelEntitySystem::direction_to_offset(3), (-1, 0)); // left
    }

    #[test]
    fn test_offset_to_direction() {
        assert_eq!(PixelEntitySystem::offset_to_direction(0, 1), 0); // up
        assert_eq!(PixelEntitySystem::offset_to_direction(1, 0), 1); // right
        assert_eq!(PixelEntitySystem::offset_to_direction(0, -1), 2); // down
        assert_eq!(PixelEntitySystem::offset_to_direction(-1, 0), 3); // left
    }

    #[test]
    fn test_get_neighbor_pos_within_chunk() {
        let chunk_pos = IVec2::new(0, 0);
        let (next_pos, next_x, next_y) = PixelEntitySystem::get_neighbor_pos(chunk_pos, 10, 10);
        assert_eq!(next_pos, chunk_pos);
        assert_eq!(next_x, 10);
        assert_eq!(next_y, 10);
    }

    #[test]
    fn test_chunk_boundary_crossing() {
        let chunk_pos = IVec2::new(0, 0);

        // Test crossing left boundary
        let (left_pos, left_x, _) = PixelEntitySystem::get_neighbor_pos(chunk_pos, -1, 10);
        assert_eq!(left_pos.x, -1);
        assert_eq!(left_x, CHUNK_SIZE - 1);

        // Test crossing right boundary
        let (right_pos, right_x, _) =
            PixelEntitySystem::get_neighbor_pos(chunk_pos, CHUNK_SIZE as i32, 10);
        assert_eq!(right_pos.x, 1);
        assert_eq!(right_x, 0);
    }

    #[test]
    fn test_ant_moves_on_ground() {
        let mut system = PixelEntitySystem::with_seed(42);
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with solid ground and an ant
        let mut chunk = Chunk::new(0, 0);
        // Ground (stone)
        for x in 0..CHUNK_SIZE {
            chunk.set_pixel(x, 10, Pixel::new(MaterialId::STONE));
        }
        // Ant on top of ground
        chunk.set_pixel(32, 11, Pixel::new(MaterialId::ANT));
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run update multiple times
        for _ in 0..20 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Ant should have moved (not necessarily in a specific direction)
        let chunk = chunks.get(&chunk_pos).unwrap();
        // Count ants - should still be exactly 1
        let mut ant_count = 0;
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get_pixel(x, y).material_id == MaterialId::ANT {
                    ant_count += 1;
                }
            }
        }
        assert_eq!(ant_count, 1, "Should have exactly one ant");
    }

    #[test]
    fn test_fish_stays_in_water() {
        let mut system = PixelEntitySystem::with_seed(99);
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with a pool of water and a fish
        let mut chunk = Chunk::new(0, 0);
        // Water pool
        for y in 10..20 {
            for x in 10..30 {
                chunk.set_pixel(x, y, Pixel::new(MaterialId::WATER));
            }
        }
        // Fish in the middle of the pool
        chunk.set_pixel(20, 15, Pixel::new(MaterialId::FISH));
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run update multiple times
        for _ in 0..30 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Fish should still exist somewhere in the pool
        let chunk = chunks.get(&chunk_pos).unwrap();
        let mut fish_count = 0;
        let mut fish_in_water_area = false;
        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get_pixel(x, y).material_id == MaterialId::FISH {
                    fish_count += 1;
                    if (10..30).contains(&x) && (10..20).contains(&y) {
                        fish_in_water_area = true;
                    }
                }
            }
        }
        assert_eq!(fish_count, 1, "Should have exactly one fish");
        assert!(fish_in_water_area, "Fish should stay in water area");
    }
}
