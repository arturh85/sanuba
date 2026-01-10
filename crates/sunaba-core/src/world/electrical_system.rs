//! Manages the propagation of electricity through conductive materials.
// Based on the POWDER_PLAN.md

use super::{CHUNK_SIZE, Chunk};
use glam::IVec2;
use std::collections::VecDeque;
use sunaba_simulation::materials::Materials; // From sunaba-simulation
use sunaba_simulation::pixel::pixel_flags; // From sunaba-simulation // From sunaba-core's own world module

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
