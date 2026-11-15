//! Global media keys listener using native Windows API.
//!
//! Captures Play/Pause, Next Track, Previous Track, and Stop keys globally.

use std::sync::Arc;
use crate::core::AppCoordinator;

#[cfg(target_os = "windows")]
pub fn start_media_keys_listener(coordinator: Arc<AppCoordinator>) {
    std::thread::Builder::new()
        .name("beatflow-media-keys".into())
        .spawn(move || {
            unsafe {
                use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
                    RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT,
                    VK_MEDIA_PLAY_PAUSE, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP,
                };
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    GetMessageW, MSG, WM_HOTKEY,
                };

                const HOTKEY_PLAY_PAUSE: i32 = 1001;
                const HOTKEY_NEXT: i32 = 1002;
                const HOTKEY_PREV: i32 = 1003;
                const HOTKEY_STOP: i32 = 1004;

                let reg_play = RegisterHotKey(0, HOTKEY_PLAY_PAUSE, MOD_NOREPEAT, VK_MEDIA_PLAY_PAUSE as u32);
                let reg_next = RegisterHotKey(0, HOTKEY_NEXT, MOD_NOREPEAT, VK_MEDIA_NEXT_TRACK as u32);
                let reg_prev = RegisterHotKey(0, HOTKEY_PREV, MOD_NOREPEAT, VK_MEDIA_PREV_TRACK as u32);
                let reg_stop = RegisterHotKey(0, HOTKEY_STOP, MOD_NOREPEAT, VK_MEDIA_STOP as u32);

                log::info!(
                    "[BeatFlow MediaKeys] Global Hotkeys registered: PlayPause={}, Next={}, Prev={}, Stop={}",
                    reg_play != 0, reg_next != 0, reg_prev != 0, reg_stop != 0
                );

                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    if msg.message == WM_HOTKEY {
                        match msg.wParam as i32 {
                            HOTKEY_PLAY_PAUSE => {
                                log::info!("[BeatFlow MediaKeys] Global Media Play/Pause triggered");
                                coordinator.toggle_play();
                            }
                            HOTKEY_NEXT => {
                                log::info!("[BeatFlow MediaKeys] Global Media Next triggered");
                                coordinator.next_track();
                            }
                            HOTKEY_PREV => {
                                log::info!("[BeatFlow MediaKeys] Global Media Prev triggered");
                                coordinator.previous_track();
                            }
                            HOTKEY_STOP => {
                                log::info!("[BeatFlow MediaKeys] Global Media Stop triggered");
                                coordinator.player.pause();
                            }
                            _ => {}
                        }
                    }
                }

                UnregisterHotKey(0, HOTKEY_PLAY_PAUSE);
                UnregisterHotKey(0, HOTKEY_NEXT);
                UnregisterHotKey(0, HOTKEY_PREV);
                UnregisterHotKey(0, HOTKEY_STOP);
            }
        })
        .expect("Failed to spawn media keys listener thread");
}

#[cfg(not(target_os = "windows"))]
pub fn start_media_keys_listener(_coordinator: Arc<AppCoordinator>) {
    log::info!("[BeatFlow MediaKeys] Global media keys listener is a no-op on non-Windows platforms.");
}
