//! World - manages chunks and simulation

use std::collections::HashMap;
use glam::IVec2;

use super::{Chunk, Pixel, CHUNK_SIZE, pixel_flags};
use crate::simulation::{
    Materials, MaterialId, MaterialType,
    TemperatureSimulator, StateChangeSystem, ReactionRegistry,
    add_heat_at_pixel, get_temperature_at_pixel,
};

/// The game world, composed of chunks
pub struct World {
    /// Loaded chunks, keyed by chunk coordinates
    chunks: HashMap<IVec2, Chunk>,

    /// Material definitions
    pub materials: Materials,

    /// Temperature simulation system
    temperature_sim: TemperatureSimulator,

    /// Chemical reaction registry
    reactions: ReactionRegistry,

    /// Player position (pixel coordinates)
    pub player_pos: glam::Vec2,

    /// Which chunks are currently active (being simulated)
    active_chunks: Vec<IVec2>,

    /// Simulation time accumulator
    time_accumulator: f32,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            chunks: HashMap::new(),
            materials: Materials::new(),
            temperature_sim: TemperatureSimulator::new(),
            reactions: ReactionRegistry::new(),
            player_pos: glam::Vec2::new(0.0, 100.0),
            active_chunks: Vec::new(),
            time_accumulator: 0.0,
        };
        
        // Initialize with some test chunks
        world.generate_test_world();
        
        world
    }
    
    /// Generate a simple test world for development
    fn generate_test_world(&mut self) {
        let mut total_pixels = 0;

        // Create a 5x5 grid of chunks around origin
        for cy in -2..=2 {
            for cx in -2..=2 {
                let mut chunk = Chunk::new(cx, cy);
                let mut chunk_pixels = 0;

                // Fill bottom half with stone
                for y in 0..32 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                        chunk_pixels += 1;
                    }
                }

                // Add some sand on top
                if cy == 0 {
                    for x in 20..44 {
                        for y in 32..40 {
                            chunk.set_material(x, y, MaterialId::SAND);
                            chunk_pixels += 1;
                        }
                    }
                }

                // Add some water
                if cx == 1 && cy == 0 {
                    for x in 10..30 {
                        for y in 35..50 {
                            chunk.set_material(x, y, MaterialId::WATER);
                            chunk_pixels += 1;
                        }
                    }
                }

                total_pixels += chunk_pixels;
                self.chunks.insert(IVec2::new(cx, cy), chunk);
                self.active_chunks.push(IVec2::new(cx, cy));
            }
        }

        log::info!("Generated test world: {} chunks, {} pixels",
                   self.chunks.len(), total_pixels);
        log::info!("  World bounds: ({}, {}) to ({}, {})",
                   -2 * CHUNK_SIZE as i32, -2 * CHUNK_SIZE as i32,
                   3 * CHUNK_SIZE as i32 - 1, 3 * CHUNK_SIZE as i32 - 1);
        log::info!("  Player starts at: {:?}", self.player_pos);
    }

    /// Player movement speed in pixels per second
    const PLAYER_SPEED: f32 = 200.0;

    /// Update player position based on input
    pub fn update_player(&mut self, input: &crate::app::InputState, dt: f32) {
        let mut velocity = glam::Vec2::ZERO;

        if input.w_pressed {
            velocity.y += 1.0;
        }
        if input.s_pressed {
            velocity.y -= 1.0;
        }
        if input.a_pressed {
            velocity.x -= 1.0;
        }
        if input.d_pressed {
            velocity.x += 1.0;
        }

        // Normalize diagonal movement
        if velocity.length() > 0.0 {
            velocity = velocity.normalize() * Self::PLAYER_SPEED;
            let new_pos = self.player_pos + velocity * dt;
            log::debug!("Player: {:?} → {:?} (velocity: {:?})",
                       self.player_pos, new_pos, velocity);
            self.player_pos = new_pos;
        }
    }

    /// Brush radius for material spawning (1 = 3x3, 2 = 5x5)
    const BRUSH_RADIUS: i32 = 1;

    /// Spawn material at world coordinates with circular brush
    pub fn spawn_material(&mut self, world_x: i32, world_y: i32, material_id: u16) {
        let material_name = &self.materials.get(material_id).name;
        log::info!("Spawning {} at world ({}, {})", material_name, world_x, world_y);

        let mut spawned = 0;
        for dy in -Self::BRUSH_RADIUS..=Self::BRUSH_RADIUS {
            for dx in -Self::BRUSH_RADIUS..=Self::BRUSH_RADIUS {
                // Circular brush
                if dx * dx + dy * dy <= Self::BRUSH_RADIUS * Self::BRUSH_RADIUS {
                    let x = world_x + dx;
                    let y = world_y + dy;
                    self.set_pixel(x, y, material_id);
                    spawned += 1;
                }
            }
        }
        log::debug!("  → Spawned {} pixels", spawned);
    }

    /// Update simulation
    pub fn update(&mut self, dt: f32, stats: &mut crate::ui::StatsCollector) {
        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;

        self.time_accumulator += dt;

        while self.time_accumulator >= FIXED_TIMESTEP {
            self.step_simulation(stats);
            self.time_accumulator -= FIXED_TIMESTEP;
        }
    }
    
    fn step_simulation(&mut self, stats: &mut crate::ui::StatsCollector) {
        // 1. Clear update flags
        for pos in &self.active_chunks {
            if let Some(chunk) = self.chunks.get_mut(pos) {
                chunk.clear_update_flags();
            }
        }

        // 2. CA updates (movement)
        // TODO: Implement Noita-style checkerboard update pattern
        // For now, simple sequential update
        for pos in self.active_chunks.clone() {
            self.update_chunk_ca(pos, stats);
        }

        // 3. Temperature diffusion (30fps throttled)
        self.temperature_sim.update(&mut self.chunks);

        // 4. State changes based on temperature
        for pos in &self.active_chunks.clone() {
            self.check_chunk_state_changes(*pos, stats);
        }
    }
    
    fn update_chunk_ca(&mut self, chunk_pos: IVec2, stats: &mut crate::ui::StatsCollector) {
        // Update from bottom to top so falling works correctly
        for y in 0..CHUNK_SIZE {
            // Alternate direction each row for symmetry
            let x_iter: Box<dyn Iterator<Item = usize>> = if y % 2 == 0 {
                Box::new(0..CHUNK_SIZE)
            } else {
                Box::new((0..CHUNK_SIZE).rev())
            };

            for x in x_iter {
                self.update_pixel(chunk_pos, x, y, stats);
            }
        }
    }
    
    fn update_pixel(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        if pixel.is_empty() {
            return;
        }

        // Special handling for fire
        if pixel.material_id == MaterialId::FIRE {
            self.update_fire(chunk_pos, x, y, stats);
            return;
        }

        // Check if pixel should ignite (before movement)
        if pixel.flags & pixel_flags::BURNING == 0 {
            self.check_ignition(chunk_pos, x, y);
        }

        // Update burning materials
        if pixel.flags & pixel_flags::BURNING != 0 {
            self.update_burning_material(chunk_pos, x, y);
        }

        // Get material type for movement logic
        let material_type = self.materials.get(pixel.material_id).material_type;

        // Normal CA movement
        match material_type {
            MaterialType::Powder => {
                self.update_powder(chunk_pos, x, y, stats);
            }
            MaterialType::Liquid => {
                self.update_liquid(chunk_pos, x, y, stats);
            }
            MaterialType::Gas => {
                self.update_gas(chunk_pos, x, y, stats);
            }
            MaterialType::Solid => {
                // Solids don't move
            }
        }

        // Check reactions with neighbors (after movement)
        self.check_pixel_reactions(chunk_pos, x, y, stats);
    }
    
    fn update_powder(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        // Convert to world coordinates
        let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
        let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

        // Try to fall down
        if self.try_move_world(world_x, world_y, world_x, world_y - 1, stats) {
            return;
        }

        // Try to fall diagonally (randomized for symmetry)
        let try_left_first = rand::random::<bool>();

        if try_left_first {
            if self.try_move_world(world_x, world_y, world_x - 1, world_y - 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x + 1, world_y - 1, stats) {
                return;
            }
        } else {
            if self.try_move_world(world_x, world_y, world_x + 1, world_y - 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x - 1, world_y - 1, stats) {
                return;
            }
        }
    }
    
    fn update_liquid(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        // Convert to world coordinates
        let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
        let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

        // Try to fall down
        if self.try_move_world(world_x, world_y, world_x, world_y - 1, stats) {
            return;
        }

        // Try to fall diagonally
        let try_left_first = rand::random::<bool>();

        if try_left_first {
            if self.try_move_world(world_x, world_y, world_x - 1, world_y - 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x + 1, world_y - 1, stats) {
                return;
            }
        } else {
            if self.try_move_world(world_x, world_y, world_x + 1, world_y - 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x - 1, world_y - 1, stats) {
                return;
            }
        }

        // Try to flow horizontally
        if try_left_first {
            if self.try_move_world(world_x, world_y, world_x - 1, world_y, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x + 1, world_y, stats) {
                return;
            }
        } else {
            if self.try_move_world(world_x, world_y, world_x + 1, world_y, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x - 1, world_y, stats) {
                return;
            }
        }
    }
    
    fn update_gas(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        // Convert to world coordinates
        let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
        let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

        // Gases rise (positive Y)
        if self.try_move_world(world_x, world_y, world_x, world_y + 1, stats) {
            return;
        }

        // Try to rise diagonally
        let try_left_first = rand::random::<bool>();

        if try_left_first {
            if self.try_move_world(world_x, world_y, world_x - 1, world_y + 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x + 1, world_y + 1, stats) {
                return;
            }
        } else {
            if self.try_move_world(world_x, world_y, world_x + 1, world_y + 1, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x - 1, world_y + 1, stats) {
                return;
            }
        }

        // Disperse horizontally
        if try_left_first {
            if self.try_move_world(world_x, world_y, world_x - 1, world_y, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x + 1, world_y, stats) {
                return;
            }
        } else {
            if self.try_move_world(world_x, world_y, world_x + 1, world_y, stats) {
                return;
            }
            if self.try_move_world(world_x, world_y, world_x - 1, world_y, stats) {
                return;
            }
        }
    }
    
    /// Try to move a pixel, returns true if successful
    fn try_move(&mut self, chunk_pos: IVec2, from_x: usize, from_y: usize, to_x: usize, to_y: usize) -> bool {
        // TODO: Handle cross-chunk movement
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return false,
        };
        
        let target = chunk.get_pixel(to_x, to_y);
        
        // Can only move into empty space (for now)
        // TODO: Handle density-based displacement (water sinks under oil, etc.)
        if target.is_empty() {
            let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
            chunk.swap_pixels(from_x, from_y, to_x, to_y);
            true
        } else {
            false
        }
    }

    /// Try to move a pixel using world coordinates (handles cross-chunk movement)
    fn try_move_world(
        &mut self,
        from_world_x: i32,
        from_world_y: i32,
        to_world_x: i32,
        to_world_y: i32,
        stats: &mut crate::ui::StatsCollector,
    ) -> bool {
        // Convert to chunk coordinates
        let (src_chunk_pos, src_x, src_y) = Self::world_to_chunk_coords(from_world_x, from_world_y);
        let (dst_chunk_pos, dst_x, dst_y) = Self::world_to_chunk_coords(to_world_x, to_world_y);

        // Phase 1: Read pixels (immutable borrows)
        let src_pixel = match self.chunks.get(&src_chunk_pos) {
            Some(c) => c.get_pixel(src_x, src_y),
            None => return false, // Source chunk not loaded
        };

        let dst_pixel = match self.chunks.get(&dst_chunk_pos) {
            Some(c) => c.get_pixel(dst_x, dst_y),
            None => return false, // Destination chunk not loaded
        };

        // Can only move into empty space (for now)
        // TODO: Handle density-based displacement (water sinks under oil, etc.)
        if !dst_pixel.is_empty() {
            return false;
        }

        // Phase 2: Write pixels (mutable borrows)
        if src_chunk_pos == dst_chunk_pos {
            // Same chunk - use swap for efficiency
            if let Some(chunk) = self.chunks.get_mut(&src_chunk_pos) {
                chunk.swap_pixels(src_x, src_y, dst_x, dst_y);
                stats.record_pixel_moved();
                return true;
            }
        } else {
            // Different chunks - sequential writes to avoid borrow checker issues
            // First, clear source
            if let Some(src_chunk) = self.chunks.get_mut(&src_chunk_pos) {
                src_chunk.set_pixel(src_x, src_y, Pixel::AIR);
            } else {
                return false;
            }

            // Then, set destination
            if let Some(dst_chunk) = self.chunks.get_mut(&dst_chunk_pos) {
                dst_chunk.set_pixel(dst_x, dst_y, src_pixel);
                stats.record_pixel_moved();
                return true;
            } else {
                // Rollback: restore source pixel
                if let Some(src_chunk) = self.chunks.get_mut(&src_chunk_pos) {
                    src_chunk.set_pixel(src_x, src_y, src_pixel);
                }
                return false;
            }
        }

        false
    }

    /// Get pixel at world coordinates
    pub fn get_pixel(&self, world_x: i32, world_y: i32) -> Option<Pixel> {
        let (chunk_pos, local_x, local_y) = Self::world_to_chunk_coords(world_x, world_y);
        self.chunks.get(&chunk_pos).map(|c| c.get_pixel(local_x, local_y))
    }
    
    /// Set pixel at world coordinates
    pub fn set_pixel(&mut self, world_x: i32, world_y: i32, material_id: u16) {
        let (chunk_pos, local_x, local_y) = Self::world_to_chunk_coords(world_x, world_y);
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.set_material(local_x, local_y, material_id);
        } else {
            log::warn!("set_pixel: chunk {:?} not loaded (world: {}, {})",
                      chunk_pos, world_x, world_y);
        }
    }

    /// Get temperature at world coordinates
    pub fn get_temperature_at_pixel(&self, world_x: i32, world_y: i32) -> f32 {
        let (chunk_pos, local_x, local_y) = Self::world_to_chunk_coords(world_x, world_y);
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            get_temperature_at_pixel(chunk, local_x, local_y)
        } else {
            20.0 // Default ambient temperature
        }
    }

    /// Convert world coordinates to chunk coordinates + local offset
    fn world_to_chunk_coords(world_x: i32, world_y: i32) -> (IVec2, usize, usize) {
        let chunk_x = world_x.div_euclid(CHUNK_SIZE as i32);
        let chunk_y = world_y.div_euclid(CHUNK_SIZE as i32);
        let local_x = world_x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = world_y.rem_euclid(CHUNK_SIZE as i32) as usize;
        (IVec2::new(chunk_x, chunk_y), local_x, local_y)
    }
    
    /// Get iterator over active chunks
    pub fn active_chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.active_chunks.iter().filter_map(|pos| self.chunks.get(pos))
    }
    
    /// Get all loaded chunks
    pub fn chunks(&self) -> &HashMap<IVec2, Chunk> {
        &self.chunks
    }

    /// Get materials registry
    pub fn materials(&self) -> &Materials {
        &self.materials
    }

    /// Clear all chunks from the world
    pub fn clear_all_chunks(&mut self) {
        self.chunks.clear();
        self.active_chunks.clear();
        log::info!("Cleared all chunks");
    }

    /// Add a chunk to the world
    pub fn add_chunk(&mut self, chunk: Chunk) {
        let pos = IVec2::new(chunk.x, chunk.y);
        self.chunks.insert(pos, chunk);

        // Add to active chunks if within range of player
        let dist_x = (pos.x - (self.player_pos.x as i32 / CHUNK_SIZE as i32)).abs();
        let dist_y = (pos.y - (self.player_pos.y as i32 / CHUNK_SIZE as i32)).abs();
        if dist_x <= 2 && dist_y <= 2 {
            if !self.active_chunks.contains(&pos) {
                self.active_chunks.push(pos);
            }
        }
    }

    /// Check all pixels in a chunk for state changes based on temperature
    fn check_chunk_state_changes(&mut self, chunk_pos: IVec2, stats: &mut crate::ui::StatsCollector) {
        let chunk = match self.chunks.get_mut(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel = chunk.get_pixel(x, y);
                if pixel.is_empty() {
                    continue;
                }

                let material = self.materials.get(pixel.material_id);
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
    fn update_fire(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        // 1. Add heat to temperature field
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            add_heat_at_pixel(chunk, x, y, 50.0); // Fire adds significant heat
        }

        // 2. Fire behaves like gas (rises)
        self.update_gas(chunk_pos, x, y, stats);

        // 3. Fire has limited lifetime - random chance to become smoke
        if rand::random::<f32>() < 0.02 {
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;
            self.set_pixel(world_x, world_y, MaterialId::SMOKE);
        }
    }

    /// Check if a pixel should ignite based on temperature
    fn check_ignition(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        let material = self.materials.get(pixel.material_id);

        if !material.flammable {
            return;
        }

        let temp = get_temperature_at_pixel(chunk, x, y);

        if let Some(ignition_temp) = material.ignition_temp {
            if temp >= ignition_temp {
                // Mark pixel as burning
                let chunk = self.chunks.get_mut(&chunk_pos).unwrap();
                let mut new_pixel = pixel;
                new_pixel.flags |= pixel_flags::BURNING;
                chunk.set_pixel(x, y, new_pixel);

                // Try to spawn fire in adjacent air cell
                let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
                let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

                for (dx, dy) in [(0, 1), (1, 0), (-1, 0), (0, -1)] {
                    if let Some(neighbor) = self.get_pixel(world_x + dx, world_y + dy) {
                        if neighbor.is_empty() {
                            self.set_pixel(world_x + dx, world_y + dy, MaterialId::FIRE);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Update burning material (gradual consumption)
    fn update_burning_material(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        let material = self.materials.get(pixel.material_id);

        // Probability check - material burns gradually
        if rand::random::<f32>() < material.burn_rate {
            let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
            let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

            // Transform to burns_to material (or air if not specified)
            let new_material = material.burns_to.unwrap_or(MaterialId::AIR);
            self.set_pixel(world_x, world_y, new_material);

            // Add heat from burning
            if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
                add_heat_at_pixel(chunk, x, y, 20.0);
            }
        }
    }

    /// Check for chemical reactions with neighboring pixels
    fn check_pixel_reactions(&mut self, chunk_pos: IVec2, x: usize, y: usize, stats: &mut crate::ui::StatsCollector) {
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };

        let pixel = chunk.get_pixel(x, y);
        if pixel.is_empty() {
            return;
        }

        let temp = get_temperature_at_pixel(chunk, x, y);
        let world_x = chunk_pos.x * CHUNK_SIZE as i32 + x as i32;
        let world_y = chunk_pos.y * CHUNK_SIZE as i32 + y as i32;

        // Check 4 neighbors for reactions
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let neighbor = match self.get_pixel(world_x + dx, world_y + dy) {
                Some(p) => p,
                None => continue,
            };

            if neighbor.is_empty() {
                continue;
            }

            // Find matching reaction
            if let Some(reaction) = self.reactions.find_reaction(
                pixel.material_id,
                neighbor.material_id,
                temp,
            ) {
                // Probability check
                if rand::random::<f32>() < reaction.probability {
                    // Apply reaction - get correct outputs based on material order
                    let (output_a, output_b) = self.reactions.get_outputs(
                        reaction,
                        pixel.material_id,
                        neighbor.material_id,
                    );

                    self.set_pixel(world_x, world_y, output_a);
                    self.set_pixel(world_x + dx, world_y + dy, output_b);
                    stats.record_reaction();
                    return; // Only one reaction per pixel per frame
                }
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
