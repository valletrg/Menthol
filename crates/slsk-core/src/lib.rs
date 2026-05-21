#![allow(dead_code)]
pub mod error;
pub mod config;
pub mod event;
pub mod command;
pub mod network;

pub use config::Config;
pub use event::Event;
pub use command::Command;
pub use network::state::{ConnectionState, DisconnectReason, ServerStats};

use tokio::sync::mpsc;

/// Starts the core daemon, returning a handle to interact with it.
/// The internal tokio runtime runs in a background thread.
pub fn start(config: Config) -> CoreHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel(256);
    let (evt_tx, evt_rx) = mpsc::channel(256);

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            run_core(config, cmd_rx, evt_tx).await;
        });
    });

    CoreHandle { cmd_tx, evt_rx }
}

pub struct CoreHandle {
    cmd_tx: mpsc::Sender<Command>,
    evt_rx: mpsc::Receiver<Event>,
}

impl CoreHandle {
    pub fn send(&self, cmd: Command) {
        let _ = self.cmd_tx.blocking_send(cmd);
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        self.evt_rx.blocking_recv()
    }
}

/// The core async entry point
async fn run_core(
    config: Config,
    mut cmd_rx: mpsc::Receiver<Command>,
    evt_tx: mpsc::Sender<Event>,
) {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tracing::info!("slsk-core starting with config: {:?}", config);

    // Simple command loop - handle connect/disconnect/search commands
    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            Command::Connect { username, password } => {
                tracing::info!("connect requested with username: {}", username);
                // TODO: spawn server connection
                let _ = evt_tx.send(Event::LoginFailed { reason: "not implemented yet".into() }).await;
            }
            Command::Disconnect => {
                tracing::info!("disconnect requested");
            }
            Command::Search { query, token } => {
                tracing::debug!("search: token={} query={}", token, query);
            }
            Command::QueueDownload { username, filename, size } => {
                tracing::debug!("queue download: {} from {} ({} bytes)", filename, username, size);
            }
        }
    }

    tracing::info!("slsk-core shutting down");
}
