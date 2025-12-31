use crate::levels::LevelManager;

/// State for the level selector UI panel
pub struct LevelSelectorState {
    pub visible: bool,
    pub selected_level: Option<usize>,
    pub return_to_world: bool,
}

impl LevelSelectorState {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected_level: None,
            return_to_world: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Render the level selector window
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        level_manager: &LevelManager,
        game_mode_desc: &str,
        in_persistent_world: bool,
    ) {
        // Reset selection flags at start of frame
        self.selected_level = None;
        self.return_to_world = false;

        if !self.visible {
            return;
        }

        egui::Window::new("Level Selector")
            .default_pos(egui::pos2(400.0, 100.0))
            .default_width(400.0)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("Game Mode");

                // Current mode display
                ui.horizontal(|ui| {
                    ui.label("Current:");
                    if in_persistent_world {
                        ui.colored_label(egui::Color32::GREEN, game_mode_desc);
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, game_mode_desc);
                    }
                });

                if in_persistent_world {
                    ui.colored_label(egui::Color32::GREEN, "✓ Auto-save enabled");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ Changes not saved");
                }

                ui.separator();

                // Button to return to persistent world
                if !in_persistent_world {
                    if ui
                        .button("🏠 Return to Persistent World")
                        .on_hover_text("Return to your saved world")
                        .clicked()
                    {
                        self.return_to_world = true;
                        self.visible = false;
                    }
                    ui.separator();
                }

                ui.heading("Demo Levels");
                ui.label("Select a level to test physics and mechanics:");

                ui.add_space(5.0);

                // Scrollable list of demo levels
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    let levels = level_manager.levels();
                    for (idx, level) in levels.iter().enumerate() {
                        let is_current = !in_persistent_world
                            && level_manager.current_level() == idx;

                        let mut button_text = format!("{}. {}", idx + 1, level.name);
                        if is_current {
                            button_text.push_str(" ◄");
                        }

                        let button = egui::Button::new(&button_text);
                        let mut response = ui.add(button);

                        if is_current {
                            response = response.highlight();
                        }

                        if response
                            .on_hover_text(level.description)
                            .clicked()
                        {
                            self.selected_level = Some(idx);
                            self.visible = false;
                        }

                        // Show description below button
                        ui.label(
                            egui::RichText::new(format!("   {}", level.description))
                                .size(11.0)
                                .color(egui::Color32::GRAY),
                        );
                        ui.add_space(5.0);
                    }
                });

                ui.separator();
                ui.label(
                    egui::RichText::new("Tip: Press L to toggle this menu")
                        .size(11.0)
                        .italics(),
                );
            });
    }
}

impl Default for LevelSelectorState {
    fn default() -> Self {
        Self::new()
    }
}
