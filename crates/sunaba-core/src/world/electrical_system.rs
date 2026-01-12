//! Manages the propagation of electricity through conductive materials.
// Based on the POWDER_PLAN.md

use super::{CHUNK_SIZE, Chunk};
use glam::IVec2;
use std::collections::VecDeque;
use sunaba_simulation::materials::{MaterialId, Materials};
use sunaba_simulation::pixel::{Pixel, pixel_flags};

const PROPAGATION_QUEUE_MAX: usize = 256;

/// Manages the electrical simulation.
pub struct ElectricalSystem {
    /// Queue of pixels to update for electrical propagation.
    propagation_queue: VecDeque<(IVec2, usize, usize)>,
}

impl ElectricalSystem {
    pub fn new() -> Self {
        Self {
            propagation_queue: VecDeque::with_capacity(PROPAGATION_QUEUE_MAX),
        }
    }

    /// Updates the electrical state for all active chunks.
    pub fn update(
        &mut self,
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // 1. Deplete power sources and discharge powered pixels
        self.discharge_and_deplete(chunks, active_chunks, materials);

        // 2. Add power from active sources to the grid
        self.add_power_from_sources(chunks, active_chunks, materials);

        // 3. Propagate power through conductors
        self.propagate_power(chunks, materials);

        // 4. Handle special behaviors (sparks, thunder) and generate heat
        self.handle_effects(chunks, active_chunks, materials);
    }

    /// Reduces power level of powered pixels and batteries.
    fn discharge_and_deplete(
        &self,
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                let mut needs_update = false;
                for coarse_y in 0..8 {
                    for coarse_x in 0..8 {
                        let coarse_idx = coarse_y * 8 + coarse_x;
                        if chunk.electrical_potential[coarse_idx] > 0.0 {
                            // Sample one pixel from this coarse cell to get decay rate
                            let pixel_x = coarse_x * 8;
                            let pixel_y = coarse_y * 8;
                            let pixel = chunk.get_pixel(pixel_x, pixel_y);
                            let material_def = materials.get(pixel.material_id);
                            let decay = material_def.power_decay_rate.max(0.01);
                            chunk.electrical_potential[coarse_idx] =
                                (chunk.electrical_potential[coarse_idx] - decay).max(0.0);

                            // Clear POWERED flag if potential is zero
                            if chunk.electrical_potential[coarse_idx] == 0.0 {
                                for dy in 0..8 {
                                    for dx in 0..8 {
                                        let px = coarse_x * 8 + dx;
                                        let py = coarse_y * 8 + dy;
                                        if px < CHUNK_SIZE && py < CHUNK_SIZE {
                                            let mut p = chunk.get_pixel(px, py);
                                            p.flags &= !pixel_flags::POWERED;
                                            chunk.set_pixel(px, py, p);
                                        }
                                    }
                                }
                            }
                            needs_update = true;
                        }
                    }
                }
                if needs_update {
                    chunk.dirty = true;
                }
            }
        }
    }

    /// Adds power from batteries and other sources.
    fn add_power_from_sources(
        &mut self,
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.flags & pixel_flags::SPARK_SOURCE != 0 {
                            let material_def = materials.get(pixel.material_id);
                            if material_def.power_generation > 0.0 {
                                let coarse_idx = chunk.get_coarse_grid_index(x, y);
                                let current_potential = chunk.electrical_potential[coarse_idx];
                                let new_potential =
                                    (current_potential + material_def.power_generation).min(100.0);
                                chunk.electrical_potential[coarse_idx] = new_potential;

                                let mut updated_pixel = pixel;
                                updated_pixel.flags |= pixel_flags::POWERED;
                                chunk.set_pixel(x, y, updated_pixel);

                                if self.propagation_queue.len() < PROPAGATION_QUEUE_MAX {
                                    self.propagation_queue.push_back((chunk_pos, x, y));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Propagates electricity through the grid.
    fn propagate_power(
        &mut self,
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        materials: &Materials,
    ) {
        let mut depth = 0;
        const MAX_DEPTH_PER_FRAME: usize = 128;

        while let Some((chunk_pos, x, y)) = self.propagation_queue.pop_front() {
            if depth > MAX_DEPTH_PER_FRAME {
                break;
            }
            depth += 1;

            let source_potential = {
                let chunk = chunks.get(&chunk_pos).unwrap();
                let coarse_idx = chunk.get_coarse_grid_index(x, y);
                chunk.electrical_potential[coarse_idx]
            };

            if source_potential <= 0.0 {
                continue;
            }

            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let (next_chunk_pos, next_x, next_y) =
                        Self::get_neighbor_pos(chunk_pos, nx, ny);

                    if let Some(neighbor_chunk) = chunks.get_mut(&next_chunk_pos) {
                        let neighbor_pixel = neighbor_chunk.get_pixel(next_x, next_y);
                        let neighbor_material = materials.get(neighbor_pixel.material_id);

                        if neighbor_material.conducts_electricity {
                            let neighbor_coarse_idx =
                                neighbor_chunk.get_coarse_grid_index(next_x, next_y);
                            let neighbor_potential =
                                neighbor_chunk.electrical_potential[neighbor_coarse_idx];

                            let transfer_amount = (source_potential - neighbor_potential)
                                * neighbor_material.electrical_conductivity
                                * 0.5;

                            if transfer_amount > 0.01 {
                                neighbor_chunk.electrical_potential[neighbor_coarse_idx] +=
                                    transfer_amount;
                                let mut updated_pixel = neighbor_pixel;
                                updated_pixel.flags |= pixel_flags::POWERED;
                                neighbor_chunk.set_pixel(next_x, next_y, updated_pixel);
                                neighbor_chunk.dirty = true;

                                if self.propagation_queue.len() < PROPAGATION_QUEUE_MAX {
                                    self.propagation_queue.push_back((
                                        next_chunk_pos,
                                        next_x,
                                        next_y,
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        self.propagation_queue.clear();
    }

    /// Generates heat and other effects from electricity.
    fn handle_effects(
        &self,
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Generate heat from electrical resistance
        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let coarse_idx = chunk.get_coarse_grid_index(x, y);
                        let potential = chunk.electrical_potential[coarse_idx];
                        if potential > 0.0 {
                            let pixel = chunk.get_pixel(x, y);
                            let material = materials.get(pixel.material_id);
                            if material.electrical_resistance > 0.0 {
                                let heat_generated =
                                    potential * material.electrical_resistance * 0.1;
                                let temp_idx = chunk.get_coarse_grid_index(x, y);
                                chunk.temperature[temp_idx] += heat_generated;
                            }
                        }
                    }
                }
            }
        }

        // Handle spark drift behavior
        Self::update_spark_behavior(chunks, active_chunks, materials);

        // Handle thunder destruction behavior
        Self::update_thunder_behavior(chunks, active_chunks, materials);
    }

    /// Moves spark pixels toward nearby conductors.
    /// Spark is a gas that drifts toward powered conductors and can jump small air gaps.
    fn update_spark_behavior(
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect spark positions first to avoid borrow issues
        let mut spark_positions: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::SPARK {
                            spark_positions.push((chunk_pos, x, y));
                        }
                    }
                }
            }
        }

        // Process each spark
        for (chunk_pos, x, y) in spark_positions {
            // Find best direction to move (toward nearest conductor with power)
            let mut best_dir: Option<(i32, i32)> = None;
            let mut found_conductor = false;

            // Check all 8 neighbors for conductors
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let (neighbor_chunk_pos, neighbor_x, neighbor_y) =
                        Self::get_neighbor_pos(chunk_pos, nx, ny);

                    if let Some(neighbor_chunk) = chunks.get(&neighbor_chunk_pos) {
                        let neighbor_pixel = neighbor_chunk.get_pixel(neighbor_x, neighbor_y);
                        let neighbor_material = materials.get(neighbor_pixel.material_id);

                        // Prioritize moving toward powered conductors
                        if neighbor_material.conducts_electricity {
                            let coarse_idx =
                                neighbor_chunk.get_coarse_grid_index(neighbor_x, neighbor_y);
                            let potential = neighbor_chunk.electrical_potential[coarse_idx];

                            if potential > 0.0 && !found_conductor {
                                best_dir = Some((dx, dy));
                                found_conductor = true;
                            }
                        }
                        // If no conductor found, try to move into air (spark drifts)
                        else if neighbor_pixel.material_id == MaterialId::AIR
                            && best_dir.is_none()
                        {
                            // Prefer moving right or up (gas behavior + horizontal drift)
                            if dy >= 0 {
                                best_dir = Some((dx, dy));
                            }
                        }
                    }
                }
            }

            // Move spark if we found a direction
            if let Some((dx, dy)) = best_dir {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                let (target_chunk_pos, target_x, target_y) =
                    Self::get_neighbor_pos(chunk_pos, nx, ny);

                // Get target pixel
                let target_pixel = if let Some(target_chunk) = chunks.get(&target_chunk_pos) {
                    target_chunk.get_pixel(target_x, target_y)
                } else {
                    continue;
                };

                // Only move into air or onto conductors
                let target_material = materials.get(target_pixel.material_id);
                if target_pixel.material_id == MaterialId::AIR
                    || target_material.conducts_electricity
                {
                    // Swap spark with target (or merge if conductor)
                    if let Some(source_chunk) = chunks.get_mut(&chunk_pos) {
                        let spark_pixel = source_chunk.get_pixel(x, y);
                        source_chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                        source_chunk.dirty = true;

                        if let Some(target_chunk) = chunks.get_mut(&target_chunk_pos) {
                            if target_material.conducts_electricity {
                                // Spark reaches conductor - add power and consume spark
                                let coarse_idx =
                                    target_chunk.get_coarse_grid_index(target_x, target_y);
                                target_chunk.electrical_potential[coarse_idx] += 10.0;
                                let mut updated_target = target_pixel;
                                updated_target.flags |= pixel_flags::POWERED;
                                target_chunk.set_pixel(target_x, target_y, updated_target);
                            } else {
                                // Move spark into air
                                target_chunk.set_pixel(target_x, target_y, spark_pixel);
                            }
                            target_chunk.dirty = true;
                        }
                    }
                }
            }
        }
    }

    /// Destroys non-conductive materials adjacent to powered thunder.
    /// Thunder is instant electrical destruction - when powered, it destroys
    /// all adjacent non-conductors and then consumes itself.
    fn update_thunder_behavior(
        chunks: &mut std::collections::HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // Collect thunder positions first to avoid borrow issues
        let mut thunder_positions: Vec<(IVec2, usize, usize)> = Vec::new();

        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        if pixel.material_id == MaterialId::THUNDER {
                            // Check if thunder is powered or adjacent to powered conductor
                            let coarse_idx = chunk.get_coarse_grid_index(x, y);
                            let is_powered = pixel.flags & pixel_flags::POWERED != 0
                                || chunk.electrical_potential[coarse_idx] > 0.0;

                            if is_powered {
                                thunder_positions.push((chunk_pos, x, y));
                            }
                        }
                    }
                }
            }
        }

        // Process each powered thunder
        for (chunk_pos, x, y) in thunder_positions {
            // Destroy all 8-connected non-conductive neighbors
            for dy in -1..=1_i32 {
                for dx in -1..=1_i32 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let (neighbor_chunk_pos, neighbor_x, neighbor_y) =
                        Self::get_neighbor_pos(chunk_pos, nx, ny);

                    if let Some(neighbor_chunk) = chunks.get_mut(&neighbor_chunk_pos) {
                        let neighbor_pixel = neighbor_chunk.get_pixel(neighbor_x, neighbor_y);
                        let neighbor_material = materials.get(neighbor_pixel.material_id);

                        // Destroy non-conductors (but not air, thunder, or laser)
                        if !neighbor_material.conducts_electricity
                            && neighbor_pixel.material_id != MaterialId::AIR
                            && neighbor_pixel.material_id != MaterialId::THUNDER
                            && neighbor_pixel.material_id != MaterialId::LASER
                        {
                            neighbor_chunk.set_pixel(
                                neighbor_x,
                                neighbor_y,
                                Pixel::new(MaterialId::AIR),
                            );
                            neighbor_chunk.dirty = true;
                        }
                    }
                }
            }

            // Generate lots of heat from thunder
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                let coarse_idx = chunk.get_coarse_grid_index(x, y);
                chunk.temperature[coarse_idx] += 500.0; // Thunder is very hot

                // Consume thunder itself
                chunk.set_pixel(x, y, Pixel::new(MaterialId::AIR));
                chunk.dirty = true;
            }
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

impl Default for ElectricalSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{MaterialId, Materials, Pixel};
    use std::collections::HashMap;

    #[test]
    fn test_new_system() {
        let system = ElectricalSystem::new();
        assert!(
            system.propagation_queue.is_empty(),
            "New system should have empty queue"
        );
    }

    #[test]
    fn test_power_source_adds_potential() {
        let mut system = ElectricalSystem::new();
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with a battery (SPARK_SOURCE flag set)
        let mut chunk = Chunk::new(0, 0);
        let mut battery_pixel = Pixel::new(MaterialId::BATTERY);
        battery_pixel.flags |= pixel_flags::SPARK_SOURCE;
        chunk.set_pixel(10, 10, battery_pixel);
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run update - battery should add power to the grid
        system.update(&mut chunks, &active_chunks, &materials);

        // Check that electrical potential increased
        let chunk = chunks.get(&chunk_pos).unwrap();
        let coarse_idx = chunk.get_coarse_grid_index(10, 10);
        assert!(
            chunk.electrical_potential[coarse_idx] > 0.0,
            "Battery should add electrical potential"
        );
    }

    #[test]
    fn test_propagation_between_conductors() {
        let mut system = ElectricalSystem::new();
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with a powered battery and adjacent wire
        // NOTE: Place them in DIFFERENT coarse cells (8x8 grid) for propagation to work
        // Battery at (0,0) -> coarse cell 0
        // Wire at (8,0) -> coarse cell 1 (different cell, will receive propagated power)
        let mut chunk = Chunk::new(0, 0);

        // Place battery with SPARK_SOURCE flag at edge of coarse cell 0
        let mut battery_pixel = Pixel::new(MaterialId::BATTERY);
        battery_pixel.flags |= pixel_flags::SPARK_SOURCE;
        chunk.set_pixel(7, 0, battery_pixel);

        // Place wire in adjacent coarse cell
        chunk.set_pixel(8, 0, Pixel::new(MaterialId::WIRE));
        chunk.set_pixel(9, 0, Pixel::new(MaterialId::WIRE));

        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run update multiple times to allow propagation
        for _ in 0..5 {
            system.update(&mut chunks, &active_chunks, &materials);
        }

        // Check that wire's coarse cell received power
        let chunk = chunks.get(&chunk_pos).unwrap();
        let wire_coarse_idx = chunk.get_coarse_grid_index(8, 0);
        assert!(
            chunk.electrical_potential[wire_coarse_idx] > 0.0,
            "Wire coarse cell should receive propagated power"
        );
    }

    #[test]
    fn test_discharge_reduces_potential() {
        let system = ElectricalSystem::new();
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with pre-set high potential (no battery, just existing charge)
        let mut chunk = Chunk::new(0, 0);
        // Place wire (conductive) in the area
        chunk.set_pixel(0, 0, Pixel::new(MaterialId::WIRE));
        // Set initial high potential
        chunk.electrical_potential[0] = 50.0;
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run discharge
        system.discharge_and_deplete(&mut chunks, &active_chunks, &materials);

        // Check that potential decreased
        let chunk = chunks.get(&chunk_pos).unwrap();
        assert!(
            chunk.electrical_potential[0] < 50.0,
            "Electrical potential should decay over time"
        );
    }

    #[test]
    fn test_heat_generation_from_resistance() {
        let system = ElectricalSystem::new();
        let materials = Materials::new();
        let mut chunks = HashMap::new();
        let chunk_pos = IVec2::new(0, 0);

        // Create a chunk with powered wire that has resistance
        let mut chunk = Chunk::new(0, 0);
        let mut wire = Pixel::new(MaterialId::WIRE);
        wire.flags |= pixel_flags::POWERED;
        chunk.set_pixel(0, 0, wire);
        // Set high potential for significant heat generation
        chunk.electrical_potential[0] = 50.0;
        let initial_temp = chunk.temperature[0];
        chunks.insert(chunk_pos, chunk);
        let active_chunks = vec![chunk_pos];

        // Run effects (which generates heat)
        system.handle_effects(&mut chunks, &active_chunks, &materials);

        // Check that temperature increased
        let chunk = chunks.get(&chunk_pos).unwrap();
        assert!(
            chunk.temperature[0] > initial_temp,
            "Electrical resistance should generate heat"
        );
    }

    #[test]
    fn test_get_neighbor_pos_within_chunk() {
        let chunk_pos = IVec2::new(0, 0);

        // Test normal case within chunk
        let (next_pos, next_x, next_y) = ElectricalSystem::get_neighbor_pos(chunk_pos, 10, 10);
        assert_eq!(next_pos, chunk_pos);
        assert_eq!(next_x, 10);
        assert_eq!(next_y, 10);
    }

    #[test]
    fn test_chunk_boundary_crossing() {
        let chunk_pos = IVec2::new(0, 0);

        // Test crossing left boundary
        let (left_pos, left_x, _) = ElectricalSystem::get_neighbor_pos(chunk_pos, -1, 10);
        assert_eq!(left_pos.x, -1, "Should move to left chunk");
        assert_eq!(left_x, CHUNK_SIZE - 1, "X should wrap to last pixel");

        // Test crossing right boundary
        let (right_pos, right_x, _) =
            ElectricalSystem::get_neighbor_pos(chunk_pos, CHUNK_SIZE as i32, 10);
        assert_eq!(right_pos.x, 1, "Should move to right chunk");
        assert_eq!(right_x, 0, "X should wrap to first pixel");

        // Test crossing bottom boundary
        let (bottom_pos, _, bottom_y) = ElectricalSystem::get_neighbor_pos(chunk_pos, 10, -1);
        assert_eq!(bottom_pos.y, -1, "Should move to bottom chunk");
        assert_eq!(bottom_y, CHUNK_SIZE - 1, "Y should wrap to last pixel");

        // Test crossing top boundary
        let (top_pos, _, top_y) =
            ElectricalSystem::get_neighbor_pos(chunk_pos, 10, CHUNK_SIZE as i32);
        assert_eq!(top_pos.y, 1, "Should move to top chunk");
        assert_eq!(top_y, 0, "Y should wrap to first pixel");
    }
}
