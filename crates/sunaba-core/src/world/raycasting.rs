//! Raycasting utilities for line-of-sight queries

use super::chunk_manager::ChunkManager;
use crate::simulation::{MaterialId, MaterialType, Materials};
use bresenham::Bresenham;
use glam::Vec2;

/// Raycasting utilities - stateless methods for line-of-sight queries
pub struct Raycasting;

impl Raycasting {
    /// Simple raycast from position in direction, stopping at first non-air material
    /// Returns (world_x, world_y, material_id) of hit pixel, or None if clear
    ///
    /// Uses Bresenham line algorithm for exact pixel traversal
    pub fn raycast(
        chunk_manager: &ChunkManager,
        from: Vec2,
        direction: Vec2,
        max_distance: f32,
    ) -> Option<(i32, i32, u16)> {
        // Calculate start and end points for Bresenham (uses isize)
        let from_i = (from.x.round() as isize, from.y.round() as isize);
        let to = from + direction.normalize_or_zero() * max_distance;
        let to_i = (to.x.round() as isize, to.y.round() as isize);

        // Use Bresenham line algorithm for exact pixel traversal
        for (x, y) in Bresenham::new(from_i, to_i) {
            let (chunk_pos, local_x, local_y) =
                ChunkManager::world_to_chunk_coords(x as i32, y as i32);
            if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
                let pixel = chunk.get_pixel(local_x, local_y);
                if pixel.material_id != MaterialId::AIR {
                    return Some((x as i32, y as i32, pixel.material_id));
                }
            }
        }
        None
    }

    /// Raycast with material type filter (e.g., only solids)
    /// Returns hit position and material ID if matching type found
    ///
    /// Starts raycast from `radius` distance (useful for sensor raycasts from body surface)
    /// Stops at first pixel matching the material_type_filter
    /// Uses Bresenham line algorithm for exact pixel traversal
    pub fn raycast_filtered(
        chunk_manager: &ChunkManager,
        materials: &Materials,
        from: Vec2,
        direction: Vec2,
        radius: f32,
        max_distance: f32,
        material_type_filter: MaterialType,
    ) -> Option<(i32, i32, u16)> {
        // Normalize direction
        let dir = direction.normalize_or_zero();
        if dir == Vec2::ZERO {
            return None;
        }

        // Calculate start and end points for Bresenham (uses isize)
        let start = from + dir * radius; // Start from edge of body
        let end = from + dir * max_distance;
        let start_i = (start.x.round() as isize, start.y.round() as isize);
        let end_i = (end.x.round() as isize, end.y.round() as isize);

        // Use Bresenham line algorithm for exact pixel traversal
        for (x, y) in Bresenham::new(start_i, end_i) {
            let (chunk_pos, local_x, local_y) =
                ChunkManager::world_to_chunk_coords(x as i32, y as i32);
            if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
                let pixel = chunk.get_pixel(local_x, local_y);
                if !pixel.is_empty() {
                    let material = materials.get(pixel.material_id);
                    if material.material_type == material_type_filter {
                        return Some((x as i32, y as i32, pixel.material_id));
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{Chunk, ChunkManager};
    use glam::IVec2;

    /// Create a test chunk manager with chunks around origin
    fn setup_test_chunk_manager() -> ChunkManager {
        let mut manager = ChunkManager::new();
        // Create chunks around origin
        for cy in -1..=1 {
            for cx in -1..=1 {
                manager
                    .chunks
                    .insert(IVec2::new(cx, cy), Chunk::new(cx, cy));
            }
        }
        manager
    }

    /// Set a pixel at world coordinates
    fn set_world_pixel(manager: &mut ChunkManager, world_x: i32, world_y: i32, material_id: u16) {
        let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(world_x, world_y);
        if let Some(chunk) = manager.chunks.get_mut(&chunk_pos) {
            chunk.set_material(local_x, local_y, material_id);
        }
    }

    #[test]
    fn test_raycast_hits_solid() {
        let mut manager = setup_test_chunk_manager();

        // Place a stone wall at x=40
        for y in 0..64 {
            set_world_pixel(&mut manager, 40, y, MaterialId::STONE);
        }

        // Cast ray from (10, 32) going right, should hit wall at x=40
        let hit = Raycasting::raycast(&manager, Vec2::new(10.0, 32.0), Vec2::new(1.0, 0.0), 100.0);

        assert!(hit.is_some(), "Raycast should hit stone wall");
        let (x, y, material_id) = hit.unwrap();
        assert_eq!(x, 40, "Hit x should be at wall position");
        assert_eq!(y, 32, "Hit y should be at ray y position");
        assert_eq!(material_id, MaterialId::STONE, "Should hit stone");
    }

    #[test]
    fn test_raycast_misses_in_air() {
        let manager = setup_test_chunk_manager();

        // Cast ray through empty air
        let hit = Raycasting::raycast(&manager, Vec2::new(10.0, 32.0), Vec2::new(1.0, 0.0), 20.0);

        assert!(hit.is_none(), "Raycast through air should not hit anything");
    }

    #[test]
    fn test_raycast_diagonal() {
        let mut manager = setup_test_chunk_manager();

        // Place stone at (30, 30)
        set_world_pixel(&mut manager, 30, 30, MaterialId::STONE);

        // Cast diagonal ray from (10, 10) toward (30, 30)
        let direction = Vec2::new(1.0, 1.0).normalize();
        let hit = Raycasting::raycast(&manager, Vec2::new(10.0, 10.0), direction, 50.0);

        assert!(hit.is_some(), "Diagonal ray should hit stone");
        let (x, y, _) = hit.unwrap();
        assert_eq!(x, 30, "Hit x should be 30");
        assert_eq!(y, 30, "Hit y should be 30");
    }

    #[test]
    fn test_raycast_max_distance() {
        let mut manager = setup_test_chunk_manager();

        // Place stone beyond max distance
        set_world_pixel(&mut manager, 50, 32, MaterialId::STONE);

        // Cast ray with limited max distance (should not reach stone)
        let hit = Raycasting::raycast(
            &manager,
            Vec2::new(10.0, 32.0),
            Vec2::new(1.0, 0.0),
            30.0, // Max distance 30, stone is at 50
        );

        assert!(hit.is_none(), "Ray should not reach beyond max_distance");
    }

    #[test]
    fn test_raycast_filtered_hits_solid_only() {
        let mut manager = setup_test_chunk_manager();
        let materials = Materials::new();

        // Place water (liquid) first, then stone (solid) behind it
        set_world_pixel(&mut manager, 30, 32, MaterialId::WATER);
        set_world_pixel(&mut manager, 40, 32, MaterialId::STONE);

        // Filter for only solid materials
        let hit = Raycasting::raycast_filtered(
            &manager,
            &materials,
            Vec2::new(10.0, 32.0),
            Vec2::new(1.0, 0.0),
            0.0, // start at origin
            100.0,
            MaterialType::Solid,
        );

        assert!(hit.is_some(), "Should find solid material");
        let (x, _, material_id) = hit.unwrap();
        assert_eq!(x, 40, "Should hit stone, not water");
        assert_eq!(material_id, MaterialId::STONE);
    }

    #[test]
    fn test_raycast_filtered_with_radius_offset() {
        let mut manager = setup_test_chunk_manager();
        let materials = Materials::new();

        // Place stone right next to origin (at radius=5 from 10,32)
        set_world_pixel(&mut manager, 15, 32, MaterialId::STONE);
        // Place another stone further out
        set_world_pixel(&mut manager, 30, 32, MaterialId::STONE);

        // Cast with radius offset - should skip the first stone and hit the second
        let hit = Raycasting::raycast_filtered(
            &manager,
            &materials,
            Vec2::new(10.0, 32.0),
            Vec2::new(1.0, 0.0),
            10.0, // Start 10 pixels out (skips stone at 15)
            100.0,
            MaterialType::Solid,
        );

        assert!(hit.is_some(), "Should find solid beyond radius");
        let (x, _, _) = hit.unwrap();
        assert_eq!(x, 30, "Should hit second stone, not first");
    }

    #[test]
    fn test_raycast_filtered_no_match() {
        let mut manager = setup_test_chunk_manager();
        let materials = Materials::new();

        // Place only water (liquid)
        set_world_pixel(&mut manager, 30, 32, MaterialId::WATER);

        // Filter for solid - should not match water
        let hit = Raycasting::raycast_filtered(
            &manager,
            &materials,
            Vec2::new(10.0, 32.0),
            Vec2::new(1.0, 0.0),
            0.0,
            100.0,
            MaterialType::Solid,
        );

        assert!(
            hit.is_none(),
            "Should not match liquid when filtering for solid"
        );
    }

    #[test]
    fn test_raycast_filtered_zero_direction() {
        let manager = setup_test_chunk_manager();
        let materials = Materials::new();

        // Zero direction should return None immediately
        let hit = Raycasting::raycast_filtered(
            &manager,
            &materials,
            Vec2::new(10.0, 32.0),
            Vec2::ZERO, // Zero direction
            0.0,
            100.0,
            MaterialType::Solid,
        );

        assert!(hit.is_none(), "Zero direction should return None");
    }
}
