//! In-game log viewer panel using egui_logger

/// Panel for viewing application logs in-game
pub struct LoggerPanel {
    /// Whether the panel is open
    pub open: bool,
}

impl LoggerPanel {
    pub fn new() -> Self {
        Self { open: false }
    }

    /// Toggle panel visibility
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// Render the logger panel
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        egui::Window::new("Log")
            .default_pos(egui::pos2(10.0, 400.0))
            .default_size([500.0, 300.0])
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                egui_logger::logger_ui().show(ui);
            });
    }
}

impl Default for LoggerPanel {
    fn default() -> Self {
        Self::new()
    }
}
