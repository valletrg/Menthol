mod login;
mod main_window;

use adw::prelude::*;
use adw::Application;
use std::time::Duration;

use slsk_core::{start, Config};

use login::ConnectionStatus;
use main_window::MainWindow;

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Application::builder()
        .application_id("io.github.slskr.slskr")
        .build();

    app.connect_activate(|app| {
        // Create one window that holds everything
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("slskr - Connect")
            .default_width(400)
            .default_height(300)
            .build();

        // Start with login content
        let login_view = login::LoginView::new();
        window.set_content(Some(&login_view.widget()));
        window.present();

        let window_clone = window.downgrade();
        let login_view_clone = login_view.clone();

        login_view.on_connect(move |username, password| {
            tracing::info!("Connecting as {}...", username);
            login_view_clone.show_status(&ConnectionStatus::Connecting);

            let config = Config {
                username,
                password,
                host: "server.slsknet.org".to_string(),
                port: 2242,
                listen_port: 2234,
                major_version: 160,
                minor_version: 1,
            };

            let core_handle = start(config);

            // Transition UI to main window
            if let Some(win) = window_clone.upgrade() {
                win.set_title(Some("slskr"));
                win.set_default_size(1200, 800);

                let main_view = MainWindow::new();
                main_view.set_core_handle(core_handle);

                win.set_content(Some(&main_view.widget()));
                win.present();

                // Start polling for events
                let main_view_clone = main_view.clone();
                glib::timeout_add_local(Duration::from_millis(100), move || {
                    main_view_clone.poll_events();
                    glib::ControlFlow::Continue
                });
            }
        });
    });

    app.run()
}
