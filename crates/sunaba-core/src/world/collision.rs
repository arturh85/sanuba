//! Collision detection utilities

use glam::IVec2;
use std::collections::HashMap;

use super::{Chunk, ChunkManager};
use crate::simulation::{MaterialType, Materials, Pixel};

/// Collision detector for entity-world collision checking
pub struct CollisionDetector;

impl CollisionDetector {
    /// Check if a rectangle collides with solid materials
    /// Returns true if collision detected
    pub fn check_solid_collision(
        chunks: &HashMap<IVec2, Chunk>,
        materials: &Materials,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        // Add collision tolerance - shrink hitbox slightly to prevent snagging on single pixels
        const TOLERANCE: f32 = 0.5; // Pixels of wiggle room
        let effective_width = width - TOLERANCE;
        let effective_height = height - TOLERANCE;

        // Check 8 points around hitbox
        let check_points = [
            (x - effective_width / 2.0, y - effective_height / 2.0), // Bottom-left
            (x + effective_width / 2.0, y - effective_height / 2.0), // Bottom-right
            (x - effective_width / 2.0, y + effective_height / 2.0), // Top-left
            (x + effective_width / 2.0, y + effective_height / 2.0), // Top-right
            (x, y - effective_height / 2.0),                         // Bottom-center
            (x, y + effective_height / 2.0),                         // Top-center
            (x - effective_width / 2.0, y),                          // Left-center
            (x + effective_width / 2.0, y),                          // Right-center
        ];

        for (px, py) in check_points {
            if let Some(pixel) = Self::get_pixel(chunks, px as i32, py as i32)
                && !pixel.is_empty()
            {
                let material = materials.get(pixel.material_id);
                // Collide only with solid materials
                if material.material_type == MaterialType::Solid {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a circle collides with solid materials
    /// Used for creature body part collision detection
    pub fn check_circle_collision(
        chunks: &HashMap<IVec2, Chunk>,
        materials: &Materials,
        x: f32,
        y: f32,
        radius: f32,
    ) -> bool {
        // Check center and 8 points around the perimeter
        let check_points = [
            (x, y),                                   // Center
            (x + radius, y),                          // Right
            (x - radius, y),                          // Left
            (x, y + radius),                          // Top
            (x, y - radius),                          // Bottom
            (x + radius * 0.707, y + radius * 0.707), // Top-right
            (x - radius * 0.707, y + radius * 0.707), // Top-left
            (x + radius * 0.707, y - radius * 0.707), // Bottom-right
            (x - radius * 0.707, y - radius * 0.707), // Bottom-left
        ];

        for (px, py) in check_points {
            if let Some(pixel) = Self::get_pixel(chunks, px as i32, py as i32)
                && !pixel.is_empty()
            {
                let material = materials.get(pixel.material_id);
                if material.material_type == MaterialType::Solid {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a rectangle is grounded (touching solid material below)
    pub fn is_rect_grounded(
        chunks: &HashMap<IVec2, Chunk>,
        materials: &Materials,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> bool {
        // Check 3 points just below the rectangle's bottom edge
        let check_y = y - (height / 2.0) - 1.5;
        let check_points = [
            (x - width / 4.0, check_y), // Left
            (x, check_y),               // Center
            (x + width / 4.0, check_y), // Right
        ];

        for (px, py) in check_points {
            if let Some(pixel) = Self::get_pixel(chunks, px as i32, py as i32)
                && !pixel.is_empty()
            {
                let material = materials.get(pixel.material_id);
                if material.material_type == MaterialType::Solid {
                    return true;
                }
            }
        }
        false
    }

    /// Check if any body part in a list is grounded (touching solid below)
    /// positions: slice of (center, radius) for each body part
    pub fn is_creature_grounded(
        chunks: &HashMap<IVec2, Chunk>,
        materials: &Materials,
        positions: &[(glam::Vec2, f32)],
    ) -> bool {
        for (center, radius) in positions {
            // Check 3 points just below the body part
            let check_y = center.y - radius - 1.0;
            let check_points = [
                (center.x - radius * 0.5, check_y),
                (center.x, check_y),
                (center.x + radius * 0.5, check_y),
            ];

            for (px, py) in check_points {
                if let Some(pixel) = Self::get_pixel(chunks, px as i32, py as i32)
                    && !pixel.is_empty()
                {
                    let material = materials.get(pixel.material_id);
                    if material.material_type == MaterialType::Solid {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Helper: Get pixel at world coordinates
    fn get_pixel(chunks: &HashMap<IVec2, Chunk>, world_x: i32, world_y: i32) -> Option<Pixel> {
        let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(world_x, world_y);
        chunks
            .get(&chunk_pos)
            .map(|c| c.get_pixel(local_x, local_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::MaterialId;

    /// Create a test world with chunks around origin
    fn setup_test_world() -> (HashMap<IVec2, Chunk>, Materials) {
        let mut chunks = HashMap::new();
        // Create chunks around origin
        for cy in -1..=1 {
            for cx in -1..=1 {
                chunks.insert(IVec2::new(cx, cy), Chunk::new(cx, cy));
            }
        }
        (chunks, Materials::new())
    }

    /// Set a pixel at world coordinates in the chunk map
    fn set_world_pixel(chunks: &mut HashMap<IVec2, Chunk>, world_x: i32, world_y: i32, material_id: u16) {
        let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(world_x, world_y);
        if let Some(chunk) = chunks.get_mut(&chunk_pos) {
            chunk.set_material(local_x, local_y, material_id);
        }
    }

    #[test]
    fn test_check_solid_collision_no_collision_in_air() {
        let (chunks, materials) = setup_test_world();

        // Rectangle in empty space (air) should not collide
        let collision = CollisionDetector::check_solid_collision(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(!collision, "Should not collide with air");
    }

    #[test]
    fn test_check_solid_collision_with_solid() {
        let (mut chunks, materials) = setup_test_world();

        // Collision checks 8 points around the hitbox edge, not the center
        // For a 10x10 box centered at (32, 32) with 0.5 tolerance:
        // - effective size is 9.5x9.5
        // - bottom-center check point is at (32, 27.25) -> (32, 27)
        set_world_pixel(&mut chunks, 32, 27, MaterialId::STONE);

        let collision = CollisionDetector::check_solid_collision(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(collision, "Should collide with stone at bottom-center check point");
    }

    #[test]
    fn test_check_solid_collision_ignores_liquids() {
        let (mut chunks, materials) = setup_test_world();

        // Place water at (32, 32) - should not cause collision
        set_world_pixel(&mut chunks, 32, 32, MaterialId::WATER);

        let collision = CollisionDetector::check_solid_collision(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(!collision, "Should not collide with liquids");
    }

    #[test]
    fn test_check_solid_collision_ignores_powder() {
        let (mut chunks, materials) = setup_test_world();

        // Place sand at (32, 32) - powder should not cause collision (it can be pushed aside)
        set_world_pixel(&mut chunks, 32, 32, MaterialId::SAND);

        let collision = CollisionDetector::check_solid_collision(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(!collision, "Should not collide with powder");
    }

    #[test]
    fn test_check_circle_collision_no_collision() {
        let (chunks, materials) = setup_test_world();

        // Circle in empty space
        let collision = CollisionDetector::check_circle_collision(
            &chunks, &materials, 32.0, 32.0, 5.0
        );
        assert!(!collision, "Should not collide in air");
    }

    #[test]
    fn test_check_circle_collision_with_solid() {
        let (mut chunks, materials) = setup_test_world();

        // Place stone at circle center
        set_world_pixel(&mut chunks, 32, 32, MaterialId::STONE);

        let collision = CollisionDetector::check_circle_collision(
            &chunks, &materials, 32.0, 32.0, 5.0
        );
        assert!(collision, "Should collide with stone at center");
    }

    #[test]
    fn test_check_circle_collision_at_perimeter() {
        let (mut chunks, materials) = setup_test_world();

        // Place stone at right edge of circle (radius 5)
        set_world_pixel(&mut chunks, 37, 32, MaterialId::STONE);

        let collision = CollisionDetector::check_circle_collision(
            &chunks, &materials, 32.0, 32.0, 5.0
        );
        assert!(collision, "Should collide with stone at perimeter");
    }

    #[test]
    fn test_is_rect_grounded_on_solid() {
        let (mut chunks, materials) = setup_test_world();

        // Place a solid floor below the rectangle
        // Rectangle at y=32 with height=10, bottom edge at y=27
        // Ground check is at y=26.5 (bottom - 1.5)
        for x in 24..=40 {
            set_world_pixel(&mut chunks, x, 25, MaterialId::STONE);
        }

        let grounded = CollisionDetector::is_rect_grounded(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(grounded, "Should be grounded on solid floor");
    }

    #[test]
    fn test_is_rect_grounded_in_air() {
        let (chunks, materials) = setup_test_world();

        // No floor - rectangle floating in air
        let grounded = CollisionDetector::is_rect_grounded(
            &chunks, &materials, 32.0, 32.0, 10.0, 10.0
        );
        assert!(!grounded, "Should not be grounded in air");
    }

    #[test]
    fn test_is_creature_grounded() {
        let (mut chunks, materials) = setup_test_world();

        // Place floor
        for x in 20..=44 {
            set_world_pixel(&mut chunks, x, 25, MaterialId::STONE);
        }

        // Creature with body parts at different positions
        let positions = vec![
            (glam::Vec2::new(32.0, 30.0), 3.0), // Body at center
            (glam::Vec2::new(28.0, 28.0), 2.0), // Left foot
            (glam::Vec2::new(36.0, 28.0), 2.0), // Right foot
        ];

        let grounded = CollisionDetector::is_creature_grounded(&chunks, &materials, &positions);
        assert!(grounded, "Creature with feet on ground should be grounded");
    }

    #[test]
    fn test_is_creature_grounded_floating() {
        let (chunks, materials) = setup_test_world();

        // No floor - creature floating
        let positions = vec![
            (glam::Vec2::new(32.0, 50.0), 3.0),
        ];

        let grounded = CollisionDetector::is_creature_grounded(&chunks, &materials, &positions);
        assert!(!grounded, "Floating creature should not be grounded");
    }

    #[test]
    fn test_get_pixel_missing_chunk() {
        let chunks = HashMap::new(); // Empty - no chunks loaded

        // Should return None for unloaded chunks
        let pixel = CollisionDetector::get_pixel(&chunks, 1000, 1000);
        assert!(pixel.is_none(), "Should return None for missing chunk");
    }
}
