//! Search panel - file search with results list.
//!
//! Protocol: FileSearchRequest (outgoing) → Event::SearchResult (incoming).
//!
//! Flow: user enters query → `on_search` callback fires → CoreHandle::search()
//!       → Event::SearchResult arrives → MainWindow calls `add_result()` → appended to list.
//!       Click Download → `on_queue_download` callback → CoreHandle::queue_download().

use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, Entry, Image, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow,
};
use std::cell::RefCell;
use std::rc::Rc;

/// A single search result row.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub token: u32,
    pub username: String,
    pub filename: String,
    pub size: u64,
}

impl SearchResult {
    fn format_size(&self) -> String {
        if self.size < 1024 {
            format!("{} B", self.size)
        } else if self.size < 1024 * 1024 {
            format!("{:.1} KB", self.size as f64 / 1024.0)
        } else if self.size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

#[derive(Clone)]
pub struct SearchPanel {
    pub container: GtkBox,
    search_entry: Entry,
    search_btn: Button,
    results_list: ListBox,
    status_label: Label,
    results_model: Rc<RefCell<Vec<SearchResult>>>,
    current_token: Rc<RefCell<u32>>,
    on_do_search: Rc<RefCell<Option<Box<dyn Fn(String) -> u32>>>>,
    on_queue: Rc<RefCell<Option<Box<dyn Fn(String, String, u64)>>>>,
}

impl SearchPanel {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .build();

        // ── Search bar ────────────────────────────────────
        let search_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(16)
            .margin_bottom(8)
            .build();

        let search_entry = Entry::builder()
            .placeholder_text("Search for files...")
            .hexpand(true)
            .build();
        let search_btn = Button::builder().label("Search").build();
        search_btn.add_css_class("suggested-action");

        search_bar.append(&search_entry);
        search_bar.append(&search_btn);

        // ── Results list ──────────────────────────────────
        let results_list = ListBox::builder().vexpand(true).build();
        results_list.add_css_class("boxed-list");

        let scrolled = ScrolledWindow::builder()
            .vexpand(true)
            .child(&results_list)
            .build();

        // ── Status bar ────────────────────────────────────
        let status_bar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .margin_start(16)
            .margin_end(16)
            .margin_top(8)
            .margin_bottom(8)
            .build();

        let status_label = Label::new(Some("Enter a query to search"));
        status_label.add_css_class("dim-label");
        status_bar.append(&status_label);

        container.append(&search_bar);
        container.append(&scrolled);
        container.append(&status_bar);

        Self {
            container,
            search_entry,
            search_btn,
            results_list,
            status_label,
            results_model: Rc::new(RefCell::new(Vec::new())),
            current_token: Rc::new(RefCell::new(0)),
            on_do_search: Rc::new(RefCell::new(None)),
            on_queue: Rc::new(RefCell::new(None)),
        }
    }

    /// Register the search callback: called with the query string, returns token.
    pub fn on_search(&self, cb: Box<dyn Fn(String) -> u32>) {
        *self.on_do_search.borrow_mut() = Some(cb);
    }

    /// Register the queue callback: called with (username, filename, size).
    pub fn on_queue_download(&self, cb: Box<dyn Fn(String, String, u64)>) {
        *self.on_queue.borrow_mut() = Some(cb);
    }

    /// Called when a search starts. Clears previous results.
    pub fn on_search_started(&self, token: u32, query: &str) {
        *self.current_token.borrow_mut() = token;
        self.results_model.borrow_mut().clear();
        while let Some(row) = self.results_list.row_at_index(0) {
            self.results_list.remove(&row);
        }
        self.status_label
            .set_text(&format!("Searching for \"{}\"...", query));
    }

    /// Called by MainWindow to add an incoming search result.
    pub fn add_result(&self, result: SearchResult) {
        // Ignore results for a different token (stale)
        if result.token != *self.current_token.borrow() {
            return;
        }

        // Deduplicate by (username, filename)
        {
            let mut model = self.results_model.borrow_mut();
            if model
                .iter()
                .any(|r| r.username == result.username && r.filename == result.filename)
            {
                return;
            }
            model.push(result.clone());
        }

        let row = Self::build_row(&result, self.on_queue.clone());
        self.results_list.append(&row);

        let count = self.results_model.borrow().len();
        self.status_label
            .set_text(&format!("{} result{}", count, if count == 1 { "" } else { "s" }));
    }

    /// Wire up the search entry + button. Call after `on_search` has been registered.
    pub fn connect(&self) {
        // Button click
        let entry = self.search_entry.clone();
        let on_search = self.on_do_search.clone();
        self.search_btn.clone().connect_clicked(move |_| {
            let query = entry.text().trim().to_string();
            if query.is_empty() {
                return;
            }
            if let Some(ref cb) = *on_search.borrow() {
                cb(query);
            }
        });

        // Enter key
        let entry2 = self.search_entry.clone();
        let on_search2 = self.on_do_search.clone();
        self.search_entry.clone().connect_activate(move |_| {
            let query = entry2.text().trim().to_string();
            if query.is_empty() {
                return;
            }
            if let Some(ref cb) = *on_search2.borrow() {
                cb(query);
            }
        });
    }

    fn build_row(
        result: &SearchResult,
        on_queue: Rc<RefCell<Option<Box<dyn Fn(String, String, u64)>>>>,
    ) -> ListBoxRow {
        let row = ListBoxRow::builder().build();
        let box_ = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // File type icon based on extension
        let icon_name = if result.filename.to_lowercase().ends_with(".mp3")
            || result.filename.to_lowercase().ends_with(".flac")
            || result.filename.to_lowercase().ends_with(".wav")
            || result.filename.to_lowercase().ends_with(".ogg")
            || result.filename.to_lowercase().ends_with(".m4a")
        {
            "audio-x-generic"
        } else if result.filename.to_lowercase().ends_with(".avi")
            || result.filename.to_lowercase().ends_with(".mkv")
            || result.filename.to_lowercase().ends_with(".mp4")
            || result.filename.to_lowercase().ends_with(".mov")
        {
            "video-x-generic"
        } else if result.filename.to_lowercase().ends_with(".jpg")
            || result.filename.to_lowercase().ends_with(".png")
            || result.filename.to_lowercase().ends_with(".gif")
        {
            "image-x-generic"
        } else {
            "text-x-generic"
        };
        let icon = Image::from_icon_name(icon_name);
        box_.append(&icon);

        // File info: name + meta
        let info = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let name_lbl = Label::new(Some(&result.filename));
        name_lbl.set_halign(Align::Start);
        name_lbl.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        name_lbl.add_css_class("title-5");

        let meta = format!("{} · {}", result.username, result.format_size());
        let meta_lbl = Label::new(Some(&meta));
        meta_lbl.set_halign(Align::Start);
        meta_lbl.add_css_class("dim-label");

        info.append(&name_lbl);
        info.append(&meta_lbl);
        box_.append(&info);

        let dl_btn = Button::builder()
            .label("Download")
            .halign(Align::End)
            .build();
        dl_btn.add_css_class("suggested-action");
        box_.append(&dl_btn);

        let user = result.username.clone();
        let fname = result.filename.clone();
        let size = result.size;
        let cb = on_queue.clone();
        let dl_btn_cb = dl_btn.clone();
        dl_btn.connect_clicked(move |_| {
            if let Some(ref q) = *cb.borrow() {
                q(user.clone(), fname.clone(), size);
                dl_btn_cb.set_label("Queued");
                dl_btn_cb.set_sensitive(false);
                dl_btn_cb.remove_css_class("suggested-action");
            }
        });

        box_.append(&dl_btn);
        row.set_child(Some(&box_));
        row
    }
}

impl Default for SearchPanel {
    fn default() -> Self {
        Self::new()
    }
}
