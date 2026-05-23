//! Login view widget - profile picker + credential form.
//!
//! Two modes:
//!  - picker: shows saved profiles as clickable rows + "Add New" button
//!  - form:   shows username + password fields + "Save profile" checkbox + Connect/Back

use adw::gtk::{self, Align, Button, CheckButton, Entry, Label, PasswordEntry, Separator, Spinner};
use adw::prelude::*;
use gtk::prelude::*;
use gtk4::Orientation;
use std::rc::Rc;

use crate::profiles::{self, Profile};

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

#[derive(Clone)]
pub struct LoginView {
    widget: gtk::Box,
    stack: gtk::Stack,
    profile_list: gtk::ListBox,
    add_profile_btn: Button,
    username_entry: Entry,
    password_entry: PasswordEntry,
    save_checkbox: CheckButton,
    connect_btn: Button,
    back_btn: Button,
    status_label: Label,
    spinner: Spinner,
}

impl LoginView {
    pub fn new() -> Self {
        let widget = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Minimal header bar — just the title, no window control buttons
        let header = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(16)
            .margin_end(16)
            .margin_top(12)
            .margin_bottom(12)
            .build();
        let title_label = gtk::Label::new(Some("Menthol"));
        title_label.add_css_class("title-2");
        header.append(&title_label);

        // Stack: picker page | form page
        let stack = gtk::Stack::builder()
            .vexpand(true)
            .transition_type(gtk::StackTransitionType::SlideLeftRight)
            .build();

        // ── Picker page ──────────────────────────────────────
        let picker_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let heading = Label::new(Some("Select a profile"));
        heading.add_css_class("title-2");
        heading.set_halign(Align::Start);
        picker_box.append(&heading);

        picker_box.append(
            &Separator::builder()
                .orientation(Orientation::Horizontal)
                .build(),
        );

        let profile_list = gtk::ListBox::builder().vexpand(true).build();
        profile_list.add_css_class("boxed-list");

        let add_profile_btn = Button::builder()
            .label("Add New Profile")
            .halign(Align::Center)
            .build();
        add_profile_btn.add_css_class("suggested-action");

        picker_box.append(&profile_list);
        picker_box.append(&add_profile_btn);

        stack.add_named(&picker_box, Some("picker"));

        // ── Form page ──────────────────────────────────────
        let form_box = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let form_heading = Label::new(Some("Add Profile"));
        form_heading.add_css_class("title-2");
        form_heading.set_halign(Align::Start);
        form_box.append(&form_heading);

        form_box.append(
            &Separator::builder()
                .orientation(Orientation::Horizontal)
                .build(),
        );

        let username_label = Label::new(Some("Username"));
        username_label.set_halign(Align::Start);
        form_box.append(&username_label);

        let username_entry = Entry::builder().placeholder_text("Enter username").build();

        let password_label = Label::new(Some("Password"));
        password_label.set_halign(Align::Start);
        password_label.set_margin_top(16);
        form_box.append(&password_label);

        let password_entry = PasswordEntry::builder()
            .placeholder_text("Enter password")
            .build();

        let save_checkbox = CheckButton::builder()
            .label("Save profile")
            .margin_top(16)
            .active(true)
            .build();

        let status_label = Label::new(None::<&str>);
        status_label.add_css_class("dim-label");
        status_label.set_halign(Align::Start);
        status_label.set_margin_top(8);

        let spinner = Spinner::builder().halign(Align::Center).build();
        spinner.set_visible(false); // hidden until connecting

        let button_row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_top(24)
            .build();

        let back_btn = Button::builder().label("Back").halign(Align::Start).build();

        let connect_btn = Button::builder()
            .label("Connect")
            .halign(Align::End)
            .build();
        connect_btn.add_css_class("suggested-action");

        button_row.append(&back_btn);
        button_row.append(&connect_btn);

        form_box.append(&username_entry);
        form_box.append(&password_label);
        form_box.append(&password_entry);
        form_box.append(&save_checkbox);
        form_box.append(&status_label);
        form_box.append(&spinner);
        form_box.append(&button_row);

        stack.add_named(&form_box, Some("form"));

        widget.append(&header);
        widget.append(&stack);

        Self {
            widget,
            stack,
            profile_list,
            add_profile_btn,
            username_entry,
            password_entry,
            save_checkbox,
            connect_btn,
            back_btn,
            status_label,
            spinner,
        }
    }

    /// Populate profile list from disk.
    pub fn load_profiles(&self) {
        while let Some(row) = self.profile_list.row_at_index(0) {
            self.profile_list.remove(&row);
        }

        let profiles = profiles::load_profiles();

        if profiles.is_empty() {
            let hint = Label::new(Some("No saved profiles — add one below"));
            hint.add_css_class("dim-label");
            hint.set_halign(Align::Center);
            let row = gtk::ListBoxRow::builder().child(&hint).build();
            row.set_selectable(false);
            self.profile_list.prepend(&row);
        } else {
            for profile in profiles {
                let row = Self::make_profile_row(profile);
                self.profile_list.append(&row);
            }
        }
    }

    fn make_profile_row(profile: Profile) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::builder().build();
        let box_ = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let icon = gtk::Image::from_icon_name("avatar-default");

        let name_label = Label::new(Some(&profile.username));
        name_label.set_halign(Align::Start);
        name_label.set_hexpand(true);

        box_.append(&icon);
        box_.append(&name_label);
        row.set_child(Some(&box_));
        row
    }

    /// Register a callback for when the user triggers connection.
    /// `callback` receives (username, password).
    pub fn on_connect(&self, callback: Rc<dyn Fn(String, String)>) {
        // Add Profile button → switch to form, clear fields
        let stack = self.stack.clone();
        let username_e = self.username_entry.clone();
        let password_e = self.password_entry.clone();
        self.add_profile_btn.clone().connect_clicked(move |_| {
            username_e.set_text("");
            password_e.set_text("");
            stack.set_visible_child_name("form");
        });

        // Back button → return to picker
        let stack = self.stack.clone();
        self.back_btn.clone().connect_clicked(move |_| {
            stack.set_visible_child_name("picker");
        });

        // Profile row click → fill form AND auto-connect with saved credentials
        {
            let cb = callback.clone();
            let status = self.status_label.clone();
            let spinner = self.spinner.clone();
            let btn = self.connect_btn.clone();
            self.profile_list.connect_row_activated(move |list, row| {
                let idx = row.index();
                let profiles = profiles::load_profiles();
                if idx as usize >= profiles.len() {
                    return;
                }
                let profile = &profiles[idx as usize];
                status.set_text(&format!("Connecting as {}...", profile.username));
                spinner.set_visible(true);
                btn.set_sensitive(false);
                list.select_row(None::<&gtk::ListBoxRow>);
                let pass = profile.password.clone().unwrap_or_default();
                cb(profile.username.clone(), pass);
            });
        }

        // Connect button in form
        {
            let cb = callback.clone();
            let username_e = self.username_entry.clone();
            let password_e = self.password_entry.clone();
            let save_btn = self.save_checkbox.clone();
            let status = self.status_label.clone();
            let spinner = self.spinner.clone();
            let btn = self.connect_btn.clone();

            self.connect_btn.clone().connect_clicked(move |_| {
                let user = username_e.text().to_string();
                let pass = password_e.text().to_string();
                if user.is_empty() || pass.is_empty() {
                    status.set_text("Please enter username and password");
                    return;
                }
                btn.set_sensitive(false);
                spinner.set_visible(true);
                status.set_text("Connecting...");
                if save_btn.is_active() {
                    if let Err(e) = profiles::add_profile(Profile {
                        name: user.clone(),
                        username: user.clone(),
                        password: Some(pass.clone()),
                    }) {
                        tracing::warn!("Failed to save profile: {}", e);
                    }
                }
                cb(user, pass);
            });
        }

        // Enter key in password field triggers connect
        {
            let cb = callback.clone();
            let username_e = self.username_entry.clone();
            let password_e = self.password_entry.clone();
            let save_btn = self.save_checkbox.clone();
            let status = self.status_label.clone();
            let spinner = self.spinner.clone();
            let btn = self.connect_btn.clone();

            self.password_entry.clone().connect_activate(move |_| {
                let user = username_e.text().to_string();
                let pass = password_e.text().to_string();
                if user.is_empty() || pass.is_empty() {
                    status.set_text("Please enter username and password");
                    return;
                }
                btn.set_sensitive(false);
                spinner.set_visible(true);
                status.set_text("Connecting...");
                if save_btn.is_active() {
                    if let Err(e) = profiles::add_profile(Profile {
                        name: user.clone(),
                        username: user.clone(),
                        password: Some(pass.clone()),
                    }) {
                        tracing::warn!("Failed to save profile: {}", e);
                    }
                }
                cb(user, pass);
            });
        }
    }

    pub fn show_status(&self, status: &ConnectionStatus) {
        match status {
            ConnectionStatus::Connecting => self.status_label.set_text("Connecting..."),
            ConnectionStatus::ResolvingHost => self
                .status_label
                .set_text("Resolving server.slsknet.org..."),
            ConnectionStatus::ConnectingToServer => {
                self.status_label.set_text("Connecting to server...")
            }
            ConnectionStatus::ConnectionRefused => {
                self.status_label
                    .set_text("Connection refused — is the server address correct?");
                self.connect_btn.set_sensitive(true);
                self.spinner.set_visible(false);
            }
            ConnectionStatus::ConnectionFailed(e) => {
                self.status_label
                    .set_text(&format!("Connection failed: {}", e));
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
