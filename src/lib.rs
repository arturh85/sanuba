//! # Sunaba - 2D Falling Sand Physics Sandbox
//!
//! A survival game where every pixel is simulated with material properties.

pub mod app;
pub mod world;
pub mod simulation;
pub mod physics;
pub mod render;
pub mod ui;
pub mod levels;
pub mod entity;

pub use app::App;

/// Common imports for internal use
pub mod prelude {
    pub use crate::world::{Chunk, World, Pixel, CHUNK_SIZE};
    pub use crate::simulation::{Materials, MaterialId, MaterialType};
    pub use glam::{Vec2, IVec2};
}

// WASM entry point
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();

    // Initialize logging for WASM
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

    log::info!("Sunaba WASM module initialized");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn run() -> Result<(), JsValue> {
    log::info!("Starting Sunaba (WASM)");

    let app = App::new()
        .await
        .map_err(|e| JsValue::from_str(&format!("Failed to create app: {}", e)))?;

    app.run()
        .map_err(|e| JsValue::from_str(&format!("Failed to run app: {}", e)))
}
