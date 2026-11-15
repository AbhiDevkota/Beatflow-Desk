//! BeatFlow Native Desktop Entry Point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::ComponentHandle;
use beatflow::platform::single_instance::check_and_forward;
use beatflow::services::network::start_loopback_server;
use beatflow::{AppCoordinator, AppWindow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Check single instance: if another instance is running, forward argument and exit
    let mut initial_arg = String::new();
    for arg in std::env::args().skip(1) {
        if arg.starts_with("beatflow://") || arg.ends_with(".mp3") || arg.ends_with(".flac") || arg.ends_with(".m4a") {
            initial_arg = arg;
            break;
        }
    }

    let forward_payload = if initial_arg.is_empty() { "activate" } else { &initial_arg };
    if check_and_forward(forward_payload) {
        log::info!("Another BeatFlow instance is running. Forwarded argument and exiting.");
        return Ok(());
    }

    log::info!("Starting BeatFlow Native Desktop Core Engine...");

    // Register beatflow:// protocol handler in the OS
    beatflow::platform::protocol::register_protocol_handler();

    let app = AppWindow::new()?;

    // Initialize core application coordinator
    let coordinator = AppCoordinator::new(&app);

    // Load initial home discovery data
    coordinator.load_initial_home_content();

    // Start background media keys listener (Play/Pause, Next, Prev, Stop)
    beatflow::platform::media_keys::start_media_keys_listener(std::sync::Arc::clone(&coordinator));

    // Start local loopback server for range requests and single-instance IPC
    let coord_ipc = std::sync::Arc::clone(&coordinator);
    start_loopback_server(Some(Box::new(move |arg| {
        log::info!("Received forwarded argument from secondary instance: {}", arg);
        beatflow::platform::protocol::handle_deep_link(&arg, &coord_ipc);
    })));

    // Process initial deep-link if launched with one
    if !initial_arg.is_empty() {
        beatflow::platform::protocol::handle_deep_link(&initial_arg, &coordinator);
    }

    // Setup window close callback
    let _app_weak_drag = app.as_weak();
    app.on_window_drag(move || {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows_sys::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
            use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_NCLBUTTONDOWN, HTCAPTION, GetForegroundWindow};
            let hwnd = GetForegroundWindow();
            if hwnd != 0 {
                ReleaseCapture();
                SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as usize, 0);
            }
        }
    });
    app.on_window_close(move || {
        slint::quit_event_loop().unwrap_or(());
    });

    let app_weak_min = app.as_weak();
    app.on_window_minimize(move || {
        if let Some(w) = app_weak_min.upgrade() {
            w.window().set_minimized(true);
        }
    });

    let app_weak_max = app.as_weak();
    app.on_window_maximize(move || {
        if let Some(w) = app_weak_max.upgrade() {
            let is_max = w.window().is_maximized();
            w.window().set_maximized(!is_max);
        }
    });

    log::info!("BeatFlow Native UI shell ready.");
    app.run()?;

    Ok(())
}

