//! Settings panel - user configuration UI per Phase 7 of GUI_IMPLEMENTATION_PLAN.md
//!
//! Layout uses AdwPreferencesPage with groups for Connection, Downloads, Uploads, Interface.

use adw::prelude::*;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Entry, Orientation, SpinButton};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Settings values mirroring slsk_core::Config
#[derive(Debug, Clone)]
pub struct Settings {
    pub username: String,
    pub password: String,
    pub server_host: String,
    pub server_port: u32,
    pub listen_port: u32,
    pub download_dir: String,
    pub upload_slots: u8,
    pub download_slots: u8,
    pub dark_mode: bool,
    pub auto_reconnect: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            server_host: "server.slsknet.org".into(),
            server_port: 2242,
            listen_port: 2234,
            download_dir: std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| ".".into()),
            upload_slots: 3,
            download_slots: 3,
            dark_mode: false,
            auto_reconnect: true,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(contents) = std::fs::read_to_string(&path) {
            if let Ok(toml) = toml::from_str::<SettingsToml>(&contents) {
                return toml.into();
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = Path::new(&path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let toml = SettingsToml::from(self.clone());
        let contents = toml::to_string_pretty(&toml).map_err(|e| e.to_string())?;
        std::fs::write(&path, contents).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn config_path() -> std::path::PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("menthol");
        path.push("config.toml");
        path
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SettingsToml {
    username: String,
    password: String,
    server_host: String,
    server_port: u32,
    listen_port: u32,
    download_dir: String,
    upload_slots: u8,
    download_slots: u8,
    dark_mode: bool,
    auto_reconnect: bool,
}

impl From<SettingsToml> for Settings {
    fn from(t: SettingsToml) -> Self {
        Self {
            username: t.username,
            password: t.password,
            server_host: t.server_host,
            server_port: t.server_port,
            listen_port: t.listen_port,
            download_dir: t.download_dir,
            upload_slots: t.upload_slots,
            download_slots: t.download_slots,
            dark_mode: t.dark_mode,
            auto_reconnect: t.auto_reconnect,
        }
    }
}

impl From<Settings> for SettingsToml {
    fn from(s: Settings) -> Self {
        Self {
            username: s.username,
            password: s.password,
            server_host: s.server_host,
            server_port: s.server_port,
            listen_port: s.listen_port,
            download_dir: s.download_dir,
            upload_slots: s.upload_slots,
            download_slots: s.download_slots,
            dark_mode: s.dark_mode,
            auto_reconnect: s.auto_reconnect,
        }
    }
}

/// Settings panel using AdwPreferencesPage
pub struct SettingsPanel {
    pub container: GtkBox,
    settings: Rc<RefCell<Settings>>,
}

impl SettingsPanel {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .build();

        let settings = Rc::new(RefCell::new(Settings::load()));

        let scroll = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let page = adw::PreferencesPage::builder()
            .margin_start(20)
            .margin_end(20)
            .margin_top(20)
            .margin_bottom(20)
            .build();

        let s = settings.borrow().clone();

        // ── Connection group ───────────────────────────────
        let conn_group = adw::PreferencesGroup::builder().title("Connection").build();

        let server_entry = Entry::builder().text(&s.server_host).build();
        let server_row = adw::EntryRow::builder().title("Server").build();
        server_row.add_suffix(&server_entry);
        conn_group.add(&server_row);

        let port_adj = gtk4::Adjustment::builder()
            .value(s.server_port as f64)
            .lower(1.0)
            .upper(65535.0)
            .step_increment(1.0)
            .build();
        let port_spin = SpinButton::builder()
            .adjustment(&port_adj)
            .digits(0)
            .build();
        let port_row = adw::ActionRow::builder().title("Server Port").build();
        port_row.add_suffix(&port_spin);
        conn_group.add(&port_row);

        let listen_adj = gtk4::Adjustment::builder()
            .value(s.listen_port as f64)
            .lower(1024.0)
            .upper(65535.0)
            .step_increment(1.0)
            .build();
        let listen_spin = SpinButton::builder()
            .adjustment(&listen_adj)
            .digits(0)
            .build();
        let listen_row = adw::ActionRow::builder().title("Listen Port").build();
        listen_row.add_suffix(&listen_spin);
        conn_group.add(&listen_row);

        let reconnect_row = adw::SwitchRow::builder()
            .title("Auto-reconnect")
            .subtitle("Automatically reconnect on disconnect")
            .build();
        reconnect_row.set_active(s.auto_reconnect);
        conn_group.add(&reconnect_row);

        // ── Downloads group ───────────────────────────────
        let dl_group = adw::PreferencesGroup::builder().title("Downloads").build();

        let dl_dir_entry = Entry::builder()
            .text(&s.download_dir)
            .placeholder_text("/path/to/downloads")
            .build();
        let dl_dir_row = adw::ActionRow::builder()
            .title("Download Directory")
            .build();
        dl_dir_row.set_activatable_widget(Some(&dl_dir_entry));
        dl_group.add(&dl_dir_row);

        let dl_slots_adj = gtk4::Adjustment::builder()
            .value(s.download_slots as f64)
            .lower(1.0)
            .upper(10.0)
            .step_increment(1.0)
            .build();
        let dl_slots_spin = SpinButton::builder()
            .adjustment(&dl_slots_adj)
            .digits(0)
            .build();
        let dl_slots_row = adw::ActionRow::builder().title("Download Slots").build();
        dl_slots_row.add_suffix(&dl_slots_spin);
        dl_group.add(&dl_slots_row);

        // ── Uploads group ─────────────────────────────────
        let ul_group = adw::PreferencesGroup::builder().title("Uploads").build();

        let ul_slots_adj = gtk4::Adjustment::builder()
            .value(s.upload_slots as f64)
            .lower(1.0)
            .upper(10.0)
            .step_increment(1.0)
            .build();
        let ul_slots_spin = SpinButton::builder()
            .adjustment(&ul_slots_adj)
            .digits(0)
            .build();
        let ul_slots_row = adw::ActionRow::builder().title("Upload Slots").build();
        ul_slots_row.add_suffix(&ul_slots_spin);
        ul_group.add(&ul_slots_row);

        // ── Interface group ──────────────────────────────
        let iface_group = adw::PreferencesGroup::builder().title("Interface").build();

        let dark_row = adw::SwitchRow::builder()
            .title("Dark Mode")
            .subtitle("Use dark color scheme")
            .build();
        dark_row.set_active(s.dark_mode);
        iface_group.add(&dark_row);

        let save_btn = Button::builder()
            .label("Save Settings")
            .halign(Align::Center)
            .width_request(120)
            .build();
        save_btn.add_css_class("suggested-action");
        iface_group.add(&save_btn);

        // ── Assemble page ────────────────────────────────
        page.add(&conn_group);
        page.add(&dl_group);
        page.add(&ul_group);
        page.add(&iface_group);
        scroll.set_child(Some(&page));
        container.append(&scroll);

        // Save
        let settings_cb = settings.clone();
        let server_row_cb = server_row.clone();
        let port_spin_cb = port_spin.clone();
        let listen_spin_cb = listen_spin.clone();
        let reconnect_row_cb = reconnect_row.clone();
        let dl_dir_entry_cb = dl_dir_entry.clone();
        let dl_slots_spin_cb = dl_slots_spin.clone();
        let ul_slots_spin_cb = ul_slots_spin.clone();
        let dark_row_cb = dark_row.clone();

        save_btn.connect_clicked(move |_| {
            let mut s = settings_cb.borrow_mut();
            s.server_host = server_row_cb.text().to_string();
            s.server_port = port_spin_cb.value() as u32;
            s.listen_port = listen_spin_cb.value() as u32;
            s.auto_reconnect = reconnect_row_cb.is_active();
            s.dark_mode = dark_row_cb.is_active();
            s.upload_slots = ul_slots_spin_cb.value() as u8;
            s.download_slots = dl_slots_spin_cb.value() as u8;
            s.download_dir = dl_dir_entry_cb.text().to_string();
            drop(s);

            if let Err(e) = settings_cb.borrow().save() {
                tracing::error!("failed to save settings: {}", e);
            } else {
                tracing::info!("settings saved");
            }
        });

        Self {
            container,
            settings,
        }
    }
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self::new()
    }
}
