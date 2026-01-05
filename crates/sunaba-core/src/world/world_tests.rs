//! Tests for World struct and its core functionality
//!
//! Kept in a separate file to maintain readability of world.rs

use super::*;
use crate::simulation::MaterialId;

/// Create a minimal world for testing (skip creature spawning, no persistence)
fn create_test_world() -> World {
    let mut world = World::new(true); // Skip initial creatures
    world.disable_persistence();
    // Ensure we have some chunks around origin
    world.ensure_chunks_for_area(-64, -64, 128, 128);
    world
}

#[test]
fn test_world_new_creates_valid_world() {
    let world = World::new(true);

    // Should have materials registry (names are lowercase)
    assert!(world.materials.get(MaterialId::AIR).name == "air");
    assert!(world.materials.get(MaterialId::STONE).name == "stone");

    // Should have tool and recipe registries
    assert!(world.tool_registry().get(1001).is_some()); // Stone pickaxe
    assert!(!world.recipe_registry.all_recipes().is_empty());

    // Player should start at spawn point
    assert_eq!(world.player.position, glam::Vec2::new(0.0, 100.0));
}

#[test]
fn test_world_default() {
    let world = World::default();
    // Default should create world with creatures (true creatures would spawn in evolution mode)
    assert_eq!(world.player.position, glam::Vec2::new(0.0, 100.0));
}

#[test]
fn test_set_generator() {
    let mut world = World::new(true);
    world.disable_persistence();

    // Set different seed
    world.set_generator(12345);

    // Generate a chunk and verify it works
    world.generate_chunk(IVec2::new(0, 0));
    assert!(world.has_chunk(IVec2::new(0, 0)));
}

#[test]
fn test_get_set_pixel() {
    let mut world = create_test_world();

    // Set a pixel
    world.set_pixel(10, 10, MaterialId::STONE);

    // Get the pixel back
    let pixel = world.get_pixel(10, 10);
    assert!(pixel.is_some());
    assert_eq!(pixel.unwrap().material_id, MaterialId::STONE);
}

#[test]
fn test_get_pixel_missing_chunk() {
    let world = World::new(true);
    // No chunks loaded, pixel should be None
    let pixel = world.get_pixel(10000, 10000);
    assert!(pixel.is_none());
}

#[test]
fn test_set_pixel_full() {
    let mut world = create_test_world();

    // Create pixel with flags
    let mut pixel = Pixel::new(MaterialId::STONE);
    pixel.flags |= pixel_flags::PLAYER_PLACED;

    world.set_pixel_full(20, 20, pixel);

    // Verify
    let retrieved = world.get_pixel(20, 20).unwrap();
    assert_eq!(retrieved.material_id, MaterialId::STONE);
    assert!(retrieved.flags & pixel_flags::PLAYER_PLACED != 0);
}

#[test]
fn test_get_pixel_material() {
    let mut world = create_test_world();

    world.set_pixel(30, 30, MaterialId::WATER);

    let material_id = world.get_pixel_material(30, 30);
    assert_eq!(material_id, Some(MaterialId::WATER));
}

#[test]
fn test_spawn_material_circular_brush() {
    let mut world = create_test_world();

    // Spawn sand with brush size 2
    world.spawn_material(50, 50, MaterialId::SAND, 2);

    // Center should be sand
    assert_eq!(
        world.get_pixel(50, 50).unwrap().material_id,
        MaterialId::SAND
    );

    // Adjacent pixels (within radius) should also be sand
    assert_eq!(
        world.get_pixel(51, 50).unwrap().material_id,
        MaterialId::SAND
    );
    assert_eq!(
        world.get_pixel(50, 51).unwrap().material_id,
        MaterialId::SAND
    );

    // Pixels at radius 2 but not exceeding should be sand
    assert_eq!(
        world.get_pixel(52, 50).unwrap().material_id,
        MaterialId::SAND
    );

    // Corners at distance sqrt(8) > 2 should not be set
    assert_eq!(
        world.get_pixel(52, 52).unwrap().material_id,
        MaterialId::AIR
    );
}

#[test]
fn test_ensure_chunks_for_area() {
    let mut world = World::new(true);
    world.disable_persistence();

    // Initially may have some chunks, but specific area may not exist
    let has_before = world.has_chunk(IVec2::new(5, 5));

    // Ensure chunks for area that includes chunk (5, 5)
    world.ensure_chunks_for_area(300, 300, 400, 400);

    // Now should have chunk
    assert!(world.has_chunk(IVec2::new(5, 5)) || has_before);
}

#[test]
fn test_has_chunk_insert_chunk() {
    let mut world = World::new(true);
    world.disable_persistence();

    let pos = IVec2::new(100, 100);
    assert!(!world.has_chunk(pos));

    // Insert a chunk
    let chunk = Chunk::new(100, 100);
    world.insert_chunk(pos, chunk);

    assert!(world.has_chunk(pos));
}

#[test]
fn test_get_chunk() {
    let world = create_test_world();

    // Should have chunk at (0, 0)
    let chunk = world.get_chunk(0, 0);
    assert!(chunk.is_some());

    // Should not have chunk at far location
    let chunk = world.get_chunk(1000, 1000);
    assert!(chunk.is_none());
}

#[test]
fn test_chunks_accessor() {
    let world = create_test_world();

    // Should have multiple chunks
    assert!(!world.chunks().is_empty());

    // Should be able to iterate
    let count = world.chunks_iter().count();
    assert!(count > 0);
}

#[test]
fn test_chunks_mut_accessor() {
    let mut world = create_test_world();

    // Should be able to mutate chunks
    if let Some(chunk) = world.chunks_mut().get_mut(&IVec2::new(0, 0)) {
        chunk.set_material(5, 5, MaterialId::LAVA);
    }

    // Verify the change
    let pixel = world.get_pixel(5, 5);
    assert!(pixel.is_some());
    assert_eq!(pixel.unwrap().material_id, MaterialId::LAVA);
}

#[test]
fn test_active_chunk_positions() {
    let world = create_test_world();

    // Active chunks are managed by update_active_chunks
    // Just verify the accessor works and returns a slice
    let _positions = world.active_chunk_positions();
    // May be empty initially without simulation running - that's ok
}

#[test]
fn test_is_player_dead_and_respawn() {
    let mut world = create_test_world();

    // Initially player should not be dead
    assert!(!world.is_player_dead());

    // Kill the player
    world.player.is_dead = true;
    assert!(world.is_player_dead());

    // Respawn
    world.respawn_player();
    assert!(!world.is_player_dead());
    assert_eq!(world.player.position, glam::Vec2::new(0.0, 100.0));
}

#[test]
fn test_tool_registry_accessor() {
    let world = create_test_world();

    let registry = world.tool_registry();
    // Tool IDs: 1001 = stone pickaxe, 1002 = iron pickaxe, etc.
    assert!(registry.get(1001).is_some()); // Stone pickaxe
}

#[test]
fn test_materials_accessor() {
    let world = create_test_world();

    let materials = world.materials();
    // Material names are lowercase
    assert_eq!(materials.get(MaterialId::AIR).name, "air");
    assert_eq!(materials.get(MaterialId::STONE).name, "stone");
    assert_eq!(materials.get(MaterialId::WATER).name, "water");
}

#[test]
fn test_falling_chunk_count_initially_zero() {
    let world = create_test_world();

    // Initially no falling chunks
    assert_eq!(world.falling_chunk_count(), 0);
}

#[test]
fn test_get_falling_chunks_empty() {
    let world = create_test_world();

    let falling = world.get_falling_chunks();
    assert!(falling.is_empty());
}

#[test]
fn test_generate_chunk() {
    let mut world = World::new(true);
    world.disable_persistence();

    let pos = IVec2::new(10, 10);
    assert!(!world.has_chunk(pos));

    world.generate_chunk(pos);

    assert!(world.has_chunk(pos));
}

#[test]
fn test_clear_all_chunks() {
    let mut world = create_test_world();

    // Should have chunks
    assert!(!world.chunks().is_empty());

    world.clear_all_chunks();

    // All chunks should be cleared
    assert!(world.chunks().is_empty());
}

#[test]
fn test_add_chunk() {
    let mut world = World::new(true);
    world.disable_persistence();

    let chunk = Chunk::new(50, 50);
    world.add_chunk(chunk);

    assert!(world.has_chunk(IVec2::new(50, 50)));
}

#[test]
fn test_check_circle_collision_solid() {
    let mut world = create_test_world();

    // Place stone at center
    world.set_pixel(100, 100, MaterialId::STONE);

    // Circle at same position should collide
    assert!(world.check_circle_collision(100.0, 100.0, 5.0));
}

#[test]
fn test_check_circle_collision_air() {
    let world = create_test_world();

    // Circle in empty space should not collide
    assert!(!world.check_circle_collision(100.0, 100.0, 5.0));
}

#[test]
fn test_is_creature_grounded() {
    let mut world = create_test_world();

    // Place floor - grounding check looks at (center.y - radius - 1.5)
    // For body at y=100 with radius=3, check is at y=95.5 -> floor at y=95
    // Actually check samples at integer positions, so floor at y=95 works
    // But we need to ensure the floor line is at the right level
    for x in 90..=110 {
        world.set_pixel(x, 96, MaterialId::STONE);
    }

    // Body part that should be grounded - center at y=100, radius=3
    // Bottom at y=97, ground check at y=95.5 rounds to 95 or 96
    let positions = vec![(glam::Vec2::new(100.0, 100.0), 3.0)];

    assert!(world.is_creature_grounded(&positions));
}

#[test]
fn test_is_creature_grounded_floating() {
    let world = create_test_world();

    // No floor - body parts floating
    let positions = vec![(glam::Vec2::new(100.0, 100.0), 3.0)];

    assert!(!world.is_creature_grounded(&positions));
}

#[test]
fn test_get_blocking_pixel() {
    let mut world = create_test_world();

    // Place a wall
    for y in 0..20 {
        world.set_pixel(120, y + 90, MaterialId::STONE);
    }

    // Cast ray toward wall
    let hit = world.get_blocking_pixel(
        glam::Vec2::new(100.0, 100.0),
        glam::Vec2::new(1.0, 0.0),
        0.0,
        50.0,
    );

    assert!(hit.is_some());
    let (x, _, material_id) = hit.unwrap();
    assert_eq!(x, 120);
    assert_eq!(material_id, MaterialId::STONE);
}

#[test]
fn test_get_blocking_pixel_no_obstacle() {
    let world = create_test_world();

    // Cast ray through empty space
    let hit = world.get_blocking_pixel(
        glam::Vec2::new(100.0, 100.0),
        glam::Vec2::new(1.0, 0.0),
        0.0,
        20.0,
    );

    assert!(hit.is_none());
}

#[test]
fn test_world_collision_query_is_solid_at() {
    let mut world = create_test_world();

    // Place stone
    world.set_pixel(50, 50, MaterialId::STONE);

    // Test WorldCollisionQuery trait
    use crate::simulation::WorldCollisionQuery;
    assert!(world.is_solid_at(50, 50));
    assert!(!world.is_solid_at(51, 51)); // Air is not solid
}

#[test]
fn test_world_collision_query_out_of_bounds() {
    let world = create_test_world();

    // Out of bounds should return true (solid) to prevent falling forever
    use crate::simulation::WorldCollisionQuery;
    assert!(world.is_solid_at(100000, 100000));
}

#[test]
fn test_get_temperature_at_pixel() {
    let world = create_test_world();

    // Temperature at any valid location should return room temperature (~20)
    let temp = world.get_temperature_at_pixel(50, 50);
    assert!(
        (temp - 20.0).abs() < 1.0,
        "Expected room temperature, got {}",
        temp
    );
}

#[test]
fn test_get_growth_progress_percent() {
    let world = create_test_world();

    // Growth progress should be 0-100
    let progress = world.get_growth_progress_percent();
    assert!((0.0..=100.0).contains(&progress));
}

#[test]
fn test_disable_persistence() {
    let mut world = World::new(true);

    // Disable persistence
    world.disable_persistence();

    // Chunk manager should be in ephemeral mode
    assert!(world.chunk_manager.ephemeral_chunks);
}

#[test]
fn test_evict_distant_chunks() {
    let mut world = create_test_world();

    // Add a distant chunk
    let distant_pos = IVec2::new(100, 100); // Very far from player at (0, 100)
    world.insert_chunk(distant_pos, Chunk::new(100, 100));

    assert!(world.has_chunk(distant_pos));

    // Evict distant chunks (player is at 0, 100)
    world.evict_distant_chunks(Vec2::new(0.0, 100.0));

    // Distant chunk should be evicted
    assert!(!world.has_chunk(distant_pos));
}

#[test]
fn test_update_chunk_settle() {
    let mut world = create_test_world();

    // Generate a chunk
    world.generate_chunk(IVec2::new(0, -1));

    // Settle it (simulate CA)
    use rand::SeedableRng;
    let mut rng = rand_xoshiro::Xoshiro256StarStar::seed_from_u64(42);
    world.update_chunk_settle(0, -1, &mut rng);

    // Should still have the chunk
    assert!(world.has_chunk(IVec2::new(0, -1)));
}
