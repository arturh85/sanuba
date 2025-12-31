//! Level definition and management

use crate::world::World;

/// A level definition with metadata and generator function
pub struct LevelDef {
    pub id: usize,
    pub name: &'static str,
    pub description: &'static str,
    pub generator: fn(&mut World),
}

/// Manages level selection and switching
pub struct LevelManager {
    levels: Vec<LevelDef>,
    current_level: usize,
}

impl LevelManager {
    /// Create a new level manager with all demo levels
    pub fn new() -> Self {
        use super::demo_levels::*;

        let levels = vec![
            LevelDef {
                id: 0,
                name: "Basic Physics Playground",
                description: "Sand and water demonstration",
                generator: generate_level_1_basic_physics,
            },
            LevelDef {
                id: 1,
                name: "Inferno",
                description: "Fire propagation through wood",
                generator: generate_level_2_inferno,
            },
            LevelDef {
                id: 2,
                name: "Lava Meets Water",
                description: "Chemical reactions demo",
                generator: generate_level_3_lava_water,
            },
            LevelDef {
                id: 3,
                name: "Material Showcase",
                description: "All materials side-by-side",
                generator: generate_level_4_showcase,
            },
            LevelDef {
                id: 4,
                name: "Powder Paradise",
                description: "Sand and powder physics",
                generator: generate_level_5_powder,
            },
            LevelDef {
                id: 5,
                name: "Liquid Lab",
                description: "Water and oil interactions",
                generator: generate_level_6_liquids,
            },
            LevelDef {
                id: 6,
                name: "Steam Engine",
                description: "Heat and steam generation",
                generator: generate_level_7_steam,
            },
            LevelDef {
                id: 7,
                name: "Volcano",
                description: "Lava eruption demo",
                generator: generate_level_8_volcano,
            },
            LevelDef {
                id: 8,
                name: "Bridge Demolition",
                description: "Remove pillars to collapse the bridge (large debris demo)",
                generator: generate_level_9_bridge,
            },
            LevelDef {
                id: 9,
                name: "Tower Collapse",
                description: "Watch towers crumble - small vs large debris",
                generator: generate_level_10_towers,
            },
            LevelDef {
                id: 10,
                name: "Floating Islands",
                description: "Cut support columns to drop floating islands",
                generator: generate_level_11_islands,
            },
            LevelDef {
                id: 11,
                name: "Crumbling Wall",
                description: "Strategic wall demolition - mixed debris sizes",
                generator: generate_level_12_wall,
            },
            LevelDef {
                id: 12,
                name: "Castle Siege",
                description: "Destroy the castle foundation for cascading collapse",
                generator: generate_level_13_castle,
            },
            LevelDef {
                id: 13,
                name: "Domino Effect",
                description: "Knock over the first domino and watch the chain reaction",
                generator: generate_level_14_domino,
            },
            LevelDef {
                id: 14,
                name: "Quarry",
                description: "Mine support beams to collapse layered stone",
                generator: generate_level_15_quarry,
            },
            LevelDef {
                id: 15,
                name: "Stress Test",
                description: "Remove the critical support - massive structure stress test",
                generator: generate_level_16_stress,
            },
        ];

        Self {
            levels,
            current_level: 0,
        }
    }

    /// Get current level name
    pub fn current_level_name(&self) -> &str {
        self.levels[self.current_level].name
    }

    /// Get current level description
    pub fn current_level_description(&self) -> &str {
        self.levels[self.current_level].description
    }

    /// Switch to next level
    pub fn next_level(&mut self, world: &mut World) {
        self.current_level = (self.current_level + 1) % self.levels.len();
        self.load_current_level(world);
        log::info!("Switched to level {}: {}", self.current_level, self.current_level_name());
    }

    /// Switch to previous level
    pub fn prev_level(&mut self, world: &mut World) {
        if self.current_level == 0 {
            self.current_level = self.levels.len() - 1;
        } else {
            self.current_level -= 1;
        }
        self.load_current_level(world);
        log::info!("Switched to level {}: {}", self.current_level, self.current_level_name());
    }

    /// Load the current level
    pub fn load_current_level(&self, world: &mut World) {
        let level = &self.levels[self.current_level];
        (level.generator)(world);
    }
}

impl Default for LevelManager {
    fn default() -> Self {
        Self::new()
    }
}
