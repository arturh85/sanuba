//! SpacetimeDB Rust SDK client wrapper for native multiplayer integration

use anyhow::Context;
use std::sync::{Arc, Mutex};

// Import generated SpacetimeDB client bindings
use super::generated::{self, DbConnection};
use generated::chunk_data_table::ChunkDataTableAccess;
use generated::creature_data_table::CreatureDataTableAccess;
use generated::player_table::PlayerTableAccess;
use generated::request_ping_reducer::request_ping;
use generated::server_metrics_table::ServerMetricsTableAccess;
use generated::{player_mine, player_place_material, player_update_position};
use spacetimedb_sdk::{DbContext, Table}; // Trait for connection and table methods

/// SpacetimeDB client wrapper for native multiplayer integration
pub struct MultiplayerClient {
    /// SpacetimeDB connection (wrapped in Arc<Mutex> for interior mutability)
    connection: Option<Arc<Mutex<DbConnection>>>,

    /// Server host URL
    host: String,

    /// Database name
    db_name: String,
}

impl MultiplayerClient {
    /// Create a new multiplayer client (not yet connected)
    pub fn new() -> Self {
        Self {
            connection: None,
            host: String::new(),
            db_name: String::new(),
        }
    }

    /// Connect to SpacetimeDB server
    pub async fn connect(
        &mut self,
        host: impl Into<String>,
        db_name: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.host = host.into();
        self.db_name = db_name.into();

        log::info!(
            "Connecting to SpacetimeDB at {}/{}",
            self.host,
            self.db_name
        );

        // Build connection using generated DbConnection
        let conn = DbConnection::builder()
            .on_connect(Self::on_connected)
            .on_connect_error(Self::on_connect_error)
            .on_disconnect(Self::on_disconnected)
            .with_uri(&self.host)
            .with_module_name(&self.db_name)
            .build()
            .context("Failed to build SpacetimeDB connection")?;

        self.connection = Some(Arc::new(Mutex::new(conn)));

        log::info!("Connected to SpacetimeDB successfully");

        Ok(())
    }

    /// Subscribe to world state (chunks, players, creatures)
    pub async fn subscribe_world(&mut self) -> anyhow::Result<()> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        log::info!("Subscribing to world state tables");

        let conn_guard = conn.lock().unwrap();

        // Subscribe to world config
        let _config_sub = conn_guard
            .subscription_builder()
            .on_applied(|_ctx| {
                log::debug!("World config subscription applied");
            })
            .subscribe("SELECT * FROM world_config");

        // Subscribe to chunk data with update callbacks
        let _chunk_sub = conn_guard
            .subscription_builder()
            .on_applied(|ctx| {
                log::info!(
                    "Chunk data subscription applied - {} chunks received",
                    ctx.db.chunk_data().iter().count()
                );
            })
            .subscribe("SELECT * FROM chunk_data");

        // Subscribe to players
        let _player_sub = conn_guard
            .subscription_builder()
            .on_applied(|ctx| {
                log::debug!(
                    "Player subscription applied - {} players",
                    ctx.db.player().iter().count()
                );
            })
            .subscribe("SELECT * FROM player");

        // Subscribe to creatures
        let _creature_sub = conn_guard
            .subscription_builder()
            .on_applied(|ctx| {
                log::debug!(
                    "Creature subscription applied - {} creatures",
                    ctx.db.creature_data().iter().count()
                );
            })
            .subscribe("SELECT * FROM creature_data");

        // Subscribe to server metrics
        let _metrics_sub = conn_guard
            .subscription_builder()
            .on_applied(|ctx| {
                log::debug!(
                    "Server metrics subscription applied - {} samples",
                    ctx.db.server_metrics().iter().count()
                );
            })
            .subscribe("SELECT * FROM server_metrics");

        log::info!("Subscribed to world state successfully");

        Ok(())
    }

    /// Process incoming messages (call this in your game loop)
    pub fn frame_tick(&self) {
        if let Some(ref conn) = self.connection {
            let conn_guard = conn.lock().unwrap();
            if let Err(e) = conn_guard.frame_tick() {
                log::error!("Error processing SpacetimeDB messages: {}", e);
            }
        }
    }

    /// Send player position update to server
    pub fn update_player_position(&self, x: f32, y: f32) -> anyhow::Result<()> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        let conn_guard = conn.lock().unwrap();
        conn_guard
            .reducers
            .player_update_position(x, y, 0.0, 0.0)
            .context("Failed to call player_update_position reducer")?;

        Ok(())
    }

    /// Request material placement at position
    pub fn place_material(&self, x: i32, y: i32, material_id: u16) -> anyhow::Result<()> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        let conn_guard = conn.lock().unwrap();
        conn_guard
            .reducers
            .player_place_material(x, y, material_id)
            .context("Failed to call player_place_material reducer")?;

        Ok(())
    }

    /// Request mining at position
    pub fn mine(&self, x: i32, y: i32) -> anyhow::Result<()> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        let conn_guard = conn.lock().unwrap();
        conn_guard
            .reducers
            .player_mine(x, y)
            .context("Failed to call player_mine reducer")?;

        Ok(())
    }

    /// Sync chunks from server cache to local world
    pub fn sync_chunks_to_world(
        &self,
        world: &mut sunaba_core::world::World,
    ) -> anyhow::Result<usize> {
        use glam::IVec2;

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        let conn_guard = conn.lock().unwrap();

        let mut synced_count = 0;

        // Iterate all cached chunks from server
        for chunk_row in conn_guard.db.chunk_data().iter() {
            let pos = IVec2::new(chunk_row.x, chunk_row.y);

            // Skip if already loaded
            if world.has_chunk(pos) {
                continue;
            }

            // Decode chunk data
            let Ok(chunk) = crate::encoding::decode_chunk(&chunk_row.pixel_data) else {
                log::warn!("Failed to decode chunk ({}, {})", chunk_row.x, chunk_row.y);
                continue;
            };

            // Insert into world
            world.insert_chunk(pos, chunk);
            synced_count += 1;
        }

        if synced_count > 0 {
            log::info!("Synced {} chunks from server", synced_count);
        }

        Ok(synced_count)
    }

    /// Get chunk data from local cache (for rendering)
    pub fn get_chunk(&self, _x: i32, _y: i32) -> Option<Vec<u8>> {
        let _conn = self.connection.as_ref()?;

        // Query chunk_data table from client cache
        // TODO: Implement chunk lookup from generated table accessors
        // conn.lock().unwrap().db.chunk_data().filter_by_chunk_x_chunk_y(&x, &y).first().map(|chunk| chunk.data.clone())

        None
    }

    /// Send ping request to server for latency measurement
    pub fn request_ping(&self, timestamp_ms: u64) -> anyhow::Result<()> {
        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not connected to server"))?;

        let conn_guard = conn.lock().unwrap();
        conn_guard
            .reducers
            .request_ping(timestamp_ms)
            .context("Failed to call request_ping reducer")?;

        Ok(())
    }

    /// Get latest server metrics from subscribed table
    pub fn get_latest_server_metrics(&self) -> Option<generated::ServerMetrics> {
        let conn = self.connection.as_ref()?;
        let conn_guard = conn.lock().unwrap();

        // Get the most recent metric by tick number
        conn_guard
            .db
            .server_metrics()
            .iter()
            .max_by_key(|m| m.tick)
            .map(|m| m.clone())
    }

    /// Check if connected to server
    pub fn is_connected(&self) -> bool {
        self.connection
            .as_ref()
            .map(|conn| {
                let conn_guard = conn.lock().unwrap();
                conn_guard.is_active()
            })
            .unwrap_or(false)
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        log::info!("Disconnecting from SpacetimeDB");

        if let Some(conn) = self.connection.take() {
            let conn_guard = conn.lock().unwrap();
            conn_guard
                .disconnect()
                .context("Failed to disconnect from SpacetimeDB")?;
        }

        Ok(())
    }

    // Connection lifecycle callbacks
    fn on_connected(_conn: &DbConnection, identity: spacetimedb_sdk::Identity, token: &str) {
        log::info!(
            "[SpacetimeDB] Connected successfully (identity: {}, token: {})",
            identity,
            token
        );
    }

    fn on_connect_error(_ctx: &generated::ErrorContext, err: spacetimedb_sdk::Error) {
        log::error!("[SpacetimeDB] Connection error: {}", err);
    }

    fn on_disconnected(_ctx: &generated::ErrorContext, err: Option<spacetimedb_sdk::Error>) {
        if let Some(err) = err {
            log::warn!("[SpacetimeDB] Disconnected with error: {}", err);
        } else {
            log::info!("[SpacetimeDB] Disconnected");
        }
    }
}

impl Default for MultiplayerClient {
    fn default() -> Self {
        Self::new()
    }
}
