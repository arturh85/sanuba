//! Crafting system with recipes and material transformation

use crate::entity::inventory::Inventory;
use crate::simulation::MaterialId;
use serde::{Deserialize, Serialize};

/// Crafting recipe definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: u16,
    pub name: String,
    pub inputs: Vec<(u16, u32)>, // (material_id, count)
    pub output: RecipeOutput,
    pub workstation: Option<WorkstationType>,
}

/// Recipe output (material or tool)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipeOutput {
    Material { id: u16, count: u32 },
    Tool { tool_id: u16, durability: u32 },
}

/// Workstation types (Phase 8: placed in world, Phase 5: hand crafting only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkstationType {
    Furnace, // Smelting (Phase 8)
    Anvil,   // Tool/weapon crafting (Phase 8)
    Alchemy, // Potions (future)
}

/// Recipe registry
#[derive(Debug, Clone)]
pub struct RecipeRegistry {
    recipes: Vec<Recipe>,
}

impl RecipeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            recipes: Vec::new(),
        };
        registry.register_default_recipes();
        registry
    }

    fn register_default_recipes(&mut self) {
        // === TOOLS ===

        // Wood Pickaxe: 5 wood
        self.register(Recipe {
            id: 0,
            name: "Wood Pickaxe".to_string(),
            inputs: vec![(MaterialId::WOOD, 5)],
            output: RecipeOutput::Tool {
                tool_id: 1000,
                durability: 50,
            },
            workstation: None, // Hand crafting
        });

        // Stone Pickaxe: 3 stone + 2 wood
        self.register(Recipe {
            id: 1,
            name: "Stone Pickaxe".to_string(),
            inputs: vec![(MaterialId::STONE, 3), (MaterialId::WOOD, 2)],
            output: RecipeOutput::Tool {
                tool_id: 1001,
                durability: 100,
            },
            workstation: None,
        });

        // Iron Pickaxe: 3 iron ingot + 2 wood
        self.register(Recipe {
            id: 2,
            name: "Iron Pickaxe".to_string(),
            inputs: vec![(MaterialId::IRON_INGOT, 3), (MaterialId::WOOD, 2)],
            output: RecipeOutput::Tool {
                tool_id: 1002,
                durability: 400,
            },
            workstation: None, // Phase 8: require anvil
        });

        // === MATERIALS ===

        // Fertilizer: 3 ash + 2 plant matter
        self.register(Recipe {
            id: 100,
            name: "Fertilizer".to_string(),
            inputs: vec![(MaterialId::ASH, 3), (MaterialId::PLANT_MATTER, 2)],
            output: RecipeOutput::Material {
                id: MaterialId::FERTILIZER,
                count: 5,
            },
            workstation: None,
        });

        // Gunpowder: 1 coal + 1 fertilizer (simplified chemistry)
        self.register(Recipe {
            id: 101,
            name: "Gunpowder".to_string(),
            inputs: vec![(MaterialId::COAL_ORE, 1), (MaterialId::FERTILIZER, 1)],
            output: RecipeOutput::Material {
                id: MaterialId::GUNPOWDER,
                count: 2,
            },
            workstation: None,
        });
    }

    fn register(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Get all recipes
    pub fn all_recipes(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Get all craftable recipes (player has materials)
    pub fn get_craftable<'a>(&'a self, inventory: &Inventory) -> Vec<&'a Recipe> {
        self.recipes
            .iter()
            .filter(|recipe| self.can_craft(recipe, inventory))
            .collect()
    }

    /// Check if player can craft a recipe
    pub fn can_craft(&self, recipe: &Recipe, inventory: &Inventory) -> bool {
        recipe
            .inputs
            .iter()
            .all(|(mat_id, count)| inventory.has_item(*mat_id, *count))
    }

    /// Attempt to craft, consuming materials
    /// Returns Some(output) if successful, None if not enough materials
    pub fn try_craft(&self, recipe: &Recipe, inventory: &mut Inventory) -> Option<RecipeOutput> {
        if !self.can_craft(recipe, inventory) {
            return None;
        }

        // Consume inputs
        for (mat_id, count) in &recipe.inputs {
            let removed = inventory.remove_item(*mat_id, *count);
            if removed != *count {
                log::error!(
                    "[CRAFTING] Failed to remove {} x{} from inventory (only removed {})",
                    mat_id,
                    count,
                    removed
                );
                return None;
            }
        }

        Some(recipe.output.clone())
    }

    /// Get recipe by ID
    pub fn get(&self, id: u16) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == id)
    }
}

impl Default for RecipeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::inventory::Inventory;

    #[test]
    fn test_recipe_registry_creation() {
        let registry = RecipeRegistry::new();
        assert_eq!(registry.all_recipes().len(), 5); // 3 tools + 2 materials
    }

    #[test]
    fn test_can_craft_with_materials() {
        let registry = RecipeRegistry::new();
        let mut inv = Inventory::new(50);

        // Add 5 wood
        inv.add_item(MaterialId::WOOD, 5);

        // Should be able to craft wood pickaxe
        let wood_pickaxe_recipe = registry.get(0).unwrap();
        assert!(registry.can_craft(wood_pickaxe_recipe, &inv));
    }

    #[test]
    fn test_cannot_craft_without_materials() {
        let registry = RecipeRegistry::new();
        let inv = Inventory::new(50);

        // Empty inventory
        let wood_pickaxe_recipe = registry.get(0).unwrap();
        assert!(!registry.can_craft(wood_pickaxe_recipe, &inv));
    }

    #[test]
    fn test_craft_tool() {
        let registry = RecipeRegistry::new();
        let mut inv = Inventory::new(50);

        // Add materials for wood pickaxe
        inv.add_item(MaterialId::WOOD, 10);

        let wood_pickaxe_recipe = registry.get(0).unwrap();

        // Craft the pickaxe
        let output = registry.try_craft(wood_pickaxe_recipe, &mut inv);
        assert!(output.is_some());

        // Check output
        match output.unwrap() {
            RecipeOutput::Tool {
                tool_id,
                durability,
            } => {
                assert_eq!(tool_id, 1000);
                assert_eq!(durability, 50);
            }
            _ => panic!("Expected tool output"),
        }

        // Check materials were consumed (10 - 5 = 5 remaining)
        assert_eq!(inv.count_item(MaterialId::WOOD), 5);
    }

    #[test]
    fn test_craft_material() {
        let registry = RecipeRegistry::new();
        let mut inv = Inventory::new(50);

        // Add materials for fertilizer (3 ash + 2 plant matter)
        inv.add_item(MaterialId::ASH, 5);
        inv.add_item(MaterialId::PLANT_MATTER, 4);

        let fertilizer_recipe = registry.get(100).unwrap();

        // Craft fertilizer
        let output = registry.try_craft(fertilizer_recipe, &mut inv);
        assert!(output.is_some());

        // Check output
        match output.unwrap() {
            RecipeOutput::Material { id, count } => {
                assert_eq!(id, MaterialId::FERTILIZER);
                assert_eq!(count, 5);
            }
            _ => panic!("Expected material output"),
        }

        // Check materials were consumed
        assert_eq!(inv.count_item(MaterialId::ASH), 2); // 5 - 3 = 2
        assert_eq!(inv.count_item(MaterialId::PLANT_MATTER), 2); // 4 - 2 = 2
    }

    #[test]
    fn test_get_craftable() {
        let registry = RecipeRegistry::new();
        let mut inv = Inventory::new(50);

        // Empty inventory - no recipes craftable
        assert_eq!(registry.get_craftable(&inv).len(), 0);

        // Add wood - wood pickaxe craftable
        inv.add_item(MaterialId::WOOD, 10);
        assert_eq!(registry.get_craftable(&inv).len(), 1);

        // Add stone - stone pickaxe also craftable
        inv.add_item(MaterialId::STONE, 5);
        assert_eq!(registry.get_craftable(&inv).len(), 2);
    }
}
