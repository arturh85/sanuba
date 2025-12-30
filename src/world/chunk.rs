//! Chunk - 64x64 region of pixels

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use crate::simulation::MaterialId;

pub const CHUNK_SIZE: usize = 64;
pub const CHUNK_AREA: usize = CHUNK_SIZE * CHUNK_SIZE;

/// A single pixel in the world
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize)]
pub struct Pixel {
    /// Material type (0 = air)
    pub material_id: u16,
    /// State flags (updated this frame, burning, etc.)
    pub flags: u16,
}

impl Pixel {
    pub const AIR: Pixel = Pixel { material_id: 0, flags: 0 };
    
    pub fn new(material_id: u16) -> Self {
        Self { material_id, flags: 0 }
    }
    
    pub fn is_empty(&self) -> bool {
        self.material_id == MaterialId::AIR
    }
}

/// Flag bits for pixel state
pub mod pixel_flags {
    pub const UPDATED: u16 = 1 << 0;      // Already updated this frame
    pub const BURNING: u16 = 1 << 1;      // Currently on fire
    pub const FALLING: u16 = 1 << 2;      // In free-fall
}

/// A 64x64 region of the world
#[derive(Clone)]
pub struct Chunk {
    /// Chunk coordinates (in chunk space, not pixel space)
    pub x: i32,
    pub y: i32,

    /// Pixel data, row-major order
    /// Index = y * CHUNK_SIZE + x
    pixels: [Pixel; CHUNK_AREA],

    /// Temperature field (8x8 coarse grid)
    pub temperature: [f32; 64],

    /// Pressure field (8x8 coarse grid)
    pub pressure: [f32; 64],
    
    /// Whether chunk has been modified since last save
    pub dirty: bool,

    /// Bounding rect of modified pixels (for efficient updates)
    pub dirty_rect: Option<DirtyRect>,
}

#[derive(Clone, Copy, Debug)]
pub struct DirtyRect {
    pub min_x: usize,
    pub min_y: usize,
    pub max_x: usize,
    pub max_y: usize,
}

impl DirtyRect {
    pub fn new(x: usize, y: usize) -> Self {
        Self { min_x: x, min_y: y, max_x: x, max_y: y }
    }
    
    pub fn expand(&mut self, x: usize, y: usize) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }
}

impl Chunk {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y,
            pixels: [Pixel::AIR; CHUNK_AREA],
            temperature: [20.0; 64],  // Room temperature (Celsius)
            pressure: [1.0; 64],       // Atmospheric pressure
            dirty: false,
            dirty_rect: None,
        }
    }
    
    /// Get pixel at local coordinates (0-63, 0-63)
    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> Pixel {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE);
        self.pixels[y * CHUNK_SIZE + x]
    }
    
    /// Set pixel at local coordinates
    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE);
        self.pixels[y * CHUNK_SIZE + x] = pixel;
        self.mark_dirty(x, y);
    }
    
    /// Set pixel by material ID
    #[inline]
    pub fn set_material(&mut self, x: usize, y: usize, material_id: u16) {
        self.set_pixel(x, y, Pixel::new(material_id));
    }
    
    /// Swap two pixels (useful for falling simulation)
    #[inline]
    pub fn swap_pixels(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        let idx1 = y1 * CHUNK_SIZE + x1;
        let idx2 = y2 * CHUNK_SIZE + x2;
        self.pixels.swap(idx1, idx2);
        self.mark_dirty(x1, y1);
        self.mark_dirty(x2, y2);
    }
    
    fn mark_dirty(&mut self, x: usize, y: usize) {
        self.dirty = true;
        match &mut self.dirty_rect {
            Some(rect) => rect.expand(x, y),
            None => self.dirty_rect = Some(DirtyRect::new(x, y)),
        }
    }
    
    /// Clear dirty flags for new frame
    pub fn clear_dirty_rect(&mut self) {
        self.dirty_rect = None;
    }
    
    /// Clear all "updated this frame" flags from pixels
    pub fn clear_update_flags(&mut self) {
        for pixel in &mut self.pixels {
            pixel.flags &= !pixel_flags::UPDATED;
        }
    }
    
    /// Get raw pixel slice for rendering
    pub fn pixels(&self) -> &[Pixel] {
        &self.pixels
    }
    
    /// Get temperature at coarse grid position
    pub fn get_temperature(&self, cx: usize, cy: usize) -> f32 {
        self.temperature[cy * 8 + cx]
    }
    
    /// Set temperature at coarse grid position  
    pub fn set_temperature(&mut self, cx: usize, cy: usize, temp: f32) {
        self.temperature[cy * 8 + cx] = temp;
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

// Custom serialization for Chunk
impl Serialize for Chunk {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Chunk", 5)?;
        state.serialize_field("x", &self.x)?;
        state.serialize_field("y", &self.y)?;
        state.serialize_field("pixels", &self.pixels.as_slice())?;
        state.serialize_field("temperature", &self.temperature.as_slice())?;
        state.serialize_field("pressure", &self.pressure.as_slice())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Chunk {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ChunkVisitor;

        impl<'de> Visitor<'de> for ChunkVisitor {
            type Value = Chunk;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Chunk")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Chunk, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;
                let mut pixels: Option<Vec<Pixel>> = None;
                let mut temperature: Option<Vec<f32>> = None;
                let mut pressure: Option<Vec<f32>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "x" => x = Some(map.next_value()?),
                        "y" => y = Some(map.next_value()?),
                        "pixels" => pixels = Some(map.next_value()?),
                        "temperature" => temperature = Some(map.next_value()?),
                        "pressure" => pressure = Some(map.next_value()?),
                        _ => { let _: serde::de::IgnoredAny = map.next_value()?; }
                    }
                }

                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;
                let pixels_vec = pixels.ok_or_else(|| de::Error::missing_field("pixels"))?;
                let temp_vec = temperature.ok_or_else(|| de::Error::missing_field("temperature"))?;
                let press_vec = pressure.ok_or_else(|| de::Error::missing_field("pressure"))?;

                if pixels_vec.len() != CHUNK_AREA {
                    return Err(de::Error::invalid_length(pixels_vec.len(), &"4096 pixels"));
                }
                if temp_vec.len() != 64 {
                    return Err(de::Error::invalid_length(temp_vec.len(), &"64 temperature values"));
                }
                if press_vec.len() != 64 {
                    return Err(de::Error::invalid_length(press_vec.len(), &"64 pressure values"));
                }

                let mut pixels_array = [Pixel::AIR; CHUNK_AREA];
                pixels_array.copy_from_slice(&pixels_vec);

                let mut temp_array = [0.0; 64];
                temp_array.copy_from_slice(&temp_vec);

                let mut press_array = [0.0; 64];
                press_array.copy_from_slice(&press_vec);

                Ok(Chunk {
                    x,
                    y,
                    pixels: pixels_array,
                    temperature: temp_array,
                    pressure: press_array,
                    dirty: false,
                    dirty_rect: None,
                })
            }
        }

        deserializer.deserialize_struct("Chunk", &["x", "y", "pixels", "temperature", "pressure"], ChunkVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pixel_access() {
        let mut chunk = Chunk::new(0, 0);
        
        chunk.set_material(10, 20, 5);
        assert_eq!(chunk.get_pixel(10, 20).material_id, 5);
        
        chunk.set_material(0, 0, 1);
        chunk.set_material(63, 63, 2);
        assert_eq!(chunk.get_pixel(0, 0).material_id, 1);
        assert_eq!(chunk.get_pixel(63, 63).material_id, 2);
    }
    
    #[test]
    fn test_dirty_rect() {
        let mut chunk = Chunk::new(0, 0);
        
        chunk.set_material(10, 10, 1);
        chunk.set_material(50, 50, 1);
        
        let rect = chunk.dirty_rect.unwrap();
        assert_eq!(rect.min_x, 10);
        assert_eq!(rect.min_y, 10);
        assert_eq!(rect.max_x, 50);
        assert_eq!(rect.max_y, 50);
    }
}
