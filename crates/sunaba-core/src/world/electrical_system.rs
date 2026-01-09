//! Manages the propagation of electricity through conductive materials.

use crate::world::{CHUNK_SIZE, Chunk, pixel_flags};

pub struct ElectricalSystem;

impl ElectricalSystem {
    pub fn new() -> Self {
        Self
    }

    /// Updates the electrical state for a single chunk.
    pub fn update(&self, chunk: &mut Chunk) {
        // A simple, single-pass update for now.
        // This does not handle complex propagation across the whole circuit in one frame,
        // but electricity will spread frame-by-frame.

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let pixel = chunk.get_pixel(x, y);

                // If a pixel is a source or is powered and conductive...
                let is_source = pixel.flags & pixel_flags::SPARK_SOURCE != 0;
                let is_powered_conductor = (pixel.flags & pixel_flags::POWERED != 0)
                    && (pixel.flags & pixel_flags::CONDUCTIVE != 0);

                if is_source || is_powered_conductor {
                    // ...try to power its neighbors.
                    self.power_neighbors(chunk, x, y);
                }
            }
        }
    }

    /// Powers the neighbors of a given pixel within the same chunk.
    fn power_neighbors(&self, chunk: &mut Chunk, x: usize, y: usize) {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;

                // Check chunk boundaries
                if nx >= 0 && nx < CHUNK_SIZE as isize && ny >= 0 && ny < CHUNK_SIZE as isize {
                    let nx = nx as usize;
                    let ny = ny as usize;

                    let mut neighbor_pixel = chunk.get_pixel(nx, ny);
                    let is_conductive = neighbor_pixel.flags & pixel_flags::CONDUCTIVE != 0;
                    let is_powered = neighbor_pixel.flags & pixel_flags::POWERED != 0;

                    // If the neighbor is conductive but not yet powered, power it.
                    if is_conductive && !is_powered {
                        neighbor_pixel.flags |= pixel_flags::POWERED;
                        chunk.set_pixel(nx, ny, neighbor_pixel);
                    }
                }
            }
        }
    }
}

impl Default for ElectricalSystem {
    fn default() -> Self {
        Self::new()
    }
}
