#![allow(dead_code)]
pub mod command;
pub mod config;
pub mod error;
pub mod event;
pub mod io;
pub mod network;
pub mod transfer;

pub use command::Command;
pub use config::Config;
pub use event::Event;
pub use network::server::ConnectionHandle;
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
        self.evt_rx.try_recv().ok()
    }
}

/// The core async entry point
async fn run_core(config: Config, cmd_rx: mpsc::Receiver<Command>, evt_tx: mpsc::Sender<Event>) {
    // Note: tracing is initialized by the GUI in main(), not here.
    // Multiple calls to init() would panic.

    tracing::info!("slsk-core starting with config: {:?}", config);

    // Connect to server. We pass cmd_rx and our evt_tx so events go directly
    // to the CoreHandle's external channel.
    match crate::network::server::connect(&config, cmd_rx, evt_tx).await {
        Ok(_) => {
            // Connection is running. Wait forever - the spawned task handles everything.
            // The GUI reads events from CoreHandle.evt_rx which we wired up.
            std::future::pending::<()>().await;
        }
        Err(e) => {
            tracing::error!("failed to connect: {}", e);
            // Can't send event here - the other side already disconnected
        }
    }

    tracing::info!("slsk-core shutting down");
}
