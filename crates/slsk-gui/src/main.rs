mod login;
mod main_window;
mod panels;
mod profiles;

use adw::prelude::*;
use adw::Application;
use std::rc::Rc;
use std::time::Duration;

use slsk_core::{start, Config};

use login::ConnectionStatus;
use main_window::MainWindow;

fn main() -> glib::ExitCode {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Application::builder()
        .application_id("io.github.menthol.menthol")
        .build();

    app.connect_activate(|app| {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Menthol")
            .default_width(420)
            .default_height(520)
            .build();

        let login_view = login::LoginView::new();
        login_view.load_profiles();
        window.set_content(Some(&login_view.widget()));
        window.present();

        let window_clone = window.downgrade();
        let login_view_clone = login_view.clone();

        login_view.on_connect(Rc::new(move |username, password| {
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

            if let Some(win) = window_clone.upgrade() {
                win.set_default_size(1200, 800);

                let main_view = MainWindow::new();
                main_view.set_core_handle(core_handle);

                win.set_content(Some(&main_view.container()));
                win.present();

                let main_view_clone = main_view.clone();
                glib::timeout_add_local(Duration::from_millis(100), move || {
                    main_view_clone.poll_events();
                    glib::ControlFlow::Continue
                });
            }
        }));
    });

    app.run()
}
