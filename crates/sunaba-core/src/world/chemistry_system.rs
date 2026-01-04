//! Chemistry system for fire, burning, ignition, and chemical reactions

use glam::IVec2;
use std::collections::HashMap;

use super::{CHUNK_SIZE, Chunk, pixel_flags};
use crate::simulation::{
    MaterialId, Materials, StateChangeSystem, add_heat_at_pixel, get_temperature_at_pixel,
};
use crate::world::{SimStats, WorldRng};

use super::ca_update::CellularAutomataUpdater;

/// Handles chemistry simulation: fire, burning, ignition, and reactions
pub struct ChemistrySystem;

impl ChemistrySystem {
    /// Check all pixels in a chunk for state changes based on temperature
    pub fn check_chunk_state_changes(
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        materials: &Materials,
        stats: &mut dyn SimStats,
    ) {
        let chunk = match chunks.get_mut(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel = chunk.get_pixel(x, y);
                if pixel.is_empty() {
                    continue;
                }

                let material = materials.get(pixel.material_id);
                let temp = get_temperature_at_pixel(chunk, x, y);

                let mut new_pixel = pixel;
                if StateChangeSystem::check_state_change(&mut new_pixel, material, temp) {
                    chunk.set_pixel(x, y, new_pixel);
                    stats.record_state_change();
                }
            }
        }
    }

    /// Update fire pixel behavior
    pub fn update_fire<R: WorldRng>(
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
        materials: &Materials,
        stats: &mut dyn SimStats,
        rng: &mut R,
    ) {
        // 1. Add heat to temperature field
        if let Some(chunk) = chunks.get_mut(&chunk_pos) {
            add_heat_at_pixel(chunk, x, y, 50.0); // Fire adds significant heat
        }

        // 2. Fire behaves like gas (rises)
        CellularAutomataUpdater::update_gas(chunks, chunk_pos, x, y, materials, stats, rng);

        // 3. Fire has limited lifetime - random chance to become smoke
        if rng.check_probability(0.02) {
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            // Set pixel directly in chunks
            let target_chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
            let target_chunk_y = world_y.div_euclid(CHUNK_SIZE as i32);
            let target_chunk_pos = IVec2::new(target_chunk_x, target_chunk_y);

            if let Some(target_chunk) = chunks.get_mut(&target_chunk_pos) {
                let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
                target_chunk.set_pixel(local_x, local_y, super::Pixel::new(MaterialId::SMOKE));
            }
        }
    }

    /// Check if a pixel should ignite based on temperature
    pub fn check_ignition(
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
        materials: &Materials,
    ) {
        let chunk = match chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        let material = materials.get(pixel.material_id);

        if !material.flammable {
            return;
        }

        let temp = get_temperature_at_pixel(chunk, x, y);

        if let Some(ignition_temp) = material.ignition_temp
            && temp >= ignition_temp
        {
            // Mark pixel as burning
            let chunk = chunks.get_mut(&chunk_pos).unwrap();
            let mut new_pixel = pixel;
            new_pixel.flags |= pixel_flags::BURNING;
            chunk.set_pixel(x, y, new_pixel);

            // Try to spawn fire in adjacent air cell
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            for (dx, dy) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                let neighbor_x = world_x + dx;
                let neighbor_y = world_y + dy;

                // Get neighbor pixel from chunks
                let neighbor_chunk_x = neighbor_x.div_euclid(CHUNK_SIZE as i32);
                let neighbor_chunk_y = neighbor_y.div_euclid(CHUNK_SIZE as i32);
                let neighbor_chunk_pos = IVec2::new(neighbor_chunk_x, neighbor_chunk_y);

                if let Some(neighbor_chunk) = chunks.get(&neighbor_chunk_pos) {
                    let local_x = neighbor_x.rem_euclid(CHUNK_SIZE as i32) as usize;
                    let local_y = neighbor_y.rem_euclid(CHUNK_SIZE as i32) as usize;
                    let neighbor_pixel = neighbor_chunk.get_pixel(local_x, local_y);

                    if neighbor_pixel.is_empty() {
                        // Set fire pixel
                        if let Some(target_chunk) = chunks.get_mut(&neighbor_chunk_pos) {
                            target_chunk.set_pixel(
                                local_x,
                                local_y,
                                super::Pixel::new(MaterialId::FIRE),
                            );
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Update burning material (gradual consumption)
    pub fn update_burning_material<R: WorldRng>(
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
        materials: &Materials,
        rng: &mut R,
    ) {
        let chunk = match chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        let material = materials.get(pixel.material_id);

        // Probability check - material burns gradually
        if rng.check_probability(material.burn_rate) {
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            // Transform to burns_to material (or air if not specified)
            let new_material = material.burns_to.unwrap_or(MaterialId::AIR);

            // Set pixel directly in chunks
            let target_chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
            let target_chunk_y = world_y.div_euclid(CHUNK_SIZE as i32);
            let target_chunk_pos = IVec2::new(target_chunk_x, target_chunk_y);

            if let Some(target_chunk) = chunks.get_mut(&target_chunk_pos) {
                let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
                let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
                target_chunk.set_pixel(local_x, local_y, super::Pixel::new(new_material));
            }

            // Add heat from burning
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                add_heat_at_pixel(chunk, x, y, 20.0);
            }
        }
    }
}
