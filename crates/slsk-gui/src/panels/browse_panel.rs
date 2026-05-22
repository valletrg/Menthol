use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation};

pub struct BrowsePanel {
    pub container: GtkBox,
}

impl BrowsePanel {
    pub fn new() -> Self {
        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .vexpand(true)
            .build();

        let label = Label::new(Some("Browse Panel — Phase 4"));
        label.add_css_class("title-1");
        label.set_halign(Align::Center);
        label.set_valign(Align::Center);
        container.append(&label);

        Self { container }
    }
}

impl Default for BrowsePanel {
    fn default() -> Self {
        Self::new()
    }
}
