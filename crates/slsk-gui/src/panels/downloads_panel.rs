use glib::Value;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CellRendererProgress, Label, ListStore, Orientation,
    ScrolledWindow, Separator, SpinButton, TreeIter, TreeModel, TreeView, TreeViewColumn,
};
use std::cell::RefCell;
use std::rc::Rc;

/// A download item
#[derive(Debug, Clone)]
pub struct DownloadItem {
    pub id: u64,
    pub filename: String,
    pub username: String,
    pub size: u64,
    pub downloaded: u64,
    pub speed_bps: u64,
    pub state: String,
}

/// GTK4 Downloads panel widget
pub struct DownloadsPanel {
    pub container: GtkBox,
    model: Rc<RefCell<ListStore>>,
}

impl DownloadsPanel {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .build();

        // ── Toolbar ──────────────────────────────────────
        let toolbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let pause_all_btn = Button::builder()
            .icon_name("media-pause")
            .tooltip_text("Pause All")
            .build();
        let resume_all_btn = Button::builder()
            .icon_name("media-play")
            .tooltip_text("Resume All")
            .build();
        let cancel_all_btn = Button::builder()
            .icon_name("process-stop")
            .tooltip_text("Clear All")
            .build();

        let slots_label = Label::new(Some("Slots:"));
        let slots_adj = gtk4::Adjustment::builder()
            .value(3.0)
            .lower(1.0)
            .upper(10.0)
            .step_increment(1.0)
            .build();
        let slots_spin = SpinButton::builder()
            .adjustment(&slots_adj)
            .digits(0)
            .build();

        toolbar.append(&pause_all_btn);
        toolbar.append(&resume_all_btn);
        toolbar.append(&cancel_all_btn);
        toolbar.append(&Separator::new(Orientation::Horizontal));
        toolbar.append(&slots_label);
        toolbar.append(&slots_spin);
        toolbar.set_margin_start(8);
        toolbar.set_margin_end(8);
        toolbar.set_margin_top(8);

        // ── TreeView ────────────────────────────────────
        let model = ListStore::new(&[
            String::static_type(), // 0: filename
            String::static_type(), // 1: username
            String::static_type(), // 2: total size
            String::static_type(), // 3: progress (string "XX%")
            i32::static_type(),    // 4: progress_value (0-100 for renderer)
            String::static_type(), // 5: speed
            String::static_type(), // 6: state
            u64::static_type(),    // 7: id
        ]);
        let model = Rc::new(RefCell::new(model));

        let treeview = TreeView::builder()
            .model(&*model.borrow())
            .vexpand(true)
            .build();

        // File column
        let col = TreeViewColumn::new();
        col.set_title("File");
        col.set_resizable(true);
        col.pack_start(&gtk4::CellRendererText::new(), true);
        col.add_attribute(&gtk4::CellRendererText::new(), "text", 0);
        treeview.append_column(&col);

        // User column
        let col = TreeViewColumn::new();
        col.set_title("User");
        col.pack_start(&gtk4::CellRendererText::new(), true);
        col.add_attribute(&gtk4::CellRendererText::new(), "text", 1);
        treeview.append_column(&col);

        // Progress column
        let col = TreeViewColumn::new();
        col.set_title("Progress");
        let prog = CellRendererProgress::new();
        col.pack_start(&prog, true);
        col.add_attribute(&prog, "value", 4);
        treeview.append_column(&col);

        // Size column
        let col = TreeViewColumn::new();
        col.set_title("Size");
        col.pack_start(&gtk4::CellRendererText::new(), true);
        col.add_attribute(&gtk4::CellRendererText::new(), "text", 2);
        treeview.append_column(&col);

        // Speed column
        let col = TreeViewColumn::new();
        col.set_title("Speed");
        col.pack_start(&gtk4::CellRendererText::new(), true);
        col.add_attribute(&gtk4::CellRendererText::new(), "text", 5);
        treeview.append_column(&col);

        // State column
        let col = TreeViewColumn::new();
        col.set_title("State");
        col.pack_start(&gtk4::CellRendererText::new(), true);
        col.add_attribute(&gtk4::CellRendererText::new(), "text", 6);
        treeview.append_column(&col);

        let scroll = ScrolledWindow::builder()
            .vexpand(true)
            .child(&treeview)
            .build();

        // Status bar
        let statusbar = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let status_label = Label::new(Some("0 downloads"));
        status_label.add_css_class("dim-label");
        status_label.set_margin_start(8);
        status_label.set_margin_bottom(8);
        statusbar.append(&status_label);

        container.append(&toolbar);
        container.append(&scroll);
        container.append(&statusbar);

        Self { container, model }
    }

    pub fn add_download(&self, id: u64, filename: &str, username: &str, size: u64) {
        let model = self.model.borrow();
        let iter = model.append();
        model.set_value(&iter, 0, &filename.to_value());
        model.set_value(&iter, 1, &username.to_value());
        model.set_value(&iter, 2, &Self::format_size(size).to_value());
        model.set_value(&iter, 3, &"0%".to_value());
        model.set_value(&iter, 4, &(0i32).to_value());
        model.set_value(&iter, 5, &"".to_value());
        model.set_value(&iter, 6, &"queued".to_value());
        model.set_value(&iter, 7, &id.to_value());
    }

    pub fn update_progress(
        &self,
        id: u64,
        downloaded: u64,
        total: u64,
        speed_bps: u64,
        state: &str,
    ) {
        let model = self.model.borrow();
        if let Some(iter) = Self::find_iter(&model, 7, id) {
            let pct = if total > 0 {
                (downloaded as f64 / total as f64 * 100.0) as i32
            } else {
                0
            };
            model.set_value(&iter, 3, &format!("{}%", pct).to_value());
            model.set_value(&iter, 4, &pct.to_value());
            model.set_value(&iter, 5, &Self::format_speed(speed_bps).to_value());
            model.set_value(&iter, 6, &state.to_value());
        }
    }

    fn find_iter(model: &ListStore, column: i32, needle: u64) -> Option<TreeIter> {
        let mut iter = model.iter_first()?;
        loop {
            let val: u64 = model.get_value(&iter, column).get().unwrap_or(0);
            if val == needle {
                return Some(iter.clone());
            }
            if !model.iter_next(&iter) {
                break;
            }
        }
        None
    }

    pub fn remove_download(&self, id: u64) {
        let model = self.model.borrow();
        if let Some(iter) = Self::find_iter(&model, 7, id) {
            model.remove(&iter);
        }
    }

    fn format_size(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.2} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    fn format_speed(bps: u64) -> String {
        if bps >= 1_000_000 {
            format!("{:.1} MB/s", bps as f64 / 1_000_000.0)
        } else if bps >= 1000 {
            format!("{:.1} KB/s", bps as f64 / 1000.0)
        } else {
            format!("{} B/s", bps)
        }
    }
}

impl Default for DownloadsPanel {
    fn default() -> Self {
        Self::new()
    }
}
