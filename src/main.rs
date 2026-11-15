//! BeatFlow Native Desktop Entry Point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::ComponentHandle;
use beatflow::platform::single_instance::check_and_forward;
use beatflow::services::network::start_loopback_server;
use beatflow::{AppCoordinator, AppWindow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Check single instance: if another instance is running, forward deep-link argument and exit
    let mut initial_arg = String::new();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("beatflow://") {
            initial_arg = arg;
            break;
        }
    }

    if check_and_forward(&initial_arg) {
        return Ok(());
    }

    log::info!("Starting BeatFlow Native Desktop Core Engine...");

    // Start local loopback server for range requests and IPC
    start_loopback_server(Some(Box::new(|arg| {
        log::info!("Received forwarded argument from secondary instance: {}", arg);
    })));

    let app = AppWindow::new()?;

    // Initialize core application coordinator
    let coordinator = AppCoordinator::new(&app);

    // Load initial home discovery data
    coordinator.load_initial_home_content();

    // Setup window close callback
    app.on_window_close(move || {
        slint::quit_event_loop().unwrap_or(());
    });

    log::info!("BeatFlow Native UI shell ready.");
    app.run()?;

    Ok(())
}

