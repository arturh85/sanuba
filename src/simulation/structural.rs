//! Structural integrity checking and falling debris conversion

use crate::simulation::{MaterialId, MaterialType};
use crate::world::World;
use glam::IVec2;
use std::collections::{HashSet, VecDeque};

/// Maximum distance to check for structural support
const MAX_FLOOD_FILL_RADIUS: i32 = 64;

/// Threshold for small vs large debris
const SMALL_DEBRIS_THRESHOLD: usize = 50;

/// System for tracking and processing structural integrity checks
pub struct StructuralIntegritySystem {
    /// Queue of positions that need structural checks (world coordinates)
    check_queue: HashSet<IVec2>,
}

impl StructuralIntegritySystem {
    pub fn new() -> Self {
        Self {
            check_queue: HashSet::new(),
        }
    }

    /// Schedule a structural check at the given world position
    /// This should be called when a structural material is removed
    pub fn schedule_check(&mut self, world_x: i32, world_y: i32) {
        self.check_queue.insert(IVec2::new(world_x, world_y));
    }

    /// Drain the check queue and return all positions
    /// Returns vector of positions that need checking
    pub fn drain_queue(&mut self) -> Vec<IVec2> {
        self.check_queue.drain().collect()
    }

    /// Process queued structural checks for a list of positions
    /// This is a static method to avoid borrow checker issues
    pub fn process_checks(world: &mut World, positions: Vec<IVec2>) -> usize {
        if positions.is_empty() {
            return 0;
        }

        let count = positions.len();
        log::debug!("Processing {} structural checks", count);

        for pos in positions {
            Self::check_position(world, pos.x, pos.y);
        }

        count
    }

    /// Check structural integrity at a specific position
    fn check_position(world: &mut World, world_x: i32, world_y: i32) {
        log::debug!("Structural: Checking position ({}, {})", world_x, world_y);

        // Get the pixel that was removed - check all 4 neighbors
        let neighbors = [
            (world_x, world_y + 1), // Above
            (world_x + 1, world_y), // Right
            (world_x, world_y - 1), // Below
            (world_x - 1, world_y), // Left
        ];

        for (nx, ny) in neighbors {
            if let Some(pixel) = world.get_pixel(nx, ny) {
                if pixel.is_empty() {
                    continue;
                }

                // Only check structural solids
                let material = world.materials().get(pixel.material_id);
                if !material.structural || material.material_type != MaterialType::Solid {
                    continue;
                }

                // Perform flood fill to find connected region
                let region = Self::flood_fill_structural(world, nx, ny);
                log::debug!(
                    "Structural: Flood fill from ({}, {}): found {} pixels",
                    nx,
                    ny,
                    region.len()
                );

                // Check if region is anchored (connected to bedrock)
                let is_anchored = Self::is_region_anchored(world, &region);
                log::debug!("Structural: Region anchored={}", is_anchored);

                if !is_anchored {
                    // Convert based on size
                    if region.len() < SMALL_DEBRIS_THRESHOLD {
                        log::info!(
                            "Structural: Converting {} pixels to sand particles",
                            region.len()
                        );
                        Self::convert_to_particles(world, region);
                    } else {
                        // Large debris - create rigid body
                        log::info!(
                            "Structural: Converting {} pixels to rigid body",
                            region.len()
                        );
                        Self::convert_to_rigid_body(world, region);
                    }
                }
            }
        }
    }

    /// Flood fill to find all connected structural solids
    /// Returns set of world coordinates
    fn flood_fill_structural(world: &World, start_x: i32, start_y: i32) -> HashSet<IVec2> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        let start_pixel = match world.get_pixel(start_x, start_y) {
            Some(p) if !p.is_empty() => p,
            _ => return visited,
        };

        let start_material = world.materials().get(start_pixel.material_id);
        if !start_material.structural || start_material.material_type != MaterialType::Solid {
            return visited;
        }

        queue.push_back(IVec2::new(start_x, start_y));
        visited.insert(IVec2::new(start_x, start_y));

        let origin = IVec2::new(start_x, start_y);

        while let Some(pos) = queue.pop_front() {
            // Distance limit to prevent runaway flood fills
            if (pos - origin).abs().max_element() > MAX_FLOOD_FILL_RADIUS {
                continue;
            }

            // Check 4-connected neighbors
            let neighbors = [
                IVec2::new(pos.x, pos.y + 1),
                IVec2::new(pos.x + 1, pos.y),
                IVec2::new(pos.x, pos.y - 1),
                IVec2::new(pos.x - 1, pos.y),
            ];

            for neighbor in neighbors {
                if visited.contains(&neighbor) {
                    continue;
                }

                if let Some(pixel) = world.get_pixel(neighbor.x, neighbor.y) {
                    if pixel.is_empty() {
                        continue;
                    }

                    let material = world.materials().get(pixel.material_id);

                    // Only traverse structural solids
                    if material.structural && material.material_type == MaterialType::Solid {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        visited
    }

    /// Check if any pixel in the region connects to bedrock
    fn is_region_anchored(world: &World, region: &HashSet<IVec2>) -> bool {
        // A region is anchored if it contains bedrock
        for pos in region {
            if let Some(pixel) = world.get_pixel(pos.x, pos.y) {
                if pixel.material_id == MaterialId::BEDROCK {
                    return true;
                }
            }
        }
        false
    }

    /// Convert small debris to powder particles (sand)
    fn convert_to_particles(world: &mut World, region: HashSet<IVec2>) {
        log::info!("Converting {} pixels to particles", region.len());

        for pos in region {
            // Convert to sand (powder that will fall naturally)
            world.set_pixel(pos.x, pos.y, MaterialId::SAND);
        }
    }

    /// Convert large debris to a falling rigid body
    fn convert_to_rigid_body(world: &mut World, region: HashSet<IVec2>) {
        log::info!("Converting {} pixels to rigid body", region.len());
        world.create_debris(region);
    }
}

impl Default for StructuralIntegritySystem {
    fn default() -> Self {
        Self::new()
    }
}
