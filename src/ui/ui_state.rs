//! Central UI state management

use super::stats::StatsCollector;
use super::tooltip::TooltipState;
use super::controls_help::ControlsHelpState;

/// Central UI state container
pub struct UiState {
    /// Stats collector and display
    pub stats: StatsCollector,

    /// Whether stats window is visible
    pub stats_visible: bool,

    /// Tooltip for mouseover information
    pub tooltip: TooltipState,

    /// Controls help panel
    pub controls_help: ControlsHelpState,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            stats: StatsCollector::new(),
            stats_visible: true,  // Start with stats visible
            tooltip: TooltipState::new(),
            controls_help: ControlsHelpState::new(),
        }
    }

    /// Toggle stats visibility
    pub fn toggle_stats(&mut self) {
        self.stats_visible = !self.stats_visible;
    }

    /// Toggle controls help visibility
    pub fn toggle_help(&mut self) {
        self.controls_help.toggle();
    }

    /// Update tooltip with world data
    pub fn update_tooltip(&mut self, world: &crate::world::World, materials: &crate::simulation::Materials, mouse_world_pos: Option<(i32, i32)>) {
        self.tooltip.update(world, materials, mouse_world_pos);
    }

    /// Render all UI elements
    pub fn render(&mut self, ctx: &egui::Context, cursor_screen_pos: egui::Pos2, selected_material: u16, materials: &crate::simulation::Materials, level_name: &str) {
        if self.stats_visible {
            self.render_stats(ctx);
        }

        // Render controls help with level name
        self.controls_help.render_with_level(ctx, selected_material, materials, level_name);

        // Always render tooltip when it has valid data
        self.tooltip.render(ctx, cursor_screen_pos);
    }

    fn render_stats(&self, ctx: &egui::Context) {
        egui::Window::new("Debug Stats")
            .default_pos(egui::pos2(10.0, 10.0))
            .resizable(false)
            .collapsible(true)
            .show(ctx, |ui| {
                let stats = self.stats.stats();

                ui.heading("Performance");
                ui.label(format!("FPS: {:.1}", stats.fps));
                ui.label(format!("Frame: {:.2}ms", stats.frame_time_ms));
                ui.label(format!("  Sim: {:.2}ms", stats.sim_time_ms));

                ui.separator();
                ui.heading("World");
                ui.label(format!("Active Chunks: {}", stats.active_chunks));
                ui.label(format!("Total Chunks: {}", stats.total_chunks));
                ui.label(format!("Active Pixels: {}", stats.active_pixels));

                ui.separator();
                ui.heading("Temperature");
                ui.label(format!("Range: {:.0}°C - {:.0}°C", stats.min_temp, stats.max_temp));
                ui.label(format!("Average: {:.1}°C", stats.avg_temp));

                ui.separator();
                ui.heading("Activity");
                ui.label(format!("Moved: {} pixels", stats.pixels_moved));
                ui.label(format!("Reactions: {}", stats.reactions));
                ui.label(format!("State Changes: {}", stats.state_changes));
            });
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
