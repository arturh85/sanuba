//! Texture variation system for adding visual depth to materials
//!
//! Provides deterministic per-pixel color variation based on world position
//! to break up flat single-color materials with texture, grain, and patterns.

use crate::MaterialId;

/// Simple hash function for deterministic noise from world coordinates
#[inline]
fn hash_coords(x: i32, y: i32) -> u32 {
    let mut h = x as u32;
    h ^= h << 13;
    h ^= h >> 17;
    h ^= h << 5;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= y as u32;
    h ^= h << 13;
    h ^= h >> 17;
    h ^= h << 5;
    h.wrapping_mul(0xc2b2ae35)
}

/// Get a deterministic random value [0.0, 1.0] from world coordinates
#[inline]
fn noise_at(x: i32, y: i32) -> f32 {
    (hash_coords(x, y) & 0xFFFF) as f32 / 65535.0
}

/// Get a deterministic random value [-1.0, 1.0] from world coordinates
#[inline]
fn signed_noise_at(x: i32, y: i32) -> f32 {
    noise_at(x, y) * 2.0 - 1.0
}

/// Apply texture variation to a base color based on material type and world position
///
/// This adds visual depth through:
/// - Per-pixel color variation (slight RGB shifts)
/// - Material-specific patterns (ore sparkles, stone grain, etc.)
/// - Edge highlighting (darker edges for depth perception)
pub fn apply_texture_variation(
    base_color: [u8; 4],
    material_id: u16,
    world_x: i32,
    world_y: i32,
    has_neighbor_above: bool,
    has_neighbor_left: bool,
) -> [u8; 4] {
    let mut r = base_color[0] as f32;
    let mut g = base_color[1] as f32;
    let mut b = base_color[2] as f32;
    let a = base_color[3];

    // Skip variation for air (transparent materials)
    if material_id == MaterialId::AIR {
        return base_color;
    }

    // 1. Base color variation (subtle per-pixel noise)
    let variation_strength = get_variation_strength(material_id);
    let noise = signed_noise_at(world_x, world_y);
    let variation = noise * variation_strength;

    r = (r + variation).clamp(0.0, 255.0);
    g = (g + variation).clamp(0.0, 255.0);
    b = (b + variation).clamp(0.0, 255.0);

    // 2. Material-specific patterns
    apply_material_pattern(material_id, &mut r, &mut g, &mut b, world_x, world_y);

    // 3. Edge darkening for depth (darken top and left edges slightly)
    if !has_neighbor_above || !has_neighbor_left {
        let darken = 0.85; // 15% darker
        if !has_neighbor_above {
            r *= darken;
            g *= darken;
            b *= darken;
        }
        if !has_neighbor_left {
            r *= 0.92; // Subtle left edge darkening
            g *= 0.92;
            b *= 0.92;
        }
    }

    [r as u8, g as u8, b as u8, a]
}

/// Get the base variation strength for a material type
fn get_variation_strength(material_id: u16) -> f32 {
    match material_id {
        // Solids - moderate grain
        MaterialId::STONE => 8.0,
        MaterialId::DIRT => 10.0,
        MaterialId::WOOD => 6.0,
        MaterialId::GLASS => 3.0,
        MaterialId::METAL => 5.0,
        MaterialId::MOSSY_STONE => 7.0,
        MaterialId::BASALT => 8.0,

        // Ores - slight sparkle variation
        MaterialId::COAL_ORE => 10.0,
        MaterialId::IRON_ORE => 6.0,
        MaterialId::COPPER_ORE => 6.0,
        MaterialId::GOLD_ORE => 5.0,
        MaterialId::CRYSTAL => 4.0,

        // Powders - more variation (rough surface)
        MaterialId::SAND => 12.0,
        MaterialId::ASH => 10.0,
        MaterialId::GUNPOWDER => 8.0,

        // Liquids - subtle shimmer
        MaterialId::WATER => 4.0,
        MaterialId::LAVA => 6.0,
        MaterialId::OIL => 5.0,
        MaterialId::ACID => 5.0,

        // Organic - moderate texture
        MaterialId::FLESH => 7.0,
        MaterialId::PLANT_MATTER => 9.0,
        MaterialId::BONE => 6.0,
        MaterialId::FRUIT => 8.0,

        // Gases - very subtle
        MaterialId::STEAM => 3.0,
        MaterialId::SMOKE => 4.0,
        MaterialId::POISON_GAS => 4.0,

        // Special materials
        MaterialId::FIRE => 8.0,     // Flicker
        MaterialId::OBSIDIAN => 4.0, // Smooth but with depth
        MaterialId::ICE => 3.0,      // Crystalline subtle variation
        MaterialId::GLOWING_MUSHROOM => 6.0,

        _ => 5.0, // Default moderate variation
    }
}

/// Apply material-specific visual patterns
fn apply_material_pattern(
    material_id: u16,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
    world_x: i32,
    world_y: i32,
) {
    match material_id {
        // Stone variants - occasional darker veins
        MaterialId::STONE | MaterialId::MOSSY_STONE | MaterialId::BASALT => {
            let vein_noise = noise_at(world_x / 3, world_y / 3);
            if vein_noise > 0.85 {
                let darken = 0.75;
                *r *= darken;
                *g *= darken;
                *b *= darken;
            }
        }

        // Ores - occasional bright sparkles
        MaterialId::IRON_ORE
        | MaterialId::COPPER_ORE
        | MaterialId::GOLD_ORE
        | MaterialId::CRYSTAL => {
            let sparkle_noise = noise_at(world_x * 3, world_y * 3);
            if sparkle_noise > 0.92 {
                let brighten = 1.3;
                *r *= brighten;
                *g *= brighten;
                *b *= brighten;
            }
        }

        // Sand - subtle dunes pattern (horizontal banding)
        MaterialId::SAND => {
            let band_noise = noise_at(world_x / 8, world_y / 2);
            let band_variation = (band_noise - 0.5) * 6.0;
            *r += band_variation;
            *g += band_variation;
            *b += band_variation;
        }

        // Wood - grain pattern (vertical lines with noise)
        MaterialId::WOOD => {
            let grain_noise = noise_at(world_x / 2, world_y / 8);
            let grain_variation = (grain_noise - 0.5) * 15.0;
            *r += grain_variation;
            *g += grain_variation * 0.8; // Less green variation
            *b += grain_variation * 0.6; // Even less blue
        }

        // Lava - flickering bright spots
        MaterialId::LAVA => {
            let flicker_noise = noise_at(world_x * 7, world_y * 7);
            if flicker_noise > 0.80 {
                let brighten = 1.25;
                *r *= brighten;
                *g *= brighten;
            }
        }

        // Water - subtle wavy shimmer
        MaterialId::WATER => {
            let wave_x = (world_x as f32 * 0.2).sin() * 0.5 + 0.5;
            let wave_y = (world_y as f32 * 0.15).sin() * 0.5 + 0.5;
            let wave_variation = (wave_x + wave_y - 1.0) * 3.0;
            *r += wave_variation;
            *g += wave_variation;
            *b += wave_variation;
        }

        // Dirt - occasional small rocks
        MaterialId::DIRT => {
            let rock_noise = noise_at(world_x * 5, world_y * 5);
            if rock_noise > 0.88 {
                // Small gray rocks
                *r = *r * 0.6 + 100.0;
                *g = *g * 0.6 + 100.0;
                *b = *b * 0.6 + 100.0;
            }
        }

        // Ice - crystalline highlights
        MaterialId::ICE => {
            let crystal_noise = noise_at(world_x * 4, world_y * 4);
            if crystal_noise > 0.90 {
                let brighten = 1.2;
                *r *= brighten;
                *g *= brighten;
                *b *= brighten;
            }
        }

        // Plant matter - organic irregular spots
        MaterialId::PLANT_MATTER | MaterialId::GLOWING_MUSHROOM => {
            let spot_noise = noise_at(world_x * 3, world_y * 3);
            if spot_noise > 0.75 {
                let darken = 0.85;
                *r *= darken;
                *g *= darken;
                *b *= darken;
            }
        }

        _ => {} // No special pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_coords_deterministic() {
        // Same coordinates should give same hash
        assert_eq!(hash_coords(10, 20), hash_coords(10, 20));

        // Different coordinates should (usually) give different hash
        assert_ne!(hash_coords(10, 20), hash_coords(10, 21));
    }

    #[test]
    fn test_noise_range() {
        // Noise should be in [0, 1] range
        for x in 0..100 {
            for y in 0..100 {
                let noise = noise_at(x, y);
                assert!((0.0..=1.0).contains(&noise));
            }
        }
    }

    #[test]
    fn test_signed_noise_range() {
        // Signed noise should be in [-1, 1] range
        for x in 0..100 {
            for y in 0..100 {
                let noise = signed_noise_at(x, y);
                assert!((-1.0..=1.0).contains(&noise));
            }
        }
    }

    #[test]
    fn test_texture_variation_preserves_alpha() {
        let base = [128, 128, 128, 200];
        let varied = apply_texture_variation(base, MaterialId::STONE, 0, 0, true, true);
        assert_eq!(varied[3], 200); // Alpha unchanged
    }

    #[test]
    fn test_texture_variation_stays_in_bounds() {
        let base = [200, 200, 200, 255];
        for x in 0..50 {
            for y in 0..50 {
                let varied = apply_texture_variation(base, MaterialId::STONE, x, y, true, true);
                // Verify variation doesn't exceed reasonable bounds from base
                // (u8 type guarantees 0-255 range, but we check variation magnitude)
                for i in 0..3 {
                    let delta = (varied[i] as i32 - base[i] as i32).abs();
                    assert!(
                        delta <= 60,
                        "Color channel {} varied by {} (too much)",
                        i,
                        delta
                    );
                }
            }
        }
    }

    #[test]
    fn test_different_materials_have_different_variation() {
        let base = [128, 128, 128, 255];
        let stone = apply_texture_variation(base, MaterialId::STONE, 10, 10, true, true);
        let sand = apply_texture_variation(base, MaterialId::SAND, 10, 10, true, true);
        // Different materials should produce different results due to different variation strengths
        assert_ne!(stone, sand);
    }
}
