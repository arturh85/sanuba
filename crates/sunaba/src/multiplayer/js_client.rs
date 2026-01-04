//! JavaScript/TypeScript SpacetimeDB SDK bindings for WASM
//!
//! This module provides Rust bindings to the SpacetimeDB TypeScript SDK
//! running in the browser via wasm-bindgen.

use wasm_bindgen::prelude::*;

// Import JavaScript functions from the global window object
// These will be implemented in web/index.html using the TypeScript SDK
#[wasm_bindgen]
extern "C" {
    /// Connect to SpacetimeDB server
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "connect", catch)]
    async fn js_connect(host: &str, db_name: &str) -> Result<JsValue, JsValue>;

    /// Subscribe to world state tables
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "subscribeWorld", catch)]
    async fn js_subscribe_world() -> Result<(), JsValue>;

    /// Send player position update
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "updatePlayerPosition", catch)]
    fn js_update_player_position(x: f32, y: f32, vel_x: f32, vel_y: f32) -> Result<(), JsValue>;

    /// Send material placement
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "placeMaterial", catch)]
    fn js_place_material(x: i32, y: i32, material_id: u16) -> Result<(), JsValue>;

    /// Send mining action
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "mine", catch)]
    fn js_mine(x: i32, y: i32) -> Result<(), JsValue>;

    /// Check if connected to server
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "isConnected")]
    fn js_is_connected() -> bool;

    /// Send ping request for latency measurement
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "requestPing", catch)]
    async fn js_request_ping(timestamp_ms: f64) -> Result<(), JsValue>;

    /// Get latest server metrics from JavaScript cache
    #[wasm_bindgen(js_namespace = ["window", "spacetimeClient"], js_name = "getLatestServerMetrics")]
    fn js_get_latest_server_metrics() -> JsValue;
}

/// Server performance metrics (matches server schema)
#[derive(Debug, Clone)]
pub struct ServerMetrics {
    pub tick: u64,
    pub timestamp_ms: u64,
    pub world_tick_time_ms: f32,
    pub creature_tick_time_ms: f32,
    pub active_chunks: u32,
    pub dirty_chunks_synced: u32,
    pub online_players: u32,
    pub creatures_alive: u32,
}

impl ServerMetrics {
    /// Parse ServerMetrics from JavaScript object
    fn from_js_value(val: JsValue) -> Option<Self> {
        if val.is_null() || val.is_undefined() {
            return None;
        }

        use wasm_bindgen::JsCast;
        let obj = val.dyn_into::<js_sys::Object>().ok()?;

        // Helper to extract u64 field
        let get_u64 = |key: &str| -> Option<u64> {
            js_sys::Reflect::get(&obj, &JsValue::from_str(key))
                .ok()?
                .as_f64()
                .map(|v| v as u64)
        };

        // Helper to extract f32 field
        let get_f32 = |key: &str| -> Option<f32> {
            js_sys::Reflect::get(&obj, &JsValue::from_str(key))
                .ok()?
                .as_f64()
                .map(|v| v as f32)
        };

        // Helper to extract u32 field
        let get_u32 = |key: &str| -> Option<u32> {
            js_sys::Reflect::get(&obj, &JsValue::from_str(key))
                .ok()?
                .as_f64()
                .map(|v| v as u32)
        };

        Some(Self {
            tick: get_u64("tick")?,
            timestamp_ms: get_u64("timestamp_ms")?,
            world_tick_time_ms: get_f32("world_tick_time_ms")?,
            creature_tick_time_ms: get_f32("creature_tick_time_ms")?,
            active_chunks: get_u32("active_chunks")?,
            dirty_chunks_synced: get_u32("dirty_chunks_synced")?,
            online_players: get_u32("online_players")?,
            creatures_alive: get_u32("creatures_alive")?,
        })
    }
}

/// SpacetimeDB client wrapper for WASM (uses TypeScript SDK via JavaScript)
pub struct MultiplayerClient {
    connected: bool,
}

impl MultiplayerClient {
    /// Create a new multiplayer client (not yet connected)
    pub fn new() -> Self {
        Self { connected: false }
    }

    /// Connect to SpacetimeDB server
    pub async fn connect(
        &mut self,
        host: impl Into<String>,
        db_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        let host = host.into();
        let db_name = db_name.into();

        log::info!(
            "Connecting to SpacetimeDB at {}/{} (via JS SDK)",
            host,
            db_name
        );

        js_connect(&host, &db_name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect: {:?}", e))?;

        self.connected = true;
        log::info!("Connected to SpacetimeDB via JavaScript SDK");

        Ok(())
    }

    /// Subscribe to world state (chunks, players, creatures)
    pub async fn subscribe_world(&mut self) -> anyhow::Result<()> {
        log::info!("Subscribing to world state tables (via JS SDK)");

        js_subscribe_world()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to subscribe: {:?}", e))?;

        log::info!("Subscribed to world state");

        Ok(())
    }

    /// Send player position update to server
    pub fn update_player_position(&self, x: f32, y: f32) -> anyhow::Result<()> {
        if !self.connected {
            anyhow::bail!("Not connected to server");
        }

        // TODO: Pass actual velocity from player state instead of zeros
        js_update_player_position(x, y, 0.0, 0.0)
            .map_err(|e| anyhow::anyhow!("Failed to update player position: {:?}", e))?;

        Ok(())
    }

    /// Request material placement at position
    pub fn place_material(&self, x: i32, y: i32, material_id: u16) -> anyhow::Result<()> {
        if !self.connected {
            anyhow::bail!("Not connected to server");
        }

        js_place_material(x, y, material_id)
            .map_err(|e| anyhow::anyhow!("Failed to place material: {:?}", e))?;

        Ok(())
    }

    /// Request mining at position
    pub fn mine(&self, x: i32, y: i32) -> anyhow::Result<()> {
        if !self.connected {
            anyhow::bail!("Not connected to server");
        }

        js_mine(x, y).map_err(|e| anyhow::anyhow!("Failed to mine: {:?}", e))?;

        Ok(())
    }

    /// Get chunk data from local cache (for rendering)
    ///
    /// Note: Chunk data flows through subscription callbacks in JavaScript
    pub fn get_chunk(&self, _x: i32, _y: i32) -> Option<Vec<u8>> {
        // TODO: Implement chunk cache access via JavaScript
        None
    }

    /// Send ping request to server for latency measurement
    pub fn request_ping(&self, timestamp_ms: u64) -> anyhow::Result<()> {
        if !self.connected {
            anyhow::bail!("Not connected to server");
        }

        // Convert to f64 for JavaScript (acceptable precision loss)
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = js_request_ping(timestamp_ms as f64).await {
                log::error!("Failed to send ping: {:?}", e);
            }
        });

        Ok(())
    }

    /// Get latest server metrics from JavaScript cache
    pub fn get_latest_server_metrics(&self) -> Option<ServerMetrics> {
        if !self.connected {
            return None;
        }

        let js_val = js_get_latest_server_metrics();
        ServerMetrics::from_js_value(js_val)
    }

    /// Check if connected to server
    pub fn is_connected(&self) -> bool {
        self.connected && js_is_connected()
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        log::info!("Disconnecting from SpacetimeDB (JS SDK)");
        self.connected = false;
        Ok(())
    }
}

impl Default for MultiplayerClient {
    fn default() -> Self {
        Self::new()
    }
}
