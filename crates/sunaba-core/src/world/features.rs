//! Post-generation feature placement
//!
//! Features are placed after base terrain/caves are generated, using
//! ContextScanner for context-aware placement decisions.

use crate::simulation::MaterialId;
use crate::world::chunk::Chunk;
use crate::world::context_scanner::{ContextScanner, PlacementPredicate};
use crate::world::generation::WorldGenerator;
use crate::world::worldgen_config::StalactiteConfig;
use fastnoise_lite::{FastNoiseLite, NoiseType};

/// Apply all enabled features to a freshly generated chunk
pub fn apply_features(chunk: &mut Chunk, chunk_x: i32, chunk_y: i32, generator: &WorldGenerator) {
    let config = generator.config();

    if config.features.stalactites.enabled {
        generate_stalactites(
            chunk,
            chunk_x,
            chunk_y,
            generator,
            &config.features.stalactites,
        );
    }

    // Structure generation - load templates once
    let templates = crate::world::structure_templates::create_builtin_templates();

    if config.features.structures.bridges.enabled {
        generate_bridges(
            chunk,
            chunk_x,
            chunk_y,
            generator,
            &config.features.structures.bridges,
            &templates,
        );
    }

    if config.features.structures.trees.enabled {
        generate_trees(
            chunk,
            chunk_x,
            chunk_y,
            generator,
            &config.features.structures.trees,
            &templates,
        );
    }

    if config.features.structures.ruins.enabled {
        generate_ruins(
            chunk,
            chunk_x,
            chunk_y,
            generator,
            &config.features.structures.ruins,
            &templates,
        );
    }
}

/// Generate stalactites in a chunk
fn generate_stalactites(
    chunk: &mut Chunk,
    chunk_x: i32,
    chunk_y: i32,
    generator: &WorldGenerator,
    config: &StalactiteConfig,
) {
    const CHUNK_SIZE: i32 = 64;

    // Skip chunks above minimum depth
    let chunk_world_y = chunk_y * CHUNK_SIZE;
    if chunk_world_y > config.min_depth {
        return;
    }

    // Create scanner and noise generators
    let scanner = ContextScanner::new(generator);
    let mut placement_noise =
        FastNoiseLite::with_seed((generator.seed as i32) + config.seed_offset);
    placement_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    let mut length_noise =
        FastNoiseLite::with_seed((generator.seed as i32) + config.seed_offset + 1);
    length_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    // Build predicate for stalactite placement
    let predicate = PlacementPredicate::All(vec![
        PlacementPredicate::IsCaveInterior,
        PlacementPredicate::AtCeiling,
        PlacementPredicate::MinAirBelow(config.min_air_below),
    ]);

    // Sample on grid with spacing
    let chunk_world_x = chunk_x * CHUNK_SIZE;

    for local_y in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
        for local_x in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
            let world_x = chunk_world_x + local_x;
            let world_y = chunk_world_y + local_y;

            // Check if position matches predicate
            if !scanner.matches(world_x, world_y, &predicate) {
                continue;
            }

            // Use noise to randomize placement
            let placement_value =
                placement_noise.get_noise_2d(world_x as f32 * 0.1, world_y as f32 * 0.1) as f64;

            if placement_value < (1.0 - config.placement_chance as f64) {
                continue;
            }

            // Determine stalactite length using noise
            let length_value =
                length_noise.get_noise_2d(world_x as f32 * 0.05, world_y as f32 * 0.05) as f64;

            // Map noise [-1, 1] to [min_length, max_length]
            let length_range = (config.max_length - config.min_length) as f64;
            let length = config.min_length + ((length_value + 1.0) * 0.5 * length_range) as i32;

            // Draw the stalactite
            draw_stalactite(chunk, chunk_x, chunk_y, world_x, world_y, length, config);
        }
    }
}

/// Draw a single stalactite starting at (x, y)
fn draw_stalactite(
    chunk: &mut Chunk,
    chunk_x: i32,
    chunk_y: i32,
    world_x: i32,
    world_y: i32,
    length: i32,
    config: &StalactiteConfig,
) {
    const CHUNK_SIZE: i32 = 64;
    let chunk_world_x = chunk_x * CHUNK_SIZE;
    let chunk_world_y = chunk_y * CHUNK_SIZE;

    for dy in 0..length {
        // Calculate width at this depth
        let width = if config.taper {
            // Linear taper: base_width at top, 1 at bottom
            let taper_factor = 1.0 - (dy as f32 / length as f32);
            (config.base_width as f32 * taper_factor).max(1.0) as i32
        } else {
            config.base_width
        };

        // Draw horizontal line at this depth
        for dx in -(width / 2)..=(width / 2) {
            let pixel_world_x = world_x + dx;
            let pixel_world_y = world_y - dy; // Grow downward

            // Convert to chunk-local coordinates
            let local_x = pixel_world_x - chunk_world_x;
            let local_y = pixel_world_y - chunk_world_y;

            // Check bounds
            if (0..CHUNK_SIZE).contains(&local_x) && (0..CHUNK_SIZE).contains(&local_y) {
                // Only place if currently air (don't overwrite solid material)
                if chunk.get_material(local_x as usize, local_y as usize) == MaterialId::AIR {
                    chunk.set_material(local_x as usize, local_y as usize, MaterialId::STONE);
                }
            }
        }
    }
}

/// Generate wooden bridges over gaps
fn generate_bridges(
    chunk: &mut Chunk,
    chunk_x: i32,
    chunk_y: i32,
    generator: &WorldGenerator,
    config: &crate::world::worldgen_config::BridgeConfig,
    templates: &std::collections::HashMap<&str, crate::world::structures::StructureVariants>,
) {
    const CHUNK_SIZE: i32 = 64;
    let chunk_world_y = chunk_y * CHUNK_SIZE;

    // Skip chunks above min depth
    if chunk_world_y > config.min_depth {
        return;
    }

    let scanner = ContextScanner::new(generator);
    let mut placement_noise =
        FastNoiseLite::with_seed((generator.seed as i32) + config.seed_offset);
    placement_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    let bridge_variants = templates.get("wooden_bridge").unwrap();
    let chunk_world_x = chunk_x * CHUNK_SIZE;

    for local_y in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
        for local_x in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
            let world_x = chunk_world_x + local_x;
            let world_y = chunk_world_y + local_y;

            // Check gap width at this position
            let gap_width = scanner.scan_gap_width(world_x, world_y);

            if gap_width < config.min_gap_width || gap_width > config.max_gap_width {
                continue;
            }

            // Check placement probability
            let placement_value =
                placement_noise.get_noise_2d(world_x as f32 * 0.1, world_y as f32 * 0.1) as f64;

            if placement_value < (1.0 - config.placement_chance as f64) {
                continue;
            }

            // Select appropriate bridge variant based on gap width
            let variant_noise =
                placement_noise.get_noise_2d(world_x as f32 * 0.05, world_y as f32 * 0.05) as f64;

            let template = bridge_variants.select_variant(variant_noise);

            // Validate placement
            if !crate::world::structure_placement::is_placement_valid(
                world_x, world_y, template, &scanner,
            ) {
                continue;
            }

            // Place the bridge
            crate::world::structure_placement::place_structure(
                chunk, chunk_x, chunk_y, world_x, world_y, template, &scanner,
            );
        }
    }
}

/// Generate surface trees (normal or marker based on cave detection)
fn generate_trees(
    chunk: &mut Chunk,
    chunk_x: i32,
    chunk_y: i32,
    generator: &WorldGenerator,
    config: &crate::world::worldgen_config::TreeConfig,
    templates: &std::collections::HashMap<&str, crate::world::structures::StructureVariants>,
) {
    const CHUNK_SIZE: i32 = 64;

    let scanner = ContextScanner::new(generator);
    let mut placement_noise =
        FastNoiseLite::with_seed((generator.seed as i32) + config.seed_offset);
    placement_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    let normal_variants = templates.get("tree_normal").unwrap();
    let marker_variants = templates.get("tree_marker").unwrap();

    let predicate = PlacementPredicate::All(vec![
        PlacementPredicate::IsSurface,
        PlacementPredicate::OnGround,
        PlacementPredicate::MinAirAbove(config.min_air_above),
    ]);

    let chunk_world_x = chunk_x * CHUNK_SIZE;
    let chunk_world_y = chunk_y * CHUNK_SIZE;

    for local_y in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
        for local_x in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
            let world_x = chunk_world_x + local_x;
            let world_y = chunk_world_y + local_y;

            // Check if position matches surface tree predicate
            if !scanner.matches(world_x, world_y, &predicate) {
                continue;
            }

            // Check placement probability
            let placement_value =
                placement_noise.get_noise_2d(world_x as f32 * 0.1, world_y as f32 * 0.1) as f64;

            if placement_value < (1.0 - config.placement_chance as f64) {
                continue;
            }

            // Detect cave below by scanning downward
            let has_cave_below =
                detect_cave_below(world_x, world_y, config.cave_scan_depth, &scanner);

            // Select tree type based on cave detection
            let tree_variants = if has_cave_below {
                let marker_chance = placement_noise
                    .get_noise_2d(world_x as f32 * 0.07, world_y as f32 * 0.07)
                    as f64;

                if marker_chance > (1.0 - config.marker_tree_chance as f64) {
                    marker_variants
                } else {
                    normal_variants
                }
            } else {
                normal_variants
            };

            let variant_noise =
                placement_noise.get_noise_2d(world_x as f32 * 0.05, world_y as f32 * 0.05) as f64;

            let template = tree_variants.select_variant(variant_noise);

            crate::world::structure_placement::place_structure(
                chunk, chunk_x, chunk_y, world_x, world_y, template, &scanner,
            );
        }
    }
}

/// Detect if there's a cave within scan depth below position
fn detect_cave_below(
    world_x: i32,
    world_y: i32,
    scan_depth: i32,
    scanner: &ContextScanner,
) -> bool {
    for dy in 1..=scan_depth {
        let check_y = world_y - dy;

        // Check for cave (air surrounded by solid)
        let is_air = scanner.get_material(world_x, check_y) == MaterialId::AIR;
        let has_ceiling = scanner.get_material(world_x, check_y + 1) != MaterialId::AIR;
        let has_floor = scanner.get_material(world_x, check_y - 1) != MaterialId::AIR;

        if is_air && has_ceiling && has_floor {
            return true; // Found enclosed cave space
        }
    }
    false
}

/// Generate underground ruins
fn generate_ruins(
    chunk: &mut Chunk,
    chunk_x: i32,
    chunk_y: i32,
    generator: &WorldGenerator,
    config: &crate::world::worldgen_config::RuinConfig,
    templates: &std::collections::HashMap<&str, crate::world::structures::StructureVariants>,
) {
    const CHUNK_SIZE: i32 = 64;
    let chunk_world_y = chunk_y * CHUNK_SIZE;

    // Skip chunks outside depth range
    if chunk_world_y > config.max_depth || chunk_world_y < config.min_depth {
        return;
    }

    let scanner = ContextScanner::new(generator);
    let mut placement_noise =
        FastNoiseLite::with_seed((generator.seed as i32) + config.seed_offset);
    placement_noise.set_noise_type(Some(NoiseType::OpenSimplex2));

    let predicate = PlacementPredicate::All(vec![
        PlacementPredicate::IsCaveInterior,
        PlacementPredicate::OnGround,
        PlacementPredicate::MinAirAbove(8),
    ]);

    let chunk_world_x = chunk_x * CHUNK_SIZE;

    for local_y in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
        for local_x in (0..CHUNK_SIZE).step_by(config.spacing as usize) {
            let world_x = chunk_world_x + local_x;
            let world_y = chunk_world_y + local_y;

            if !scanner.matches(world_x, world_y, &predicate) {
                continue;
            }

            let placement_value =
                placement_noise.get_noise_2d(world_x as f32 * 0.1, world_y as f32 * 0.1) as f64;

            if placement_value < (1.0 - config.placement_chance as f64) {
                continue;
            }

            // Randomly select wall or pillar
            let type_noise =
                placement_noise.get_noise_2d(world_x as f32 * 0.03, world_y as f32 * 0.03) as f64;

            let variants = if type_noise > 0.0 {
                templates.get("ruin_wall").unwrap()
            } else {
                templates.get("ruin_pillar").unwrap()
            };

            let variant_noise =
                placement_noise.get_noise_2d(world_x as f32 * 0.05, world_y as f32 * 0.05) as f64;

            let template = variants.select_variant(variant_noise);

            crate::world::structure_placement::place_structure(
                chunk, chunk_x, chunk_y, world_x, world_y, template, &scanner,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::generation::WorldGenerator;
    use crate::world::worldgen_config::WorldGenConfig;

    #[test]
    fn test_stalactite_config_default() {
        let config = StalactiteConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_length, 3);
        assert_eq!(config.max_length, 12);
        assert_eq!(config.spacing, 16);
        assert_eq!(config.base_width, 3);
        assert_eq!(config.min_air_below, 5);
        assert_eq!(config.placement_chance, 0.5);
        assert!(config.taper);
    }

    #[test]
    fn test_stalactites_disabled() {
        let mut config = WorldGenConfig::default();
        config.features.stalactites.enabled = false;

        let generator = WorldGenerator::from_config(42, config);
        let chunk = generator.generate_chunk(0, -10); // Deep underground

        // With stalactites disabled, should be deterministic
        let chunk2 = generator.generate_chunk(0, -10);

        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(
                    chunk.get_material(x, y),
                    chunk2.get_material(x, y),
                    "Chunk should be deterministic with features disabled"
                );
            }
        }
    }

    #[test]
    fn test_stalactites_deterministic() {
        let generator = WorldGenerator::new(42);

        let chunk1 = generator.generate_chunk(0, -20);
        let chunk2 = generator.generate_chunk(0, -20);

        // Same seed should produce identical stalactites
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(
                    chunk1.get_material(x, y),
                    chunk2.get_material(x, y),
                    "Stalactites should be deterministic"
                );
            }
        }
    }

    #[test]
    fn test_draw_stalactite_bounds_checking() {
        // Test that drawing near chunk edges doesn't panic
        let mut chunk = Chunk::new(0, 0);
        let config = StalactiteConfig::default();

        // Draw at edge of chunk
        draw_stalactite(&mut chunk, 0, 0, 0, 0, 10, &config);
        draw_stalactite(&mut chunk, 0, 0, 63, 63, 10, &config);

        // Should not panic
    }

    #[test]
    fn test_stalactite_only_places_in_air() {
        let mut chunk = Chunk::new(0, 0);
        let config = StalactiteConfig::default();

        // Fill chunk with stone
        for y in 0..64 {
            for x in 0..64 {
                chunk.set_material(x, y, MaterialId::STONE);
            }
        }

        // Try to draw stalactite - should not overwrite existing stone
        draw_stalactite(&mut chunk, 0, 0, 32, 32, 10, &config);

        // All pixels should still be stone
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(chunk.get_material(x, y), MaterialId::STONE);
            }
        }
    }
}
