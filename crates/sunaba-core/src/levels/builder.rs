//! Level builder API for declarative level construction
//!
//! Provides a fluent API to eliminate boilerplate in demo level generators.
//! Instead of manually iterating over chunks and pixels, use method chaining:
//!
//! ```ignore
//! LevelBuilder::new()
//!     .chunk_grid(-2..=2, -2..=2)
//!     .bedrock_foundation()
//!     .fill_layer(0..16, MaterialId::STONE)
//!     .in_chunk(0, 0, |chunk| {
//!         chunk.fill_rect(10..54, 16..40, MaterialId::WATER);
//!     })
//!     .build(world);
//! ```

use crate::simulation::MaterialId;
use crate::world::{CHUNK_SIZE, Chunk, World};
use std::collections::HashMap;
use std::ops::{Range, RangeInclusive};

/// Builder for constructing levels declaratively
pub struct LevelBuilder {
    /// Chunk grid bounds (inclusive ranges)
    chunk_bounds_x: RangeInclusive<i32>,
    chunk_bounds_y: RangeInclusive<i32>,

    /// Created chunks, keyed by (cx, cy)
    chunks: HashMap<(i32, i32), Chunk>,
}

impl LevelBuilder {
    /// Create a new level builder with default settings
    pub fn new() -> Self {
        Self {
            chunk_bounds_x: -2..=2, // Standard 5x5 grid
            chunk_bounds_y: -2..=2,
            chunks: HashMap::new(),
        }
    }

    /// Set the chunk grid bounds (inclusive)
    ///
    /// # Example
    /// ```ignore
    /// builder.chunk_grid(-2..=2, -2..=2)  // 5x5 grid
    /// ```
    pub fn chunk_grid(
        mut self,
        cx_range: RangeInclusive<i32>,
        cy_range: RangeInclusive<i32>,
    ) -> Self {
        self.chunk_bounds_x = cx_range;
        self.chunk_bounds_y = cy_range;
        self
    }

    /// Create bedrock foundation (used in 66% of levels)
    ///
    /// - cy == -2: Full bedrock chunk
    /// - cy == -1: Bedrock in bottom 8 pixels
    pub fn bedrock_foundation(mut self) -> Self {
        for cy in self.chunk_bounds_y.clone() {
            for cx in self.chunk_bounds_x.clone() {
                let chunk = self.get_or_create_chunk(cx, cy);

                if cy == -2 {
                    // Full bedrock chunk
                    for y in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            chunk.set_material(x, y, MaterialId::BEDROCK);
                        }
                    }
                } else if cy == -1 {
                    // Bedrock bottom 8 pixels
                    for y in 0..8 {
                        for x in 0..CHUNK_SIZE {
                            chunk.set_material(x, y, MaterialId::BEDROCK);
                        }
                    }
                }
            }
        }
        self
    }

    /// Fill a horizontal layer across all chunks (world Y coordinates)
    ///
    /// # Example
    /// ```ignore
    /// builder.fill_layer(0..16, MaterialId::STONE)  // Stone ground
    /// ```
    pub fn fill_layer(mut self, y_range: Range<usize>, material: u16) -> Self {
        for cy in self.chunk_bounds_y.clone() {
            for cx in self.chunk_bounds_x.clone() {
                let chunk = self.get_or_create_chunk(cx, cy);

                // Fill local coordinates in this chunk
                for y in y_range.clone() {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, material);
                    }
                }
            }
        }
        self
    }

    /// Fill a rectangle across all chunks (world-relative pixel coordinates)
    ///
    /// Coordinates are world-relative (center of world = 0,0).
    /// Automatically maps to correct chunks.
    ///
    /// # Example
    /// ```ignore
    /// builder.fill_rect(-20..20, 20..40, MaterialId::WATER)
    /// ```
    pub fn fill_rect(mut self, x_range: Range<i32>, y_range: Range<i32>, material: u16) -> Self {
        for cy in self.chunk_bounds_y.clone() {
            for cx in self.chunk_bounds_x.clone() {
                // Calculate chunk's world pixel bounds
                let chunk_world_min_x = cx * CHUNK_SIZE as i32;
                let chunk_world_max_x = chunk_world_min_x + CHUNK_SIZE as i32;
                let chunk_world_min_y = cy * CHUNK_SIZE as i32;
                let chunk_world_max_y = chunk_world_min_y + CHUNK_SIZE as i32;

                // Check if rect overlaps this chunk
                if x_range.end <= chunk_world_min_x || x_range.start >= chunk_world_max_x {
                    continue;
                }
                if y_range.end <= chunk_world_min_y || y_range.start >= chunk_world_max_y {
                    continue;
                }

                // Calculate local coordinates for this chunk
                let local_min_x = (x_range.start - chunk_world_min_x).max(0) as usize;
                let local_max_x = (x_range.end - chunk_world_min_x).min(CHUNK_SIZE as i32) as usize;
                let local_min_y = (y_range.start - chunk_world_min_y).max(0) as usize;
                let local_max_y = (y_range.end - chunk_world_min_y).min(CHUNK_SIZE as i32) as usize;

                let chunk = self.get_or_create_chunk(cx, cy);
                for y in local_min_y..local_max_y {
                    for x in local_min_x..local_max_x {
                        chunk.set_material(x, y, material);
                    }
                }
            }
        }
        self
    }

    /// Execute a function on a specific chunk (for chunk-specific logic)
    ///
    /// # Example
    /// ```ignore
    /// builder.in_chunk(0, 0, |chunk| {
    ///     chunk.fill_rect(10..54, 16..40, MaterialId::WATER);
    /// })
    /// ```
    pub fn in_chunk<F>(mut self, cx: i32, cy: i32, f: F) -> Self
    where
        F: FnOnce(&mut ChunkBuilder),
    {
        let chunk = self.get_or_create_chunk(cx, cy);
        let mut builder = ChunkBuilder { chunk };
        f(&mut builder);
        self
    }

    /// Create a chamber (hollow rectangle) with walls and floor
    ///
    /// # Example
    /// ```ignore
    /// builder.chamber(-30..30, 0..50, 4, MaterialId::STONE)
    /// ```
    pub fn chamber(
        mut self,
        x_range: Range<i32>,
        y_range: Range<i32>,
        wall_thickness: usize,
        wall_material: u16,
    ) -> Self {
        let thickness = wall_thickness as i32;

        // Floor
        self = self.fill_rect(
            x_range.start..x_range.end,
            y_range.start..y_range.start + thickness,
            wall_material,
        );

        // Ceiling
        self = self.fill_rect(
            x_range.start..x_range.end,
            y_range.end - thickness..y_range.end,
            wall_material,
        );

        // Left wall
        self = self.fill_rect(
            x_range.start..x_range.start + thickness,
            y_range.start..y_range.end,
            wall_material,
        );

        // Right wall
        self = self.fill_rect(
            x_range.end - thickness..x_range.end,
            y_range.start..y_range.end,
            wall_material,
        );

        self
    }

    /// Create a pyramid (triangular pile)
    ///
    /// # Example
    /// ```ignore
    /// builder.pyramid(0, 16, 32, MaterialId::SAND)
    /// ```
    pub fn pyramid(mut self, center_x: i32, base_y: i32, height: usize, material: u16) -> Self {
        for offset_y in 0..height {
            let y = base_y + offset_y as i32;
            let width = height - offset_y;
            let half_width = width as i32 / 2;

            self = self.fill_rect(
                center_x - half_width..center_x + half_width,
                y..y + 1,
                material,
            );
        }
        self
    }

    /// Finalize and add all chunks to the world
    ///
    /// This clears all existing chunks in the world first.
    pub fn build(self, world: &mut World) {
        world.clear_all_chunks();

        // Add all created chunks to the world
        for chunk in self.chunks.into_values() {
            world.add_chunk(chunk);
        }
    }

    /// Get or create a chunk at the given coordinates
    fn get_or_create_chunk(&mut self, cx: i32, cy: i32) -> &mut Chunk {
        self.chunks
            .entry((cx, cy))
            .or_insert_with(|| Chunk::new(cx, cy))
    }
}

impl Default for LevelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for chunk-specific operations (local coordinates 0-63)
pub struct ChunkBuilder<'a> {
    chunk: &'a mut Chunk,
}

impl<'a> ChunkBuilder<'a> {
    /// Set a single pixel (local coordinates 0-63)
    ///
    /// # Example
    /// ```ignore
    /// chunk.set_material(32, 32, MaterialId::LAVA);
    /// ```
    pub fn set_material(&mut self, x: usize, y: usize, material: u16) {
        if x < CHUNK_SIZE && y < CHUNK_SIZE {
            self.chunk.set_material(x, y, material);
        }
    }

    /// Fill a rectangle within this chunk (local coordinates 0-63)
    ///
    /// # Example
    /// ```ignore
    /// chunk.fill_rect(10..54, 16..40, MaterialId::WATER);
    /// ```
    pub fn fill_rect(&mut self, x_range: Range<usize>, y_range: Range<usize>, material: u16) {
        for y in y_range {
            for x in x_range.clone() {
                if x < CHUNK_SIZE && y < CHUNK_SIZE {
                    self.chunk.set_material(x, y, material);
                }
            }
        }
    }

    /// Fill a circle within this chunk
    ///
    /// # Example
    /// ```ignore
    /// chunk.fill_circle(32, 32, 10, MaterialId::LAVA);
    /// ```
    pub fn fill_circle(&mut self, cx: usize, cy: usize, radius: usize, material: u16) {
        let radius_sq = (radius * radius) as i32;

        for y in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let dx = x as i32 - cx as i32;
                let dy = y as i32 - cy as i32;

                if dx * dx + dy * dy <= radius_sq {
                    self.chunk.set_material(x, y, material);
                }
            }
        }
    }

    /// Fill using a pattern function (for complex shapes)
    ///
    /// The function receives (x, y) coordinates and returns a material ID.
    /// Return MaterialId::AIR to leave pixels empty.
    ///
    /// # Example
    /// ```ignore
    /// chunk.fill_pattern(0..64, 0..64, |x, y| {
    ///     if (x + y) % 2 == 0 {
    ///         MaterialId::STONE
    ///     } else {
    ///         MaterialId::DIRT
    ///     }
    /// });
    /// ```
    pub fn fill_pattern<F>(&mut self, x_range: Range<usize>, y_range: Range<usize>, f: F)
    where
        F: Fn(usize, usize) -> u16,
    {
        for y in y_range {
            for x in x_range.clone() {
                if x < CHUNK_SIZE && y < CHUNK_SIZE {
                    let material = f(x, y);
                    if material != MaterialId::AIR {
                        self.chunk.set_material(x, y, material);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_builder_creates_chunks() {
        let mut world = World::default();

        LevelBuilder::new()
            .chunk_grid(-1..=1, -1..=1) // 3x3 grid
            .fill_layer(0..8, MaterialId::STONE)
            .build(&mut world);

        // Verify chunks were created
        assert!(world.get_pixel(0, 0).is_some());
        assert!(world.get_pixel(32, 32).is_some());
    }

    #[test]
    fn test_fill_rect_world_coords() {
        let mut world = World::default();

        LevelBuilder::new()
            .chunk_grid(0..=0, 0..=0)
            .fill_rect(10..20, 10..20, MaterialId::STONE)
            .build(&mut world);

        // Check that rect was filled
        assert_eq!(
            world.get_pixel(15, 15).unwrap().material_id,
            MaterialId::STONE
        );
        assert_eq!(world.get_pixel(5, 5).unwrap().material_id, MaterialId::AIR);
    }

    #[test]
    fn test_in_chunk() {
        let mut world = World::default();

        LevelBuilder::new()
            .chunk_grid(0..=0, 0..=0)
            .in_chunk(0, 0, |chunk| {
                chunk.fill_circle(32, 32, 10, MaterialId::LAVA);
            })
            .build(&mut world);

        // Check that circle was filled
        assert_eq!(
            world.get_pixel(32, 32).unwrap().material_id,
            MaterialId::LAVA
        );
    }

    #[test]
    fn test_bedrock_foundation() {
        let mut world = World::default();

        LevelBuilder::new()
            .chunk_grid(-2..=2, -2..=2)
            .bedrock_foundation()
            .build(&mut world);

        // Check bottom chunk is full bedrock
        let bottom_y = -2 * CHUNK_SIZE as i32;
        assert_eq!(
            world.get_pixel(0, bottom_y).unwrap().material_id,
            MaterialId::BEDROCK
        );
    }
}
