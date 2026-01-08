//! TCP remote control server for live game control
//!
//! Allows sending commands to a running game instance via TCP socket.
//!
//! ## Usage
//!
//! Start game with remote control:
//! ```bash
//! cargo run --release -- --remote-control
//! ```
//!
//! Send commands via netcat:
//! ```bash
//! echo '(type: "MineCircle", center_x: 0, center_y: 50, radius: 10)' | nc localhost 7453
//! ```

use crate::scenario::ScenarioAction;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

const TCP_PORT: u16 = 7453;

/// Command sent from TCP client to game
#[derive(Debug, Clone)]
pub struct RemoteCommand {
    pub action: ScenarioAction,
}

/// Response sent from game to TCP client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteResponse {
    pub success: bool,
    pub message: String,
}

/// Start TCP server in background thread
/// Returns (command_receiver, response_sender) for main game loop
pub fn start_server() -> Result<(Receiver<RemoteCommand>, Sender<RemoteResponse>)> {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (resp_tx, resp_rx) = mpsc::channel();

    thread::spawn(move || {
        if let Err(e) = run_tcp_server(cmd_tx, resp_rx) {
            log::error!("TCP server error: {}", e);
        }
    });

    log::info!("TCP remote control server started on port {}", TCP_PORT);
    log::info!(
        "Send commands via: echo '(type: \"TeleportPlayer\", x: 0.0, y: 100.0)' | nc localhost {}",
        TCP_PORT
    );

    Ok((cmd_rx, resp_tx))
}

/// Run TCP server loop (runs in background thread)
fn run_tcp_server(cmd_tx: Sender<RemoteCommand>, resp_rx: Receiver<RemoteResponse>) -> Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", TCP_PORT))
        .context("Failed to bind TCP listener")?;

    log::info!(
        "Listening for remote control connections on 127.0.0.1:{}",
        TCP_PORT
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_client(stream, &cmd_tx, &resp_rx) {
                    log::warn!("Client handler error: {}", e);
                }
            }
            Err(e) => {
                log::warn!("Connection error: {}", e);
            }
        }
    }

    Ok(())
}

/// Handle single TCP client connection
fn handle_client(
    mut stream: TcpStream,
    cmd_tx: &Sender<RemoteCommand>,
    resp_rx: &Receiver<RemoteResponse>,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    log::debug!("Client connected: {}", peer_addr);

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // Read command (RON format, newline-terminated)
    reader.read_line(&mut line)?;
    let line = line.trim();

    if line.is_empty() {
        return Ok(());
    }

    log::debug!("Received command: {}", line);

    // Parse RON command
    let action: ScenarioAction = match ron::from_str(line) {
        Ok(action) => action,
        Err(e) => {
            let error_resp = RemoteResponse {
                success: false,
                message: format!("Failed to parse RON: {}", e),
            };
            let json = serde_json::to_string(&error_resp)?;
            writeln!(stream, "{}", json)?;
            return Ok(());
        }
    };

    // Send command to game loop
    cmd_tx.send(RemoteCommand { action })?;

    // Wait for response from game loop (with timeout)
    match resp_rx.recv_timeout(std::time::Duration::from_secs(5)) {
        Ok(response) => {
            let json = serde_json::to_string(&response)?;
            writeln!(stream, "{}", json)?;
            log::debug!("Sent response: {}", json);
        }
        Err(_) => {
            let error_resp = RemoteResponse {
                success: false,
                message: "Timeout waiting for game response".to_string(),
            };
            let json = serde_json::to_string(&error_resp)?;
            writeln!(stream, "{}", json)?;
        }
    }

    Ok(())
}
