//! Neighbor pixel collection utilities

use super::chunk_manager::ChunkManager;
use smallvec::SmallVec;

/// Neighbor collection utilities - stateless methods for querying neighboring pixels
pub struct NeighborQueries;

impl NeighborQueries {
    /// Collect all 8 neighboring materials (cardinal + diagonal)
    /// Returns SmallVec of neighbor material IDs (may be empty if neighbors are air or out of bounds)
    ///
    /// Order: NW, N, NE, W, E, SW, S, SE
    pub fn get_8_neighbors(
        chunk_manager: &ChunkManager,
        center_x: i32,
        center_y: i32,
    ) -> SmallVec<[u16; 8]> {
        let mut neighbors = SmallVec::new();

        for (dx, dy) in [
            (-1, -1), // NW
            (0, -1),  // N
            (1, -1),  // NE
            (-1, 0),  // W
            (1, 0),   // E
            (-1, 1),  // SW
            (0, 1),   // S
            (1, 1),   // SE
        ] {
            let x = center_x + dx;
            let y = center_y + dy;

            let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(x, y);
            if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
                let pixel = chunk.get_pixel(local_x, local_y);
                neighbors.push(pixel.material_id);
            }
        }

        neighbors
    }

    /// Iterate over 4 orthogonal neighbors (N, E, S, W)
    /// Calls callback for each neighbor pixel that exists
    ///
    /// Order: S, E, N, W (matches common iteration pattern in reactions)
    pub fn for_each_orthogonal_neighbor<F>(
        chunk_manager: &ChunkManager,
        center_x: i32,
        center_y: i32,
        mut callback: F,
    ) where
        F: FnMut(i32, i32, u16),
    {
        for (dx, dy) in [(0, 1), (1, 0), (0, -1), (-1, 0)] {
            let x = center_x + dx;
            let y = center_y + dy;

            let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(x, y);
            if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
                let pixel = chunk.get_pixel(local_x, local_y);
                callback(x, y, pixel.material_id);
            }
        }
    }

    /// Get pixels in circular radius around center
    /// Returns SmallVec of (x, y, material_id) for all pixels within radius
    ///
    /// Useful for area effects, spreading, erosion, etc.
    /// Stack-allocated up to 64 neighbors (typical radii: 5-10 pixels = 20-60 neighbors)
    pub fn get_pixels_in_radius(
        chunk_manager: &ChunkManager,
        center_x: i32,
        center_y: i32,
        radius: i32,
    ) -> SmallVec<[(i32, i32, u16); 64]> {
        let mut pixels = SmallVec::new();

        // Iterate over square containing circle
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Check if point is inside circle (Euclidean distance)
                if dx * dx + dy * dy <= radius * radius {
                    let x = center_x + dx;
                    let y = center_y + dy;

                    let (chunk_pos, local_x, local_y) = ChunkManager::world_to_chunk_coords(x, y);
                    if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
                        let pixel = chunk.get_pixel(local_x, local_y);
                        pixels.push((x, y, pixel.material_id));
                    }
                }
            }
        }

        pixels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::MaterialId;
    use crate::world::Chunk;
    use glam::IVec2;

    /// Create a test chunk manager with a single chunk at origin
    fn setup_test_chunk_manager() -> ChunkManager {
        let mut manager = ChunkManager::new();
        manager.chunks.insert(IVec2::new(0, 0), Chunk::new(0, 0));
        manager
    }

    /// Create chunk manager with chunks at origin and its neighbors
    fn setup_multi_chunk_manager() -> ChunkManager {
        let mut manager = ChunkManager::new();
        for cy in -1..=1 {
            for cx in -1..=1 {
                manager.chunks.insert(IVec2::new(cx, cy), Chunk::new(cx, cy));
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
    fn test_get_8_neighbors_all_air() {
        let manager = setup_test_chunk_manager();

        // Center pixel in chunk (32, 32), all neighbors are air
        let neighbors = NeighborQueries::get_8_neighbors(&manager, 32, 32);

        // Should have 8 neighbors, all AIR
        assert_eq!(neighbors.len(), 8, "Should return 8 neighbors");
        for &material_id in &neighbors {
            assert_eq!(material_id, MaterialId::AIR, "All neighbors should be air");
        }
    }

    #[test]
    fn test_get_8_neighbors_with_materials() {
        let mut manager = setup_test_chunk_manager();

        // Place different materials around (32, 32)
        set_world_pixel(&mut manager, 31, 31, MaterialId::STONE); // NW
        set_world_pixel(&mut manager, 32, 31, MaterialId::SAND);  // N
        set_world_pixel(&mut manager, 33, 31, MaterialId::WATER); // NE

        let neighbors = NeighborQueries::get_8_neighbors(&manager, 32, 32);

        assert_eq!(neighbors.len(), 8);
        // Order: NW, N, NE, W, E, SW, S, SE
        assert_eq!(neighbors[0], MaterialId::STONE, "NW should be stone");
        assert_eq!(neighbors[1], MaterialId::SAND, "N should be sand");
        assert_eq!(neighbors[2], MaterialId::WATER, "NE should be water");
    }

    #[test]
    fn test_get_8_neighbors_at_chunk_edge() {
        let manager = setup_test_chunk_manager();

        // Query at chunk boundary (0, 0) - some neighbors will be in non-existent chunks
        let neighbors = NeighborQueries::get_8_neighbors(&manager, 0, 0);

        // Only 3 neighbors in the existing chunk (E, S, SE)
        assert_eq!(neighbors.len(), 3, "Only neighbors in loaded chunks are returned");
    }

    #[test]
    fn test_get_8_neighbors_cross_chunk() {
        let manager = setup_multi_chunk_manager();

        // Query at chunk boundary (0, 0) - with multiple chunks loaded
        let neighbors = NeighborQueries::get_8_neighbors(&manager, 0, 0);

        // All 8 neighbors should be available now
        assert_eq!(neighbors.len(), 8, "All neighbors accessible with chunks loaded");
    }

    #[test]
    fn test_for_each_orthogonal_neighbor() {
        let mut manager = setup_test_chunk_manager();

        // Place materials in 4 cardinal directions
        set_world_pixel(&mut manager, 32, 33, MaterialId::STONE); // S
        set_world_pixel(&mut manager, 33, 32, MaterialId::SAND);  // E
        set_world_pixel(&mut manager, 32, 31, MaterialId::WATER); // N
        set_world_pixel(&mut manager, 31, 32, MaterialId::LAVA);  // W

        let mut collected = Vec::new();
        NeighborQueries::for_each_orthogonal_neighbor(&manager, 32, 32, |x, y, mat| {
            collected.push((x, y, mat));
        });

        assert_eq!(collected.len(), 4, "Should visit 4 orthogonal neighbors");

        // Verify the materials were collected (order: S, E, N, W)
        let materials: Vec<u16> = collected.iter().map(|(_, _, m)| *m).collect();
        assert!(materials.contains(&MaterialId::STONE), "Should include stone (S)");
        assert!(materials.contains(&MaterialId::SAND), "Should include sand (E)");
        assert!(materials.contains(&MaterialId::WATER), "Should include water (N)");
        assert!(materials.contains(&MaterialId::LAVA), "Should include lava (W)");
    }

    #[test]
    fn test_for_each_orthogonal_at_edge() {
        let manager = setup_test_chunk_manager();

        // Query at corner - only 2 neighbors available
        let mut count = 0;
        NeighborQueries::for_each_orthogonal_neighbor(&manager, 0, 0, |_, _, _| {
            count += 1;
        });

        assert_eq!(count, 2, "Only 2 orthogonal neighbors in single chunk at corner");
    }

    #[test]
    fn test_get_pixels_in_radius_zero() {
        let manager = setup_test_chunk_manager();

        // Radius 0 should return just the center pixel
        let pixels = NeighborQueries::get_pixels_in_radius(&manager, 32, 32, 0);

        assert_eq!(pixels.len(), 1, "Radius 0 should return just center");
        assert_eq!(pixels[0], (32, 32, MaterialId::AIR));
    }

    #[test]
    fn test_get_pixels_in_radius_small() {
        let manager = setup_test_chunk_manager();

        // Radius 1 should return center + 4 orthogonal neighbors (5 total in a small circle)
        let pixels = NeighborQueries::get_pixels_in_radius(&manager, 32, 32, 1);

        // Radius 1: pixels where dx*dx + dy*dy <= 1 = center + 4 orthogonal = 5
        assert_eq!(pixels.len(), 5, "Radius 1 should return 5 pixels");

        // Verify center is included
        let has_center = pixels.iter().any(|(x, y, _)| *x == 32 && *y == 32);
        assert!(has_center, "Center pixel should be included");
    }

    #[test]
    fn test_get_pixels_in_radius_finds_materials() {
        let mut manager = setup_test_chunk_manager();

        // Place stone at (34, 32) - within radius 3 of (32, 32)
        set_world_pixel(&mut manager, 34, 32, MaterialId::STONE);

        let pixels = NeighborQueries::get_pixels_in_radius(&manager, 32, 32, 3);

        // Find the stone in the results
        let stone = pixels.iter().find(|(x, y, _)| *x == 34 && *y == 32);
        assert!(stone.is_some(), "Should find stone within radius");
        assert_eq!(stone.unwrap().2, MaterialId::STONE);
    }

    #[test]
    fn test_get_pixels_in_radius_circular_shape() {
        let manager = setup_test_chunk_manager();

        // Radius 2: check that corners are excluded (dx*dx + dy*dy > 4)
        let pixels = NeighborQueries::get_pixels_in_radius(&manager, 32, 32, 2);

        // Corner (34, 34) is at distance sqrt(8) > 2, should not be included
        let has_corner = pixels.iter().any(|(x, y, _)| *x == 34 && *y == 34);
        assert!(!has_corner, "Corner at distance sqrt(8) should not be in radius 2");

        // But (34, 32) is at distance 2, should be included
        let has_edge = pixels.iter().any(|(x, y, _)| *x == 34 && *y == 32);
        assert!(has_edge, "Edge at distance 2 should be included");
    }
}
