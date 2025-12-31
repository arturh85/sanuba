//! Demo level generators

use crate::world::{World, Chunk, CHUNK_SIZE};
use crate::simulation::MaterialId;

/// Level 1: Basic Physics Playground
/// Sand pile (left), water pool (right), stone platforms
pub fn generate_level_1_basic_physics(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // Stone ground (bottom 16 pixels)
            for y in 0..16 {
                for x in 0..CHUNK_SIZE {
                    chunk.set_material(x, y, MaterialId::STONE);
                }
            }

            // Sand pile (left side, pyramid shape)
            if cx == -1 && cy == 0 {
                for base_y in 16..48 {
                    let height = 48 - base_y;
                    let width = height / 2;
                    for dx in 0..width {
                        if 20 + dx < CHUNK_SIZE && 20 + height - dx < CHUNK_SIZE {
                            chunk.set_material(20 + dx, base_y, MaterialId::SAND);
                            chunk.set_material(20 + height - dx, base_y, MaterialId::SAND);
                        }
                    }
                }
            }

            // Water pool (right side)
            if cx == 1 && cy == 0 {
                for x in 10..54 {
                    for y in 16..40 {
                        chunk.set_material(x, y, MaterialId::WATER);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 2: Inferno
/// Dense forest of wood columns, fire at bottom
pub fn generate_level_2_inferno(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // Stone base
            for y in 0..8 {
                for x in 0..CHUNK_SIZE {
                    chunk.set_material(x, y, MaterialId::STONE);
                }
            }

            // Wood columns (every 12 pixels)
            for wood_x in (4..CHUNK_SIZE).step_by(12) {
                for y in 8..56 {
                    chunk.set_material(wood_x, y, MaterialId::WOOD);
                    if wood_x + 1 < CHUNK_SIZE {
                        chunk.set_material(wood_x + 1, y, MaterialId::WOOD);
                    }
                }
            }

            // Fire at bottom (center chunk only)
            if cx == 0 && cy == 0 {
                for x in 28..36 {
                    chunk.set_material(x, 8, MaterialId::FIRE);
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 3: Lava Meets Water
/// Stone basin with lava (left) and water (right) separated by wall
pub fn generate_level_3_lava_water(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            if cx == 0 && cy == 0 {
                // Create stone basin
                // Bottom
                for x in 0..CHUNK_SIZE {
                    for y in 0..4 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Left wall
                for y in 4..40 {
                    for x in 0..4 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Right wall
                for y in 4..40 {
                    for x in 60..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Center divider (removable)
                for y in 4..36 {
                    for x in 30..34 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Lava (left chamber)
                for x in 4..30 {
                    for y in 4..32 {
                        chunk.set_material(x, y, MaterialId::LAVA);
                    }
                }

                // Water (right chamber)
                for x in 34..60 {
                    for y in 4..32 {
                        chunk.set_material(x, y, MaterialId::WATER);
                    }
                }
            } else {
                // Ground for other chunks
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 4: Material Showcase
/// Vertical chambers, each with a different material
pub fn generate_level_4_showcase(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            if cx == 0 && cy == 0 {
                // Stone base and dividers
                for y in 0..4 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Create 8 vertical chambers
                let materials = [
                    MaterialId::STONE,
                    MaterialId::SAND,
                    MaterialId::WATER,
                    MaterialId::WOOD,
                    MaterialId::FIRE,
                    MaterialId::SMOKE,
                    MaterialId::LAVA,
                    MaterialId::OIL,
                ];

                for (i, &mat_id) in materials.iter().enumerate() {
                    let chamber_x = i * 8;

                    // Divider walls
                    if chamber_x > 0 {
                        for y in 4..48 {
                            chunk.set_material(chamber_x, y, MaterialId::STONE);
                        }
                    }

                    // Fill chamber with material
                    for x in (chamber_x + 1)..(chamber_x + 7).min(CHUNK_SIZE) {
                        for y in 4..40 {
                            chunk.set_material(x, y, mat_id);
                        }
                    }
                }
            } else {
                // Ground
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 5: Powder Paradise
/// Multiple sand piles and powder materials
pub fn generate_level_5_powder(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // Stone base
            for y in 0..12 {
                for x in 0..CHUNK_SIZE {
                    chunk.set_material(x, y, MaterialId::STONE);
                }
            }

            // Sand piles at different positions
            if cx == -1 && cy == 0 {
                for x in 10..30 {
                    for y in 12..(12 + (x - 10)) {
                        chunk.set_material(x, y, MaterialId::SAND);
                    }
                }
            }

            if cx == 0 && cy == 0 {
                // Central tall pile
                for x in 24..40 {
                    let height = 20 + ((x as i32 - 32).abs() as usize * 2);
                    for y in 12..(12 + height).min(CHUNK_SIZE) {
                        chunk.set_material(x, y, MaterialId::SAND);
                    }
                }
            }

            if cx == 1 && cy == 0 {
                for x in 34..54 {
                    for y in 12..(12 + (54 - x)) {
                        chunk.set_material(x, y, MaterialId::SAND);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 6: Liquid Lab
/// Water and oil pools demonstrating liquid physics
pub fn generate_level_6_liquids(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // Stone base with steps
            for y in 0..8 {
                for x in 0..CHUNK_SIZE {
                    chunk.set_material(x, y, MaterialId::STONE);
                }
            }

            if cx == 0 && cy == 0 {
                // Create stepped platforms
                for x in 0..20 {
                    for y in 8..16 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                for x in 44..CHUNK_SIZE {
                    for y in 8..16 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Water on left platform
                for x in 2..18 {
                    for y in 16..28 {
                        chunk.set_material(x, y, MaterialId::WATER);
                    }
                }

                // Oil on right platform
                for x in 46..62 {
                    for y in 16..28 {
                        chunk.set_material(x, y, MaterialId::OIL);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 7: Steam Engine
/// Lava heats water to create steam
pub fn generate_level_7_steam(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            if cx == 0 && cy == 0 {
                // Stone chamber
                // Bottom
                for y in 0..4 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Walls
                for y in 4..50 {
                    for x in 0..4 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                    for x in 60..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Lava at bottom
                for x in 4..60 {
                    for y in 4..12 {
                        chunk.set_material(x, y, MaterialId::LAVA);
                    }
                }

                // Water above lava
                for x in 4..60 {
                    for y in 12..28 {
                        chunk.set_material(x, y, MaterialId::WATER);
                    }
                }
            } else {
                // Ground
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 8: Volcano
/// Lava-filled mountain that can erupt
pub fn generate_level_8_volcano(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            if cx == 0 && cy == 0 {
                // Create mountain shape
                for y in 0..48 {
                    let width = (48 - y) / 2;
                    for dx in 0..width {
                        let left_x = 32 - dx;
                        let right_x = 32 + dx;

                        if left_x < CHUNK_SIZE && y < CHUNK_SIZE {
                            // Outer mountain (stone)
                            if dx >= 4 {
                                chunk.set_material(left_x, y, MaterialId::STONE);
                            }
                        }

                        if right_x < CHUNK_SIZE && y < CHUNK_SIZE {
                            if dx >= 4 {
                                chunk.set_material(right_x, y, MaterialId::STONE);
                            }
                        }

                        // Inner chamber (lava)
                        if dx < 4 && y < 44 {
                            if left_x < CHUNK_SIZE {
                                chunk.set_material(left_x, y, MaterialId::LAVA);
                            }
                            if right_x < CHUNK_SIZE {
                                chunk.set_material(right_x, y, MaterialId::LAVA);
                            }
                        }
                    }
                }

                // Thin cap at top (removable to trigger eruption)
                for x in 28..36 {
                    chunk.set_material(x, 44, MaterialId::STONE);
                    chunk.set_material(x, 45, MaterialId::STONE);
                }
            } else {
                // Ground for other chunks
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 9: Bridge Demolition
/// Stone bridge supported by pillars - remove pillars to collapse bridge
pub fn generate_level_9_bridge(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER (required for anchoring)
            if cy == -2 {
                // Full bedrock chunk at bottom
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

            // CENTER CHUNK - Bridge structure
            if cx == 0 && cy == 0 {
                // Left pillar (x: 10-13, height: 20)
                for x in 10..=13 {
                    for y in 0..20 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Right pillar (x: 50-53, height: 20)
                for x in 50..=53 {
                    for y in 0..20 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Bridge span (connecting the two pillars at height 20-23)
                for x in 10..=53 {
                    for y in 20..=23 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 10: Tower Collapse
/// Three towers of different sizes to demonstrate small vs large debris threshold
pub fn generate_level_10_towers(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Three towers of different sizes
            if cx == 0 && cy == 0 {
                // Tower 1 (left): 2 pixels wide, 20 tall (40 pixels total - small debris)
                for x in 14..=15 {
                    for y in 0..20 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Tower 2 (center): 4 pixels wide, 30 tall (120 pixels total - large debris)
                for x in 30..=33 {
                    for y in 0..30 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Tower 3 (right): 6 pixels wide, 40 tall (240 pixels total - large debris)
                for x in 48..=53 {
                    for y in 0..40 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 11: Floating Islands
/// Multiple stone platforms supported by thin columns
pub fn generate_level_11_islands(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Islands with support columns
            if cx == 0 && cy == 0 {
                // Island 1 (left): 10×10 platform at height 35
                for x in 6..=15 {
                    for y in 35..=44 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                // Support column for island 1
                for x in 10..=11 {
                    for y in 0..35 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Island 2 (center): 8×8 platform at height 45
                for x in 28..=35 {
                    for y in 45..=52 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                // Support column for island 2
                for x in 31..=32 {
                    for y in 0..45 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Island 3 (right): 12×12 platform at height 25
                for x in 48..=59 {
                    for y in 25..=36 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                // Support column for island 3
                for x in 53..=54 {
                    for y in 0..25 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 12: Crumbling Wall
/// Tall wall with pre-placed gaps for mixed debris demonstration
pub fn generate_level_12_wall(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Wall with strategic gaps
            if cx == 0 && cy == 0 {
                // Build wall (x: 22-42, y: 0-50)
                for x in 22..=42 {
                    for y in 0..50 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Cut strategic gaps to create sections
                // Gap 1 (creates left section ~30px wide × 20px tall = 600px)
                for x in 22..=26 {
                    for y in 15..=30 {
                        chunk.set_material(x, y, MaterialId::AIR);
                    }
                }

                // Gap 2 (creates middle small section ~5px × 8px = 40px)
                for x in 32..=36 {
                    for y in 20..=27 {
                        chunk.set_material(x, y, MaterialId::AIR);
                    }
                }

                // Gap 3 (creates right section ~15px × 15px = 225px)
                for x in 38..=42 {
                    for y in 10..=24 {
                        chunk.set_material(x, y, MaterialId::AIR);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 13: Castle Siege
/// Castle structure with towers and walls
pub fn generate_level_13_castle(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Castle structure
            if cx == 0 && cy == 0 {
                // Foundation platform
                for x in 10..=54 {
                    for y in 0..=3 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Left tower (6×15)
                for x in 12..=17 {
                    for y in 4..=18 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Right tower (6×15)
                for x in 47..=52 {
                    for y in 4..=18 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Central keep (12×20)
                for x in 26..=37 {
                    for y in 4..=23 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Connecting walls (4 pixels thick)
                for x in 18..=26 {
                    for y in 4..=7 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                for x in 37..=47 {
                    for y in 4..=7 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 14: Domino Effect
/// Line of thin pillars for chain reaction physics
pub fn generate_level_14_domino(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Line of dominos
            if cx == 0 && cy == 0 {
                // Create 8 thin pillars (2 pixels wide, 25 tall, spaced 7 apart)
                for i in 0..8 {
                    let x_start = 6 + (i * 7);
                    for x in x_start..=(x_start + 1) {
                        for y in 0..25 {
                            chunk.set_material(x, y, MaterialId::STONE);
                        }
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 15: Quarry
/// Layered stone with support beams to simulate mining
pub fn generate_level_15_quarry(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Layered quarry
            if cx == 0 && cy == 0 {
                // Bottom layer (anchored to bedrock)
                for x in 10..=54 {
                    for y in 0..=3 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Support beams for layer 2
                for x in 20..=22 {
                    for y in 4..=13 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                for x in 42..=44 {
                    for y in 4..=13 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Layer 2
                for x in 12..=52 {
                    for y in 14..=17 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Support beams for layer 3
                for x in 25..=27 {
                    for y in 18..=27 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
                for x in 37..=39 {
                    for y in 18..=27 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Layer 3 (top)
                for x in 14..=50 {
                    for y in 28..=31 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}

/// Level 16: Stress Test
/// Massive 40×40 structure on single support - performance demonstration
pub fn generate_level_16_stress(world: &mut World) {
    world.clear_all_chunks();

    for cy in -2..=2 {
        for cx in -2..=2 {
            let mut chunk = Chunk::new(cx, cy);

            // BEDROCK LAYER
            if cy == -2 {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            } else if cy == -1 {
                for y in 0..8 {
                    for x in 0..CHUNK_SIZE {
                        chunk.set_material(x, y, MaterialId::BEDROCK);
                    }
                }
            }

            // CENTER CHUNK - Massive structure
            if cx == 0 && cy == 0 {
                // Critical support column (4 pixels wide, 15 tall)
                for x in 30..=33 {
                    for y in 0..15 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }

                // Massive platform above (40×40 pixels = 1600 pixels!)
                for x in 12..=51 {
                    for y in 15..=54 {
                        chunk.set_material(x, y, MaterialId::STONE);
                    }
                }
            }

            world.add_chunk(chunk);
        }
    }
}
