//! Main window content - sidebar navigation + content panels.
//!
//! Layout: vertical Box with body (horizontal sidebar + AdwViewStack content) + status bar.
//! Sidebar: gtk4::Box with clickable nav rows → switches AdwViewStack visible page.

use adw::prelude::*;
use adw::ViewStack;
use gtk4::prelude::*;
use gtk4::{self as gtk, Align, GestureClick, Image, Label, Orientation};

use slsk_core::{CoreHandle, Event};

use super::panels::{
    browse_panel::BrowsePanel, downloads_panel::DownloadsPanel, messages_panel::MessagesPanel,
    rooms_panel::RoomsPanel, search_panel::{SearchPanel, SearchResult},
    settings_panel::SettingsPanel, uploads_panel::UploadsPanel,
};

const PAGE_SEARCH: &str = "search";
const PAGE_DOWNLOADS: &str = "downloads";
const PAGE_UPLOADS: &str = "uploads";
const PAGE_BROWSE: &str = "browse";
const PAGE_ROOMS: &str = "rooms";
const PAGE_MESSAGES: &str = "messages";
const PAGE_SETTINGS: &str = "settings";

#[derive(Clone)]
pub struct MainWindow {
    pub container: gtk::Box,
    core_handle: std::sync::Arc<std::sync::Mutex<Option<CoreHandle>>>,
    content_stack: ViewStack,
    search_panel: SearchPanel,
}

impl MainWindow {
    pub fn new() -> Self {
        let container = gtk::Box::builder().orientation(Orientation::Vertical).build();

        // ── Body: sidebar + content stack ────────────────────
        let body = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .hexpand(true)
            .vexpand(true)
            .build();

        // ── Sidebar ─────────────────────────────────────────
        let sidebar = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(220)
            .build();

        let sidebar_header = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(8)
            .build();
        let title_label = Label::new(Some("Menthol"));
        title_label.add_css_class("title-2");
        sidebar_header.append(&title_label);
        sidebar.append(&sidebar_header);

        let sep = gtk::Separator::builder()
            .orientation(Orientation::Horizontal)
            .build();
        sidebar.append(&sep);

        // ── Content ViewStack ───────────────────────────────
        let content_stack = ViewStack::builder().vexpand(true).hexpand(true).build();

        let search = SearchPanel::new();
        let downloads = DownloadsPanel::new();
        let uploads = UploadsPanel::new();
        let browse = BrowsePanel::new();
        let rooms = RoomsPanel::new();
        let messages = MessagesPanel::new();
        let settings = SettingsPanel::new();

        let panels: [(&str, &str, &gtk::Box); 7] = [
            (PAGE_SEARCH, "Search", &search.container),
            (PAGE_DOWNLOADS, "Downloads", &downloads.container),
            (PAGE_UPLOADS, "Uploads", &uploads.container),
            (PAGE_BROWSE, "Browse", &browse.container),
            (PAGE_ROOMS, "Rooms", &rooms.container),
            (PAGE_MESSAGES, "Messages", &messages.container),
            (PAGE_SETTINGS, "Settings", &settings.container),
        ];

        for &(page_name, title, widget) in &panels {
            let panel_box = gtk::Box::builder()
                .orientation(Orientation::Vertical)
                .vexpand(true)
                .build();
            let title_lbl = Label::new(Some(title));
            title_lbl.add_css_class("title-2");
            title_lbl.set_margin_start(20);
            title_lbl.set_margin_top(16);
            title_lbl.set_margin_bottom(8);
            panel_box.append(&title_lbl);
            panel_box.append(widget);
            content_stack.add_named(&panel_box, Some(page_name));
        }

        // Wire sidebar nav: click row → switch content stack page
        let nav_items: [(&str, &str, &str); 7] = [
            (PAGE_SEARCH, "Search", "system-search"),
            (PAGE_DOWNLOADS, "Downloads", "go-down"),
            (PAGE_UPLOADS, "Uploads", "go-up"),
            (PAGE_BROWSE, "Browse", "drive-harddisk"),
            (PAGE_ROOMS, "Rooms", "x-office-chat"),
            (PAGE_MESSAGES, "Messages", "mail-unread"),
            (PAGE_SETTINGS, "Settings", "preferences-system"),
        ];

        let content_stack_for_nav = content_stack.clone();
        for &(name, label, icon) in &nav_items {
            let row = gtk::Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(12)
                .margin_start(12)
                .margin_end(12)
                .margin_top(8)
                .margin_bottom(8)
                .build();
            let img = Image::from_icon_name(icon);
            let lbl = Label::new(Some(label));
            lbl.set_halign(Align::Start);
            lbl.set_hexpand(true);
            row.append(&img);
            row.append(&lbl);

            let stack = content_stack_for_nav.clone();
            let page_name = name.to_string();
            let click = GestureClick::new();
            let page_name_clone = page_name.clone();
            let stack_clone = stack.clone();
            click.connect_pressed(move |_gesture, _n_press, _x, _y| {
                stack_clone.set_visible_child_name(&page_name_clone);
            });
            row.add_controller(click);

            sidebar.append(&row);
        }

        body.append(&sidebar);
        body.append(&content_stack);

        // ── Status bar ─────────────────────────────────────
        let status_bar = Self::build_status_bar();

        container.append(&body);
        container.append(&status_bar);

        content_stack.set_visible_child_name(PAGE_SEARCH);

        Self {
            container,
            core_handle: std::sync::Arc::new(std::sync::Mutex::new(None)),
            content_stack,
            search_panel: search,
        }
    }

    fn build_status_bar() -> gtk::Box {
        let bar = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let status_label = Label::new(Some("Connected"));
        status_label.add_css_class("dim-label");

        let separator = gtk::Separator::builder()
            .orientation(Orientation::Vertical)
            .build();

        bar.append(&status_label);
        bar.append(&separator);

        bar
    }

    pub fn set_core_handle(&self, handle: CoreHandle) {
        // Wire SearchPanel callbacks to CoreHandle
        let core = self.core_handle.clone();

        let search_fn = Box::new(move |query: String| {
            let guard = match core.lock() {
                Ok(g) => g,
                Err(_) => return 0,
            };
            if let Some(ref h) = *guard {
                return h.search(query);
            }
            0
        }) as Box<dyn Fn(String) -> u32>;

        let core2 = self.core_handle.clone();
        let queue_fn = Box::new(move |username: String, filename: String, size: u64| {
            let guard = match core2.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            if let Some(ref h) = *guard {
                h.queue_download(username, filename, size);
            }
        }) as Box<dyn Fn(String, String, u64)>;

        self.search_panel.on_search(search_fn);
        self.search_panel.on_queue_download(queue_fn);
        self.search_panel.connect();

        *self.core_handle.lock().unwrap() = Some(handle);
    }

    pub fn container(&self) -> gtk::Box {
        self.container.clone()
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
            Event::SearchStarted { token, query: q } => {
                tracing::info!("Search started: token={} query={}", token, q);
                self.search_panel.on_search_started(*token, &q);
            }
            Event::SearchResult { token, username, filename, size } => {
                tracing::debug!("Search result token={} from {}: {}", token, username, filename);
                self.search_panel.add_result(SearchResult {
                    token: *token,
                    username: (*username).clone(),
                    filename: (*filename).clone(),
                    size: *size,
                });
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
                tracing::warn!("core_handle mutex poisoned, skipping poll: {}", e);
                return;
            }
        };
        if let Some(ref mut handle) = *guard {
            let mut polled = 0;
            while let Some(evt) = handle.poll_event() {
                self.handle_event(&evt);
                polled += 1;
                if polled % 64 == 0 {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
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
