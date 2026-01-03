//! Toast notification manager using egui-notify

use egui_notify::Toasts;
use std::time::Duration;

/// Toast notification manager wrapping egui-notify
pub struct ToastManager {
    toasts: Toasts,
}

impl ToastManager {
    /// Create a new toast manager with default settings
    pub fn new() -> Self {
        Self {
            toasts: Toasts::default()
                .with_anchor(egui_notify::Anchor::TopRight)
                .with_margin(egui::vec2(10.0, 50.0)),
        }
    }

    /// Show an info toast
    pub fn info(&mut self, message: impl Into<String>) {
        let msg: String = message.into();
        self.toasts.info(msg).duration(Some(Duration::from_secs(3)));
    }

    /// Show a success toast
    pub fn success(&mut self, message: impl Into<String>) {
        let msg: String = message.into();
        self.toasts
            .success(msg)
            .duration(Some(Duration::from_secs(3)));
    }

    /// Show a warning toast
    pub fn warning(&mut self, message: impl Into<String>) {
        let msg: String = message.into();
        self.toasts
            .warning(msg)
            .duration(Some(Duration::from_secs(3)));
    }

    /// Show an error toast
    pub fn error(&mut self, message: impl Into<String>) {
        let msg: String = message.into();
        self.toasts
            .error(msg)
            .duration(Some(Duration::from_secs(5)));
    }

    /// Render toast notifications - call this every frame
    pub fn render(&mut self, ctx: &egui::Context) {
        self.toasts.show(ctx);
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}
