//! Crafting UI panel

use crate::entity::crafting::{Recipe, RecipeOutput, RecipeRegistry};
use crate::entity::inventory::Inventory;
use crate::simulation::Materials;
use egui::{Color32, RichText};

pub struct CraftingUI {
    pub visible: bool,
}

impl CraftingUI {
    pub fn new() -> Self {
        Self { visible: false }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Render the crafting window
    /// Returns Some(output) if a recipe was crafted this frame
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        inventory: &mut Inventory,
        recipes: &RecipeRegistry,
        materials: &Materials,
    ) -> Option<RecipeOutput> {
        if !self.visible {
            return None;
        }

        let mut crafted_output = None;

        egui::Window::new("Crafting")
            .default_width(500.0)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                ui.heading("Available Recipes");
                ui.add_space(10.0);

                let craftable = recipes.get_craftable(inventory);

                if craftable.is_empty() {
                    ui.colored_label(Color32::GRAY, "No craftable recipes");
                    ui.label("Gather more materials to unlock recipes.");
                    return;
                }

                // Render each craftable recipe
                for recipe in &craftable {
                    ui.separator();
                    crafted_output = Self::render_recipe(ui, recipe, inventory, recipes, materials);
                    if crafted_output.is_some() {
                        break; // Only craft one item per frame
                    }
                }

                ui.add_space(10.0);
                ui.separator();
                ui.label(format!("Total recipes: {}", recipes.all_recipes().len()));
                ui.label(format!("Craftable: {}", craftable.len()));
            });

        crafted_output
    }

    /// Render a single recipe
    /// Returns Some(output) if the recipe was crafted
    fn render_recipe(
        ui: &mut egui::Ui,
        recipe: &Recipe,
        inventory: &mut Inventory,
        recipes: &RecipeRegistry,
        materials: &Materials,
    ) -> Option<RecipeOutput> {
        let mut crafted = None;

        ui.horizontal(|ui| {
            // Recipe name
            ui.label(RichText::new(&recipe.name).size(16.0).strong());

            ui.add_space(10.0);

            // Craft button
            if ui.button("Craft").clicked() {
                crafted = recipes.try_craft(recipe, inventory);
                if crafted.is_some() {
                    log::info!("[CRAFTING] Crafted: {}", recipe.name);
                } else {
                    log::warn!("[CRAFTING] Failed to craft: {}", recipe.name);
                }
            }
        });

        // Show inputs
        ui.horizontal(|ui| {
            ui.label(RichText::new("Requires:").color(Color32::GRAY));

            for (i, (mat_id, count)) in recipe.inputs.iter().enumerate() {
                if i > 0 {
                    ui.label("+");
                }

                let mat_name = &materials.get(*mat_id).name;
                let has_count = inventory.count_item(*mat_id);

                let text = format!("{} x{}", mat_name, count);
                let color = if has_count >= *count {
                    Color32::from_rgb(100, 255, 100) // Green if we have enough
                } else {
                    Color32::from_rgb(255, 100, 100) // Red if not enough
                };

                ui.colored_label(color, text);
            }
        });

        // Show output
        ui.horizontal(|ui| {
            ui.label(RichText::new("Produces:").color(Color32::GRAY));

            match &recipe.output {
                RecipeOutput::Material { id, count } => {
                    let mat_name = &materials.get(*id).name;
                    ui.colored_label(
                        Color32::from_rgb(150, 200, 255),
                        format!("{} x{}", mat_name, count),
                    );
                }
                RecipeOutput::Tool {
                    tool_id,
                    durability,
                } => {
                    let tool_name = match *tool_id {
                        1000 => "Wood Pickaxe",
                        1001 => "Stone Pickaxe",
                        1002 => "Iron Pickaxe",
                        _ => "Unknown Tool",
                    };
                    ui.colored_label(
                        Color32::from_rgb(255, 215, 0), // Gold color for tools
                        format!("{} ({}⚒)", tool_name, durability),
                    );
                }
            }
        });

        ui.add_space(5.0);

        crafted
    }
}

impl Default for CraftingUI {
    fn default() -> Self {
        Self::new()
    }
}
