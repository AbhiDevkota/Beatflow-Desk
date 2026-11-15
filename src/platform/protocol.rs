//! Custom URI scheme (`beatflow://`) registration and deep-link routing.

use std::sync::Arc;
use crate::core::AppCoordinator;

/// Registers the `beatflow://` protocol in the Windows Registry under `HKCU\Software\Classes\beatflow`.
#[cfg(target_os = "windows")]
pub fn register_protocol_handler() {
    if let Ok(exe_path) = std::env::current_exe() {
        let exe_str = exe_path.to_string_lossy().to_string();
        let cmd_str = format!("\"{}\" \"%1\"", exe_str);

        // Run reg.exe to register HKCU protocol handler without needing admin elevation
        let _ = std::process::Command::new("reg")
            .args(["add", "HKCU\\Software\\Classes\\beatflow", "/ve", "/d", "URL:BeatFlow Protocol", "/f"])
            .output();
        let _ = std::process::Command::new("reg")
            .args(["add", "HKCU\\Software\\Classes\\beatflow", "/v", "URL Protocol", "/d", "", "/f"])
            .output();
        let _ = std::process::Command::new("reg")
            .args(["add", "HKCU\\Software\\Classes\\beatflow\\shell\\open\\command", "/ve", "/d", &cmd_str, "/f"])
            .output();

        log::info!("[BeatFlow Protocol] Registered beatflow:// protocol handler to: {}", exe_str);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn register_protocol_handler() {
    // Linux desktop entry or macOS CFBundleURLTypes would go here
}

/// Routes incoming deep link arguments to coordinator actions
pub fn handle_deep_link(arg: &str, coordinator: &Arc<AppCoordinator>) {
    let trimmed = arg.trim();
    if trimmed.is_empty() {
        return;
    }

    log::info!("[BeatFlow Protocol] Handling incoming argument / deep-link: {}", trimmed);

    // Bring window to foreground
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
        };
        // Encode "BeatFlow\0" in UTF-16
        let title: Vec<u16> = "BeatFlow\0".encode_utf16().collect();
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if hwnd != 0 {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }

    if trimmed == "activate" {
        return;
    }

    if trimmed.starts_with("beatflow://track/") {
        let track_id = trimmed.trim_start_matches("beatflow://track/");
        let c = Arc::clone(coordinator);
        let id = track_id.to_string();
        tokio::spawn(async move {
            c.play_track_id(&id).await;
        });
    } else if trimmed.contains("auth-callback") {
        if let Some(pos) = trimmed.find("data=") {
            let encoded = &trimmed[pos + 5..];
            let clean = encoded.split('&').next().unwrap_or(encoded);
            let decoded = urlencoding::decode(clean).unwrap_or_else(|_| clean.into());
            log::info!("[BeatFlow Protocol] Intercepted auth-callback payload length: {}", decoded.len());
            // Payload can be parsed for YouTube cookies / tokens
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&decoded) {
                log::info!("[BeatFlow Protocol] Persisting authenticated cookies in YouTube auth store");
                crate::services::youtube::auth::save_yt_auth(&val);
            }
        }
    } else if trimmed.ends_with(".mp3") || trimmed.ends_with(".flac") || trimmed.ends_with(".m4a") {
        log::info!("[BeatFlow Protocol] Requested local audio file playback: {}", trimmed);
        let path = std::path::PathBuf::from(trimmed);
        if path.exists() {
            let _ = coordinator.player.play_file(&path);
        }
    }
}
