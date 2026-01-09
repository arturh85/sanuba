//! Manages the propagation of electricity through conductive materials.
// Based on the POWDER_PLAN.md

use crate::simulation::{MaterialId, Materials, Pixel, pixel_flags};
use crate::world::{CHUNK_SIZE, Chunk};
use glam::IVec2;
use std::collections::{HashMap, VecDeque};

const PROPAGATION_QUEUE_MAX: usize = 256;
const MAX_SPARK_JUMP_DISTANCE: i32 = 1; // How far a spark can jump across air to another conductor
const THUNDER_RADIUS: i32 = 2; // Radius of destruction for thunder

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
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        // 1. Deplete power sources and discharge powered pixels
        self.discharge_and_deplete(chunks, active_chunks);

        // 2. Add power from active sources to the grid
        self.add_power_from_sources(chunks, active_chunks, materials);

        
        // 3. Propagate power through conductors
        self.propagate_power(chunks, materials);

        // 4. Handle special behaviors (sparks, thunder) and generate heat
        self.handle_spark_and_thunder_effects(chunks, active_chunks, materials);
        self.handle_effects(chunks, active_chunks, materials);
    }

    /// Reduces power level of powered pixels and batteries.
    fn discharge_and_deplete(&self, chunks: &mut HashMap<IVec2, Chunk>, active_chunks: &[IVec2]) {
        for &chunk_pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&chunk_pos) {
                let mut needs_update = false;
                for i in 0..chunk.electrical_potential.len() {
                    if chunk.electrical_potential[i] > 0.0 {
                        let (cy, cx) = (i / 8, i % 8);
                        let mut is_battery_cell = false;
                        'outer: for y in (cy * 8)..(cy * 8 + 8) {
                            for x in (cx * 8)..(cx * 8 + 8) {
                                if chunk.get_pixel(x, y).flags & pixel_flags::SPARK_SOURCE != 0 {
                                    is_battery_cell = true;
                                    break 'outer;
                                }
                            }
                        }

                        if !is_battery_cell {
                            chunk.electrical_potential[i] =
                                (chunk.electrical_potential[i] - 0.1).max(0.0);
                            needs_update = true;
                        }

                        if chunk.electrical_potential[i] == 0.0 {
                            for y in (cy * 8)..(cy * 8 + 8) {
                                for x in (cx * 8)..(cx * 8 + 8) {
                                    let mut pixel = chunk.get_pixel(x, y);
                                    if pixel.flags & pixel_flags::POWERED != 0 {
                                        pixel.flags &= !pixel_flags::POWERED;
                                        chunk.set_pixel(x, y, pixel);
                                    }
                                }
                            }
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
        chunks: &mut HashMap<IVec2, Chunk>,
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

                                let mut pixel = chunk.get_pixel(x, y);
                                pixel.flags |= pixel_flags::POWERED;
                                chunk.set_pixel(x, y, pixel);


                            }
                        }
                    }
                }
            }
        }
    }

    /// Propagates electricity through the grid.
    fn propagate_power(&mut self, chunks: &mut HashMap<IVec2, Chunk>, materials: &Materials) {
        let mut depth = 0;
        const MAX_DEPTH_PER_FRAME: usize = 128;


        while let Some((chunk_pos, x, y)) = self.propagation_queue.pop_front() {

            if depth > MAX_DEPTH_PER_FRAME {

                break;
            }
            depth += 1;

            let source_potential = {
                if let Some(chunk) = chunks.get(&chunk_pos) {
                    let coarse_idx = chunk.get_coarse_grid_index(x, y);
                    chunk.electrical_potential[coarse_idx]
                } else {
                    continue;
                }
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
                    let (next_chunk_pos, next_x, next_y) = self.get_neighbor_pos(chunk_pos, nx, ny);

                    // Check if chunk exists before attempting to get_mut
                    if let Some(neighbor_chunk) = chunks.get_mut(&next_chunk_pos) {
                        let neighbor_pixel = neighbor_chunk.get_pixel(next_x, next_y);
                        let neighbor_material = materials.get(neighbor_pixel.material_id);

                        if neighbor_material.conducts_electricity {
                            let neighbor_coarse_idx =
                                neighbor_chunk.get_coarse_grid_index(next_x, next_y);

                            let neighbor_potential =
                                &mut neighbor_chunk.electrical_potential[neighbor_coarse_idx];

                            let transfer_amount = (source_potential - *neighbor_potential)
                                * neighbor_material.electrical_conductivity
                                * 0.5;

                            if transfer_amount > 0.01 {
                                *neighbor_potential += transfer_amount;

                                let mut p = neighbor_chunk.get_pixel(next_x, next_y);
                                p.flags |= pixel_flags::POWERED;
                                neighbor_chunk.set_pixel(next_x, next_y, p);

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

    /// Handles special behaviors for SPARK and THUNDER materials.
    fn handle_spark_and_thunder_effects(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        active_chunks: &[IVec2],
        materials: &Materials,
    ) {
        let mut pixels_to_change: Vec<(IVec2, usize, usize, Pixel)> = Vec::new();
        let mut potentials_to_boost: Vec<(IVec2, usize, usize, f32)> = Vec::new();

        for &chunk_pos in active_chunks {
            // Cannot get_mut here because other functions might need mutable access to other chunks
            if let Some(chunk) = chunks.get(&chunk_pos) {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let pixel = chunk.get_pixel(x, y);
                        // Skip if already updated this frame (for sparks moving)
                        if pixel.flags & pixel_flags::UPDATED != 0 {
                            continue;
                        }

                        // Only powered sparks/thunder do anything special
                        let coarse_idx = chunk.get_coarse_grid_index(x, y);
                        let potential_at_pixel = chunk.electrical_potential[coarse_idx];
                        if potential_at_pixel <= 0.0 {
                            continue;
                        }

                        match pixel.material_id {
                            MaterialId::SPARK => {
                                // Spark: Try to jump small gaps
                                // Find an adjacent AIR pixel
                                'spark_check: for dy_air in
                                    -MAX_SPARK_JUMP_DISTANCE..=MAX_SPARK_JUMP_DISTANCE
                                {
                                    for dx_air in -MAX_SPARK_JUMP_DISTANCE..=MAX_SPARK_JUMP_DISTANCE
                                    {
                                        if dx_air == 0 && dy_air == 0 {
                                            continue;
                                        }

                                        let (air_chunk_pos, air_x, air_y) = self.get_neighbor_pos(
                                            chunk_pos,
                                            x as i32 + dx_air,
                                            y as i32 + dy_air,
                                        );

                                        let Some(air_pixel) = self.get_pixel_across_chunks(
                                            chunks,
                                            air_chunk_pos,
                                            air_x,
                                            air_y,
                                        ) else {
                                            continue;
                                        };
                                        if air_pixel.material_id != MaterialId::AIR {
                                            continue;
                                        }

                                        // Check if this AIR pixel has a POWERED, CONDUCTIVE neighbor (excluding the spark itself)
                                        for dy_cond in -1..=1 {
                                            for dx_cond in -1..=1 {
                                                if dy_cond == 0 && dx_cond == 0 {
                                                    continue;
                                                } // Don't check itself
                                                // Ensure the conductor is not the original spark's position
                                                if (air_x as i32 + dx_cond == x as i32)
                                                    && (air_y as i32 + dy_cond == y as i32)
                                                    && air_chunk_pos == chunk_pos
                                                {
                                                    continue;
                                                }

                                                let (cond_chunk_pos, cond_x, cond_y) = self
                                                    .get_neighbor_pos(
                                                        air_chunk_pos,
                                                        air_x as i32 + dx_cond,
                                                        air_y as i32 + dy_cond,
                                                    );

// use log::{debug, info}; // Commented out log imports

// ... (rest of the file) ...

                                                if let Some(cond_pixel) = self
                                                    .get_pixel_across_chunks(
                                                        chunks,
                                                        cond_chunk_pos,
                                                        cond_x,
                                                        cond_y,
                                                    )
                                                {
                                                    let cond_material =
                                                        materials.get(cond_pixel.material_id);
                                                    // Conductor must be unpowered for spark to jump and power it
                                                    if cond_pixel.flags & pixel_flags::POWERED == 0
                                                        && cond_material.conducts_electricity
                                                    {
                                                        pixels_to_change.push((
                                                            chunk_pos,
                                                            x,
                                                            y,
                                                            Pixel::new(MaterialId::AIR),
                                                        )); // Original spark becomes air
                                                        pixels_to_change.push((
                                                            air_chunk_pos,
                                                            air_x,
                                                            air_y,
                                                            Pixel {
                                                                material_id: MaterialId::SPARK,
                                                                flags: pixel_flags::POWERED
                                                                    | pixel_flags::UPDATED, // New spark, powered and updated
                                                            },
                                                        ));
                                                        break 'spark_check; // Only one jump per spark per frame
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            MaterialId::THUNDER => {
                                // Thunder: Instant propagation, destroys non-conductors
                                // Collect effects, then apply later to avoid mutable borrow issues
                                pixels_to_change.push((
                                    chunk_pos,
                                    x,
                                    y,
                                    Pixel::new(MaterialId::AIR),
                                )); // Thunder dissipates

                                for dy in -THUNDER_RADIUS..=THUNDER_RADIUS {
                                    for dx in -THUNDER_RADIUS..=THUNDER_RADIUS {
                                        let (target_chunk_pos, target_x, target_y) = self
                                            .get_neighbor_pos(
                                                chunk_pos,
                                                x as i32 + dx,
                                                y as i32 + dy,
                                            );

                                        if let Some(target_pixel) = self.get_pixel_across_chunks(
                                            chunks,
                                            target_chunk_pos,
                                            target_x,
                                            target_y,
                                        ) {
                                            let target_material =
                                                materials.get(target_pixel.material_id);

                                            if !target_material.conducts_electricity
                                                && target_pixel.material_id != MaterialId::AIR
                                            {
                                                // Destroy non-conductors
                                                pixels_to_change.push((
                                                    target_chunk_pos,
                                                    target_x,
                                                    target_y,
                                                    Pixel::new(MaterialId::AIR),
                                                ));
                                            } else if target_material.conducts_electricity {
                                                // Boost potential of conductors (instant propagation effect)
                                                potentials_to_boost.push((
                                                    target_chunk_pos,
                                                    target_x,
                                                    target_y,
                                                    100.0,
                                                )); // Max potential
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {} // Other materials
                        }
                    }
                }
            }
        }

        // Apply all collected pixel changes
        for (pos, x, y, new_pixel) in pixels_to_change {
            self.set_pixel_across_chunks(chunks, pos, x, y, new_pixel);
        }

        // Apply all collected potential boosts
        for (pos, x, y, boost_amount) in potentials_to_boost {
            self.set_electrical_potential_across_chunks(chunks, pos, x, y, boost_amount);
        }
    }

    /// Generates heat and other effects from electricity.
    fn handle_effects(
        &self,
        chunks: &mut HashMap<IVec2, Chunk>,
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

    /// Helper to get a pixel from chunks, handling chunk boundaries.
    fn get_pixel_across_chunks(
        &self,
        chunks: &HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
    ) -> Option<Pixel> {
        let (actual_chunk_pos, actual_x, actual_y) =
            self.get_neighbor_pos(chunk_pos, x as i32, y as i32);
        chunks
            .get(&actual_chunk_pos)
            .map(|chunk| chunk.get_pixel(actual_x, actual_y))
    }

    /// Helper to set a pixel in chunks, handling chunk boundaries.
    /// This requires mutable access to the chunks HashMap.
    fn set_pixel_across_chunks(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
        new_pixel: Pixel,
    ) {
        let (actual_chunk_pos, actual_x, actual_y) =
            self.get_neighbor_pos(chunk_pos, x as i32, y as i32);
        if let Some(chunk) = chunks.get_mut(&actual_chunk_pos) {
            chunk.set_pixel(actual_x, actual_y, new_pixel);
            chunk.dirty = true;
        }
    }

    /// Helper to set electrical potential in chunks, handling chunk boundaries.
    fn set_electrical_potential_across_chunks(
        &mut self,
        chunks: &mut HashMap<IVec2, Chunk>,
        chunk_pos: IVec2,
        x: usize,
        y: usize,
        potential: f32,
    ) {
        let (actual_chunk_pos, actual_x, actual_y) =
            self.get_neighbor_pos(chunk_pos, x as i32, y as i32);
        if let Some(chunk) = chunks.get_mut(&actual_chunk_pos) {
            let coarse_idx = chunk.get_coarse_grid_index(actual_x, actual_y);
            chunk.electrical_potential[coarse_idx] =
                potential.max(chunk.electrical_potential[coarse_idx]); // Only boost, not set blindly
            chunk.dirty = true;
        }
    }

    /// Helper to get neighbor position, handling chunk boundaries.
    fn get_neighbor_pos(&self, chunk_pos: IVec2, x: i32, y: i32) -> (IVec2, usize, usize) {
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
