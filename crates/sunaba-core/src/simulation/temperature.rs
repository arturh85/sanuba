//! Temperature simulation system
//!
//! Manages heat diffusion across the 8x8 temperature grid within each chunk.
//! Hot materials (fire, lava) heat their surroundings, and temperature spreads
//! to neighboring cells over time.

use crate::world::Chunk;
use glam::IVec2;
use std::collections::HashMap;

/// Maximum temperature cap to prevent runaway heat accumulation
const MAX_TEMPERATURE: f32 = 3000.0;

/// Temperature simulator with 30fps throttling
pub struct TemperatureSimulator {
    /// Counter for throttling updates to 30fps (every 2 frames at 60fps)
    update_counter: u8,
}

impl TemperatureSimulator {
    pub fn new() -> Self {
        Self { update_counter: 0 }
    }

    /// Update temperature diffusion for active chunks only
    /// Throttled to 30fps for performance
    pub fn update(&mut self, chunks: &mut HashMap<IVec2, Chunk>, active_chunks: &[IVec2]) {
        // Throttle to 30fps (every 2 frames at 60fps)
        self.update_counter += 1;
        if self.update_counter < 2 {
            return;
        }
        self.update_counter = 0;

        // Only diffuse temperature in active chunks (not all 1000+ loaded chunks)
        for &pos in active_chunks {
            if let Some(chunk) = chunks.get_mut(&pos) {
                self.diffuse_chunk_temperature(chunk);
            }
        }
    }

    /// Diffuse temperature within a single chunk
    fn diffuse_chunk_temperature(&self, chunk: &mut Chunk) {
        const DIFFUSION_RATE: f32 = 0.1; // 0.0 - 1.0, how fast heat spreads

        let mut new_temps = chunk.temperature;

        // Update each temperature cell (8x8 grid)
        for cy in 0..8 {
            for cx in 0..8 {
                let idx = cy * 8 + cx;
                let current_temp = chunk.temperature[idx];

                // Average with 4 neighbors (von Neumann neighborhood)
                let mut neighbor_sum = 0.0;
                let mut neighbor_count = 0;

                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as i32 + dx;
                    let ny = cy as i32 + dy;

                    if (0..8).contains(&nx) && (0..8).contains(&ny) {
                        neighbor_sum += chunk.temperature[ny as usize * 8 + nx as usize];
                        neighbor_count += 1;
                    }
                    // Note: Cross-chunk diffusion skipped for MVP simplicity
                    // Chunk boundaries act as insulated barriers for now
                }

                if neighbor_count > 0 {
                    let avg_neighbor = neighbor_sum / neighbor_count as f32;
                    // Diffuse toward average of neighbors
                    new_temps[idx] = current_temp + (avg_neighbor - current_temp) * DIFFUSION_RATE;
                }
            }
        }

        chunk.temperature = new_temps;
    }
}

impl Default for TemperatureSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert pixel coordinates (0-63) to temperature cell coordinates (0-7)
/// Each temperature cell covers 8x8 pixels
#[inline]
pub fn pixel_to_temp_coords(pixel_x: usize, pixel_y: usize) -> (usize, usize) {
    (pixel_x / 8, pixel_y / 8)
}

/// Convert temperature cell coordinates to array index
#[inline]
pub fn temp_to_index(cx: usize, cy: usize) -> usize {
    cy * 8 + cx // Row-major order
}

/// Add heat to the temperature cell containing the given pixel
pub fn add_heat_at_pixel(chunk: &mut Chunk, x: usize, y: usize, heat: f32) {
    let (cx, cy) = pixel_to_temp_coords(x, y);
    let idx = temp_to_index(cx, cy);
    chunk.temperature[idx] = (chunk.temperature[idx] + heat).min(MAX_TEMPERATURE);
}

/// Get temperature at the cell containing the given pixel
pub fn get_temperature_at_pixel(chunk: &Chunk, x: usize, y: usize) -> f32 {
    let (cx, cy) = pixel_to_temp_coords(x, y);
    let idx = temp_to_index(cx, cy);
    chunk.temperature[idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_to_temp_coords() {
        assert_eq!(pixel_to_temp_coords(0, 0), (0, 0));
        assert_eq!(pixel_to_temp_coords(7, 7), (0, 0));
        assert_eq!(pixel_to_temp_coords(8, 8), (1, 1));
        assert_eq!(pixel_to_temp_coords(63, 63), (7, 7));
    }

    #[test]
    fn test_temp_to_index() {
        assert_eq!(temp_to_index(0, 0), 0);
        assert_eq!(temp_to_index(1, 0), 1);
        assert_eq!(temp_to_index(0, 1), 8);
        assert_eq!(temp_to_index(7, 7), 63);
    }

    #[test]
    fn test_add_and_get_temperature() {
        let mut chunk = Chunk::new(0, 0);

        // Initial temperature should be room temp (20.0)
        assert_eq!(get_temperature_at_pixel(&chunk, 0, 0), 20.0);

        // Add heat
        add_heat_at_pixel(&mut chunk, 0, 0, 100.0);
        assert_eq!(get_temperature_at_pixel(&chunk, 0, 0), 120.0);

        // All pixels in same 8x8 cell share temperature
        assert_eq!(get_temperature_at_pixel(&chunk, 7, 7), 120.0);

        // Different cell unaffected
        assert_eq!(get_temperature_at_pixel(&chunk, 8, 8), 20.0);
    }

    #[test]
    fn test_temperature_simulator_new() {
        let sim = TemperatureSimulator::new();
        assert_eq!(sim.update_counter, 0);
    }

    #[test]
    fn test_temperature_simulator_default() {
        let sim = TemperatureSimulator::default();
        assert_eq!(sim.update_counter, 0);
    }

    #[test]
    fn test_temperature_simulator_throttling() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        chunks.insert(IVec2::new(0, 0), Chunk::new(0, 0));
        let active_chunks = vec![IVec2::new(0, 0)];

        // First update should be skipped (throttle counter = 1)
        sim.update(&mut chunks, &active_chunks);
        assert_eq!(sim.update_counter, 1);

        // Second update should run (throttle counter resets)
        sim.update(&mut chunks, &active_chunks);
        assert_eq!(sim.update_counter, 0);
    }

    #[test]
    fn test_temperature_diffusion_hot_center() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        let mut chunk = Chunk::new(0, 0);

        // Set center cell to high temperature
        chunk.temperature[temp_to_index(4, 4)] = 100.0;
        chunks.insert(IVec2::new(0, 0), chunk);

        let active_chunks = vec![IVec2::new(0, 0)];

        // Run two updates to trigger diffusion
        sim.update(&mut chunks, &active_chunks);
        sim.update(&mut chunks, &active_chunks);

        let chunk = chunks.get(&IVec2::new(0, 0)).unwrap();

        // Center should have cooled down (diffused to neighbors)
        let center_temp = chunk.temperature[temp_to_index(4, 4)];
        assert!(
            center_temp < 100.0,
            "Center should cool down from 100, got {}",
            center_temp
        );

        // Neighbors should have warmed up from room temperature (20.0)
        let neighbor_temp = chunk.temperature[temp_to_index(4, 5)];
        assert!(
            neighbor_temp > 20.0,
            "Neighbor should warm up from 20, got {}",
            neighbor_temp
        );
    }

    #[test]
    fn test_temperature_diffusion_corner() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        let mut chunk = Chunk::new(0, 0);

        // Set corner cell to high temperature
        chunk.temperature[temp_to_index(0, 0)] = 100.0;
        chunks.insert(IVec2::new(0, 0), chunk);

        let active_chunks = vec![IVec2::new(0, 0)];

        // Run diffusion
        sim.update(&mut chunks, &active_chunks);
        sim.update(&mut chunks, &active_chunks);

        let chunk = chunks.get(&IVec2::new(0, 0)).unwrap();

        // Corner has only 2 neighbors (edge effect)
        let corner_temp = chunk.temperature[temp_to_index(0, 0)];
        assert!(corner_temp < 100.0, "Corner should cool down");
    }

    #[test]
    fn test_temperature_uniform_no_change() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        let chunk = Chunk::new(0, 0); // All temperatures at 20.0
        chunks.insert(IVec2::new(0, 0), chunk);

        let active_chunks = vec![IVec2::new(0, 0)];

        // Uniform temperature should remain uniform
        sim.update(&mut chunks, &active_chunks);
        sim.update(&mut chunks, &active_chunks);

        let chunk = chunks.get(&IVec2::new(0, 0)).unwrap();

        // All cells should still be at room temperature
        for temp in &chunk.temperature {
            assert!(
                (temp - 20.0).abs() < 0.001,
                "Uniform temp should not change"
            );
        }
    }

    #[test]
    fn test_temperature_empty_active_chunks() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        chunks.insert(IVec2::new(0, 0), Chunk::new(0, 0));

        // No active chunks - should not crash
        let active_chunks: Vec<IVec2> = vec![];
        sim.update(&mut chunks, &active_chunks);
        sim.update(&mut chunks, &active_chunks);
    }

    #[test]
    fn test_temperature_missing_chunk() {
        let mut sim = TemperatureSimulator::new();
        let mut chunks = HashMap::new();
        // No chunks at all

        // Active chunk not in map - should not crash
        let active_chunks = vec![IVec2::new(5, 5)];
        sim.update(&mut chunks, &active_chunks);
        sim.update(&mut chunks, &active_chunks);
    }
}
