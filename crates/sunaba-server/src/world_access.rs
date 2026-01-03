//! WorldAccess trait implementation for SpacetimeDB
//!
//! Provides creature sensing and world interaction over SpacetimeDB tables.

use std::cell::RefCell;
use std::collections::HashMap;

use glam::Vec2;
use spacetimedb::{ReducerContext, Table};
use sunaba_creature::{WorldAccess, WorldMutAccess};
use sunaba_simulation::{MaterialType, Materials, Pixel, CHUNK_SIZE};

use crate::encoding::{decode_chunk_pixels, encode_chunk_pixels};
use crate::{chunk_data, ChunkData};

/// Cached chunk data
struct CachedChunk {
    pixels: Vec<Pixel>,
    dirty: bool,
    db_id: u64,
}

/// WorldAccess implementation over SpacetimeDB tables
///
/// Uses interior mutability for chunk caching since WorldAccess trait
/// methods take &self for compatibility with existing creature code.
pub struct SpacetimeWorldAccess<'a> {
    ctx: &'a ReducerContext,
    materials: Materials,
    /// Cached decoded chunks (uses RefCell for interior mutability)
    chunk_cache: RefCell<HashMap<(i32, i32), CachedChunk>>,
}

impl<'a> SpacetimeWorldAccess<'a> {
    /// Create new world access wrapper
    pub fn new(ctx: &'a ReducerContext) -> Self {
        Self {
            ctx,
            materials: Materials::default(),
            chunk_cache: RefCell::new(HashMap::new()),
        }
    }

    /// Convert world coordinates to chunk coordinates and local offset
    fn world_to_chunk(x: i32, y: i32) -> (i32, i32, usize, usize) {
        let chunk_x = x.div_euclid(CHUNK_SIZE as i32);
        let chunk_y = y.div_euclid(CHUNK_SIZE as i32);
        let local_x = x.rem_euclid(CHUNK_SIZE as i32) as usize;
        let local_y = y.rem_euclid(CHUNK_SIZE as i32) as usize;
        (chunk_x, chunk_y, local_x, local_y)
    }

    /// Load and cache a chunk from the database
    fn load_chunk(&self, chunk_x: i32, chunk_y: i32) -> bool {
        let mut cache = self.chunk_cache.borrow_mut();
        if cache.contains_key(&(chunk_x, chunk_y)) {
            return true;
        }

        // Query chunk from database
        let chunk_opt = self
            .ctx
            .db
            .chunk_data()
            .iter()
            .find(|c| c.x == chunk_x && c.y == chunk_y);

        if let Some(chunk) = chunk_opt
            && let Ok(pixels) = decode_chunk_pixels(&chunk.pixel_data) {
                cache.insert(
                    (chunk_x, chunk_y),
                    CachedChunk {
                        pixels,
                        dirty: false,
                        db_id: chunk.id,
                    },
                );
                return true;
            }

        // Create empty chunk if not found
        cache.insert(
            (chunk_x, chunk_y),
            CachedChunk {
                pixels: vec![Pixel::new(0); CHUNK_SIZE * CHUNK_SIZE],
                dirty: false,
                db_id: 0, // Will need to insert
            },
        );

        true
    }

    /// Get pixel from cache
    fn get_cached_pixel(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        local_x: usize,
        local_y: usize,
    ) -> Option<Pixel> {
        self.load_chunk(chunk_x, chunk_y);
        let cache = self.chunk_cache.borrow();
        cache.get(&(chunk_x, chunk_y)).map(|chunk| {
            let idx = local_y * CHUNK_SIZE + local_x;
            chunk.pixels.get(idx).copied().unwrap_or_else(|| Pixel::new(0))
        })
    }

    /// Set pixel in cache and mark dirty
    fn set_cached_pixel(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        local_x: usize,
        local_y: usize,
        pixel: Pixel,
    ) {
        self.load_chunk(chunk_x, chunk_y);
        let mut cache = self.chunk_cache.borrow_mut();
        if let Some(chunk) = cache.get_mut(&(chunk_x, chunk_y)) {
            let idx = local_y * CHUNK_SIZE + local_x;
            if idx < chunk.pixels.len() {
                chunk.pixels[idx] = pixel;
                chunk.dirty = true;
            }
        }
    }

    /// Commit all dirty chunks back to database
    #[allow(dead_code)]
    pub fn commit_changes(&self) -> Result<(), String> {
        let cache = self.chunk_cache.borrow();
        for ((chunk_x, chunk_y), cached) in cache.iter() {
            if cached.dirty {
                let pixel_data = encode_chunk_pixels(&cached.pixels)?;

                if cached.db_id != 0 {
                    // Update existing chunk
                    if let Some(existing) = self
                        .ctx
                        .db
                        .chunk_data()
                        .iter()
                        .find(|c| c.x == *chunk_x && c.y == *chunk_y)
                    {
                        self.ctx.db.chunk_data().id().update(ChunkData {
                            pixel_data,
                            dirty: true,
                            ..existing
                        });
                    }
                } else {
                    // Insert new chunk
                    let _ = self.ctx.db.chunk_data().try_insert(ChunkData {
                        id: 0,
                        x: *chunk_x,
                        y: *chunk_y,
                        pixel_data,
                        dirty: true,
                        last_modified_tick: 0,
                    });
                }
            }
        }
        Ok(())
    }

    /// Check if material type is solid-like (blocks movement)
    fn is_blocking_material(&self, material_id: u16) -> bool {
        if material_id == 0 {
            return false; // Air
        }
        let material = self.materials.get(material_id);
        matches!(
            material.material_type,
            MaterialType::Solid | MaterialType::Powder
        )
    }
}

impl<'a> WorldAccess for SpacetimeWorldAccess<'a> {
    fn get_pixel(&self, x: i32, y: i32) -> Option<Pixel> {
        let (chunk_x, chunk_y, local_x, local_y) = Self::world_to_chunk(x, y);
        self.get_cached_pixel(chunk_x, chunk_y, local_x, local_y)
    }

    fn get_temperature_at_pixel(&self, _x: i32, _y: i32) -> f32 {
        20.0 // Default room temperature
    }

    fn get_light_at(&self, _x: i32, _y: i32) -> Option<u8> {
        Some(15) // Full brightness
    }

    fn materials(&self) -> &Materials {
        &self.materials
    }

    fn is_solid_at(&self, x: i32, y: i32) -> bool {
        if let Some(pixel) = self.get_pixel(x, y) {
            self.is_blocking_material(pixel.material_id)
        } else {
            true // Out of bounds = solid
        }
    }

    fn check_circle_collision(&self, x: f32, y: f32, radius: f32) -> bool {
        let r = radius.ceil() as i32;
        let cx = x.round() as i32;
        let cy = y.round() as i32;
        let r_squared = radius * radius;

        for dy in -r..=r {
            for dx in -r..=r {
                if (dx * dx + dy * dy) as f32 <= r_squared
                    && self.is_solid_at(cx + dx, cy + dy) {
                        return true;
                    }
            }
        }
        false
    }

    fn raycast(&self, from: Vec2, direction: Vec2, max_distance: f32) -> Option<(i32, i32, u16)> {
        if direction.length_squared() < 0.0001 {
            return None;
        }

        let step = direction.normalize();
        let mut pos = from;
        let mut dist = 0.0;

        while dist < max_distance {
            let px = pos.x.round() as i32;
            let py = pos.y.round() as i32;

            if let Some(pixel) = self.get_pixel(px, py) {
                if pixel.material_id != 0 {
                    // Not air
                    return Some((px, py, pixel.material_id));
                }
            } else {
                return None; // Out of bounds
            }

            pos += step;
            dist += 1.0;
        }
        None
    }

    fn get_pressure_at(&self, _x: i32, _y: i32) -> f32 {
        1.0 // Atmospheric pressure
    }

    fn is_creature_grounded(&self, positions: &[(Vec2, f32)]) -> bool {
        for (pos, radius) in positions {
            // Check below each body part
            let check_y = (pos.y + radius + 1.0).round() as i32;
            let check_x = pos.x.round() as i32;

            if self.is_solid_at(check_x, check_y) {
                return true;
            }
        }
        false
    }

    fn get_blocking_pixel(
        &self,
        from: Vec2,
        direction: Vec2,
        _radius: f32,
        max_distance: f32,
    ) -> Option<(i32, i32, u16)> {
        // Simplified: just use raycast for now
        self.raycast(from, direction, max_distance)
    }
}

impl<'a> WorldMutAccess for SpacetimeWorldAccess<'a> {
    fn set_pixel(&mut self, x: i32, y: i32, material_id: u16) {
        self.set_pixel_full(x, y, Pixel::new(material_id));
    }

    fn set_pixel_full(&mut self, x: i32, y: i32, pixel: Pixel) {
        let (chunk_x, chunk_y, local_x, local_y) = Self::world_to_chunk(x, y);
        self.set_cached_pixel(chunk_x, chunk_y, local_x, local_y, pixel);
    }
}
