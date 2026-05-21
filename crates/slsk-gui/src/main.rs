use adw::prelude::*;

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.slskr.slskr")
        .build();

    app.connect_activate(|app| {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("slskr")
            .default_width(1200)
            .default_height(800)
            .build();

        window.present();
    });

    app.run()
}
