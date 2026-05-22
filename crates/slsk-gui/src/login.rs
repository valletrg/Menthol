//! Login view widget - placed inside an existing window, doesn't create its own window.

use adw::prelude::*;
use adw::Clamp;
use adw::gtk::{self, Align, Button, Entry, Label, PasswordEntry, Spinner};
use gtk4::Orientation;

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    ResolvingHost,
    ConnectingToServer,
    ConnectionRefused,
    ConnectionFailed(String),
    LoginFailed(String),
    Connected,
}

/// A login widget that can be placed in an existing window.
/// Use `on_connect` to register a callback for when the user clicks Connect.
#[derive(Clone)]
pub struct LoginView {
    widget: gtk::Box,
    username_entry: Entry,
    password_entry: PasswordEntry,
    connect_btn: Button,
    status_label: Label,
    spinner: Spinner,
}

impl LoginView {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .build();

        let content = Clamp::builder().build();

        let vbox = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(32)
            .margin_bottom(32)
            .margin_start(32)
            .margin_end(32)
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        let title = Label::new(Some("Connect to Soulseek"));
        title.add_css_class("title-1");

        let username_label = Label::new(Some("Username"));
        username_label.set_halign(Align::Start);

        let username_entry = Entry::builder()
            .placeholder_text("Enter username")
            .build();

        let password_label = Label::new(Some("Password"));
        password_label.set_halign(Align::Start);
        password_label.set_margin_top(16);

        let password_entry = PasswordEntry::builder()
            .placeholder_text("Enter password")
            .build();

        let status_label = Label::new(None::<&str>);
        status_label.add_css_class("dim-label");
        status_label.set_halign(Align::Start);
        status_label.set_margin_top(8);

        let spinner = Spinner::builder()
            .halign(Align::Center)
            .build();
        spinner.set_visible(false);

        let connect_btn = Button::builder()
            .label("Connect")
            .halign(Align::End)
            .margin_top(24)
            .build();
        connect_btn.add_css_class("suggested-action");

        vbox.append(&title);
        vbox.append(&username_label);
        vbox.append(&username_entry);
        vbox.append(&password_label);
        vbox.append(&password_entry);
        vbox.append(&status_label);
        vbox.append(&spinner);
        vbox.append(&connect_btn);

        content.set_child(Some(&vbox));
        widget.append(&content);

        Self {
            widget,
            username_entry,
            password_entry,
            connect_btn,
            status_label,
            spinner,
        }
    }

    pub fn on_connect<F: Fn(String, String) + 'static>(&self, callback: F) {
        let username = self.username_entry.clone();
        let password = self.password_entry.clone();
        let status = self.status_label.clone();
        let spinner = self.spinner.clone();
        let btn = self.connect_btn.clone();

        self.connect_btn.clone().connect_clicked(move |_| {
            let user = username.text().to_string();
            let pass = password.text().to_string();

            if user.is_empty() || pass.is_empty() {
                status.set_text("Please enter username and password");
                return;
            }

            btn.set_sensitive(false);
            spinner.set_visible(true);
            status.set_text("Connecting...");

            callback(user, pass);
        });
    }

    pub fn show_status(&self, status: &ConnectionStatus) {
        match status {
            ConnectionStatus::Connecting => {
                self.status_label.set_text("Connecting...");
            }
            ConnectionStatus::ResolvingHost => {
                self.status_label.set_text("Resolving server.slsknet.org...");
            }
            ConnectionStatus::ConnectingToServer => {
                self.status_label.set_text("Connecting to server...");
            }
            ConnectionStatus::ConnectionRefused => {
                self.status_label.set_text("Connection refused - is the server address correct?");
                self.connect_btn.set_sensitive(true);
                self.spinner.set_visible(false);
            }
            ConnectionStatus::ConnectionFailed(e) => {
                self.status_label.set_text(&format!("Connection failed: {}", e));
                self.connect_btn.set_sensitive(true);
                self.spinner.set_visible(false);
            }
            ConnectionStatus::LoginFailed(e) => {
                self.status_label.set_text(&format!("Login failed: {}", e));
                self.connect_btn.set_sensitive(true);
                self.spinner.set_visible(false);
            }
            ConnectionStatus::Connected => {
                self.status_label.set_text("Connected!");
                self.spinner.set_visible(false);
            }
        }
    }

    pub fn widget(&self) -> gtk::Box {
        self.widget.clone()
    }
}

impl Default for LoginView {
    fn default() -> Self {
        Self::new()
    }
}
