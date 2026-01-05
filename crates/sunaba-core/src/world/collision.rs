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
