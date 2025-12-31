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
