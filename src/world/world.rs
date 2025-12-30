//! World - manages chunks and simulation

use std::collections::HashMap;
use glam::IVec2;

use super::{Chunk, Pixel, CHUNK_SIZE};
use crate::simulation::{Materials, MaterialId};

/// The game world, composed of chunks
pub struct World {
    /// Loaded chunks, keyed by chunk coordinates
    chunks: HashMap<IVec2, Chunk>,
    
    /// Material definitions
    pub materials: Materials,
    
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
        // Create a 5x5 grid of chunks around origin
        for cy in -2..=2 {
            for cx in -2..=2 {
                let mut chunk = Chunk::new(cx, cy);
                
                // Fill bottom half with stone
                for y in 0..32 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                
                // Add some sand on top
                if cy == 0 {
                    for x in 20..44 {
                        for y in 32..40 {
                            chunk.set_material(x, y, MaterialId::SAND);
                        }
                    }
                }
                
                // Add some water
                if cx == 1 && cy == 0 {
                    for x in 10..30 {
                        for y in 35..50 {
                            chunk.set_material(x, y, MaterialId::WATER);
                        }
                    }
                }
                
                self.chunks.insert(IVec2::new(cx, cy), chunk);
                self.active_chunks.push(IVec2::new(cx, cy));
            }
        }
    }
    
    /// Update simulation
    pub fn update(&mut self, dt: f32) {
        const FIXED_TIMESTEP: f32 = 1.0 / 60.0;
        
        self.time_accumulator += dt;
        
        while self.time_accumulator >= FIXED_TIMESTEP {
            self.step_simulation();
            self.time_accumulator -= FIXED_TIMESTEP;
        }
    }
    
    fn step_simulation(&mut self) {
        // Clear update flags
        for pos in &self.active_chunks {
            if let Some(chunk) = self.chunks.get_mut(pos) {
                chunk.clear_update_flags();
            }
        }
        
        // TODO: Implement Noita-style checkerboard update pattern
        // For now, simple sequential update
        for pos in self.active_chunks.clone() {
            self.update_chunk_ca(pos);
        }
    }
    
    fn update_chunk_ca(&mut self, chunk_pos: IVec2) {
        // Update from bottom to top so falling works correctly
        for y in 0..CHUNK_SIZE {
            // Alternate direction each row for symmetry
            let x_iter: Box<dyn Iterator<Item = usize>> = if y % 2 == 0 {
                Box::new(0..CHUNK_SIZE)
            } else {
                Box::new((0..CHUNK_SIZE).rev())
            };
            
            for x in x_iter {
                self.update_pixel(chunk_pos, x, y);
            }
        }
    }
    
    fn update_pixel(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        let chunk = match self.chunks.get(&chunk_pos) {
            Some(c) => c,
            None => return,
        };
        
        let pixel = chunk.get_pixel(x, y);
        if pixel.is_empty() {
            return;
        }
        
        let material = self.materials.get(pixel.material_id);
        
        match material.material_type {
            crate::simulation::MaterialType::Powder => {
                self.update_powder(chunk_pos, x, y);
            }
            crate::simulation::MaterialType::Liquid => {
                self.update_liquid(chunk_pos, x, y);
            }
            crate::simulation::MaterialType::Gas => {
                self.update_gas(chunk_pos, x, y);
            }
            crate::simulation::MaterialType::Solid => {
                // Solids don't move
            }
        }
    }
    
    fn update_powder(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        // Try to fall down
        if y > 0 && self.try_move(chunk_pos, x, y, x, y - 1) {
            return;
        }
        
        // Try to fall diagonally
        if y > 0 {
            // Randomize which diagonal to try first
            let try_left_first = rand::random::<bool>();
            
            if try_left_first {
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y - 1) {
                    return;
                }
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y - 1) {
                    return;
                }
            } else {
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y - 1) {
                    return;
                }
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y - 1) {
                    return;
                }
            }
        }
    }
    
    fn update_liquid(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        // Try to fall down
        if y > 0 && self.try_move(chunk_pos, x, y, x, y - 1) {
            return;
        }
        
        // Try to fall diagonally
        if y > 0 {
            let try_left_first = rand::random::<bool>();
            
            if try_left_first {
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y - 1) {
                    return;
                }
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y - 1) {
                    return;
                }
            } else {
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y - 1) {
                    return;
                }
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y - 1) {
                    return;
                }
            }
        }
        
        // Try to flow horizontally
        let try_left_first = rand::random::<bool>();
        if try_left_first {
            if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y) {
                return;
            }
            if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y) {
                return;
            }
        } else {
            if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y) {
                return;
            }
            if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y) {
                return;
            }
        }
    }
    
    fn update_gas(&mut self, chunk_pos: IVec2, x: usize, y: usize) {
        // Gases rise
        if y < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x, y + 1) {
            return;
        }
        
        // Try to rise diagonally
        if y < CHUNK_SIZE - 1 {
            let try_left_first = rand::random::<bool>();
            
            if try_left_first {
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y + 1) {
                    return;
                }
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y + 1) {
                    return;
                }
            } else {
                if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y + 1) {
                    return;
                }
                if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y + 1) {
                    return;
                }
            }
        }
        
        // Disperse horizontally
        let try_left_first = rand::random::<bool>();
        if try_left_first {
            if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y) {
                return;
            }
            if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y) {
                return;
            }
        } else {
            if x < CHUNK_SIZE - 1 && self.try_move(chunk_pos, x, y, x + 1, y) {
                return;
            }
            if x > 0 && self.try_move(chunk_pos, x, y, x - 1, y) {
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
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
