//! Main window content - sidebar, content area, etc.

use adw::prelude::*;
use adw::StatusPage;
use adw::gtk::{self, Orientation, Stack};

use slsk_core::{CoreHandle, Event};

#[derive(Clone)]
pub struct MainWindow {
    widget: gtk::Box,
    core_handle: std::sync::Arc<std::sync::Mutex<Option<CoreHandle>>>,
}

impl MainWindow {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        let stack = Stack::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        // Placeholder until connected
        let placeholder = StatusPage::builder()
            .title("Connecting...")
            .description("Please wait")
            .build();
        stack.add_named(&placeholder, Some("loading"));
        stack.set_visible_child_name("loading");

        let sidebar = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(250)
            .build();

        let content = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .vexpand(true)
            .build();

        content.append(&sidebar);
        content.append(&stack);

        widget.append(&content);

        Self {
            widget,
            core_handle: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn set_core_handle(&self, handle: CoreHandle) {
        *self.core_handle.lock().unwrap() = Some(handle);
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }

    pub fn handle_event(&self, evt: &Event) {
        match evt {
            Event::Connected { motd } => {
                tracing::info!("Connected! MOTD length: {}", motd.len());
            }
            Event::LoginFailed { reason } => {
                tracing::warn!("Login failed: {}", reason);
            }
            Event::Disconnected { reason } => {
                tracing::info!("Disconnected: {:?}", reason);
            }
            Event::RoomJoined { room } => {
                tracing::info!("Joined room: {}", room);
            }
            Event::RoomLeft { room } => {
                tracing::info!("Left room: {}", room);
            }
            Event::RoomMessage { room, username, message } => {
                tracing::debug!("[{}] {}: {}", room, username, message);
            }
            Event::PrivateMessage { username, message, .. } => {
                tracing::debug!("PM from {}: {}", username, message);
            }
            Event::UserStatusChanged { username, status } => {
                tracing::debug!("User {} status: {}", username, status);
            }
            Event::SearchResult { token, username } => {
                tracing::debug!("Search result token={} from {}", token, username);
            }
            Event::TransferProgress { id, bytes_done, total, direction: _ } => {
                tracing::trace!("Transfer {}: {}/{}", id, bytes_done, total);
            }
            Event::TransferRequest { username, filename, size, token: _ } => {
                tracing::info!("Transfer request from {}: {} ({} bytes)", username, filename, size);
            }
            Event::TransferComplete { id, success } => {
                tracing::info!("Transfer {} complete: {}", id, success);
            }
            Event::UploadFailed { username, filename } => {
                tracing::warn!("Upload failed: {} to {}", filename, username);
            }
            Event::UploadDenied { username, filename, reason } => {
                tracing::warn!("Upload denied: {} to {} - {}", filename, username, reason);
            }
            Event::PlaceInQueue { username, filename, position } => {
                tracing::debug!("Place in queue: {} at {} for {}", filename, position, username);
            }
        }
    }

    pub fn poll_events(&self) {
        let mut guard = match self.core_handle.lock() {
            Ok(g) => g,
            Err(e) => {
                // Poisoned - log and skip this poll cycle
                tracing::warn!("core_handle mutex poisoned, skipping poll: {}", e);
                return;
            }
        };
        if let Some(ref mut handle) = *guard {
            let mut polled = 0;
            while let Some(evt) = handle.poll_event() {
                self.handle_event(&evt);
                polled += 1;
                // If we're getting flooded with events, yield to GTK periodically
                if polled % 64 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            // No more events - small sleep to avoid spinning the GTK main loop
            if polled == 0 {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    }
}

impl Default for MainWindow {
    fn default() -> Self {
        Self::new()
    }
}
