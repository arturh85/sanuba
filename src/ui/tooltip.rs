//! Mouseover tooltip showing pixel information

use crate::world::World;
use crate::simulation::Materials;

/// Tooltip state for displaying pixel information at cursor
pub struct TooltipState {
    visible: bool,
    material_name: String,
    temperature: f32,
    world_pos: (i32, i32),
}

impl TooltipState {
    pub fn new() -> Self {
        Self {
            visible: false,
            material_name: String::from("Air"),
            temperature: 20.0,
            world_pos: (0, 0),
        }
    }

    /// Update tooltip with information from world at mouse position
    pub fn update(&mut self, world: &World, materials: &Materials, mouse_world_pos: Option<(i32, i32)>) {
        if let Some((wx, wy)) = mouse_world_pos {
            self.world_pos = (wx, wy);

            // Query pixel at mouse position
            if let Some(pixel) = world.get_pixel(wx, wy) {
                if pixel.is_empty() {
                    self.visible = false;
                    self.material_name = String::from("Air");
                    self.temperature = 20.0;
                } else {
                    self.visible = true;
                    let material = materials.get(pixel.material_id);
                    self.material_name = material.name.clone();

                    // Get temperature at this pixel
                    self.temperature = world.get_temperature_at_pixel(wx, wy);
                }
            } else {
                // No chunk loaded at this position
                self.visible = false;
            }
        } else {
            self.visible = false;
        }
    }

    /// Render tooltip near cursor position
    pub fn render(&self, ctx: &egui::Context, cursor_screen_pos: egui::Pos2) {
        if !self.visible {
            return;
        }

        // Offset tooltip slightly from cursor to avoid blocking view
        let tooltip_pos = egui::pos2(cursor_screen_pos.x + 20.0, cursor_screen_pos.y + 20.0);

        egui::Window::new("##tooltip")
            .title_bar(false)
            .resizable(false)
            .fixed_pos(tooltip_pos)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
                stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                inner_margin: egui::Margin::same(8.0),
                outer_margin: egui::Margin::same(0.0),
                rounding: egui::Rounding::same(4.0),
                shadow: egui::epaint::Shadow::NONE,
            })
            .show(ctx, |ui| {
                ui.label(egui::RichText::new(&self.material_name)
                    .color(egui::Color32::WHITE)
                    .strong());
                ui.label(egui::RichText::new(format!("Temp: {:.0}°C", self.temperature))
                    .color(egui::Color32::LIGHT_GRAY)
                    .size(12.0));
                ui.label(egui::RichText::new(format!("Pos: ({}, {})", self.world_pos.0, self.world_pos.1))
                    .color(egui::Color32::DARK_GRAY)
                    .size(11.0));
            });
    }
}

impl Default for TooltipState {
    fn default() -> Self {
        Self::new()
    }
}
