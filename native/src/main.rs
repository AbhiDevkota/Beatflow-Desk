//! BeatFlow's native shell (Slint) — the UI-only skeleton of the desktop
//! app's port off the web stack, one component at a time
//! (.claude/NATIVE_MIGRATION_BLUEPRINT.md). Everything playback-related is
//! mocked here: a 3:42 demo track, a playhead timer, and a
//! loading → playing toggle that feeds the BeatLoader. The audio engine
//! arrives with the blueprint's Stage 1; see native/README.md for what's
//! mocked and what deviates from the web app.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};

slint::include_modules!();

mod backend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWindow::new()?;

    // ---- Mock playback state (state/player.ts's shape) --------------------
    // "Demo Track" / "Demo Artist", 3:42 (222s), paused at 0:00, full volume,
    // signed out (no favorite/menu queue state yet).
    let player = app.global::<Player>();
    player.set_title("Demo Track".into());
    player.set_artist("Demo Artist".into());
    player.set_duration(222.0);

    // ---- Player callbacks --------------------------------------------------
    let weak = app.as_weak();
    player.on_toggle_play(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        match player.get_status() {
            // paused → the loader's latency: buffering for 600ms, then playing.
            0 => {
                player.set_status(2);
                // The web's CSS animation restarts when the loader mounts.
                player.set_beat_phase(0.0);
                let weak = app.as_weak();
                Timer::single_shot(Duration::from_millis(600), move || {
                    let Some(app) = weak.upgrade() else { return };
                    let player = app.global::<Player>();
                    // Ignore the tick if the user paused meanwhile.
                    if player.get_status() == 2 {
                        player.set_status(1);
                    }
                });
            }
            // playing / buffering → paused.
            _ => player.set_status(0),
        }
    });

    // next / previous: the mock queue is one track, so both restart it.
    let weak = app.as_weak();
    player.on_next(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_elapsed(0.0);
        sync_time(&app);
    });

    let weak = app.as_weak();
    player.on_previous(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_elapsed(0.0);
        sync_time(&app);
    });

    let weak = app.as_weak();
    player.on_toggle_shuffle(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_shuffle(!player.get_shuffle());
    });

    let weak = app.as_weak();
    player.on_cycle_repeat(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        // off → all → one → off
        player.set_repeat((player.get_repeat() + 1) % 3);
    });

    // useScrubber seeks on release; the Lcd shows the drag position live.
    let weak = app.as_weak();
    player.on_seeked(move |fraction| {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_elapsed(fraction * player.get_duration());
        sync_time(&app);
    });

    let weak = app.as_weak();
    player.on_set_volume(move |volume| {
        let Some(app) = weak.upgrade() else { return };
        set_volume(&app, volume);
    });

    let weak = app.as_weak();
    player.on_toggle_mute(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_muted(!player.get_muted());
    });

    // VolumeControl.tsx's useVolumeWheel: one notch steps 0.05, up on
    // negative deltaY, clamped and rounded to whole percents
    // (wheelVolume.ts); a zero delta leaves the volume unchanged.
    let weak = app.as_weak();
    player.on_wheel_volume(move |delta_y| {
        if delta_y == 0.0 {
            return;
        }
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        set_volume(&app, player.get_volume() + if delta_y < 0.0 { 0.05 } else { -0.05 });
    });

    // ---- Window chrome callbacks -------------------------------------------
    let weak = app.as_weak();
    app.on_minimize(move || {
        if let Some(app) = weak.upgrade() {
            app.window().set_minimized(true);
        }
    });

    let weak = app.as_weak();
    app.on_toggle_maximize(move || {
        let Some(app) = weak.upgrade() else { return };
        let window = app.window();
        window.set_maximized(!window.is_maximized());
        // The restore glyph follows the real state; the strip's double click
        // and the maximize button both land here.
        app.global::<Ui>().set_maximized(window.is_maximized());
    });

    let weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
        let _ = slint::quit_event_loop();
    });

    // ---- Timers -------------------------------------------------------------
    // The playhead: +100ms per tick while playing, wrapping on repeat all/one
    // and pausing at the end otherwise.
    let progress_timer = Timer::default();
    let weak = app.as_weak();
    progress_timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        if player.get_status() != 1 {
            return;
        }
        let mut elapsed = player.get_elapsed() + 0.1;
        let duration = player.get_duration();
        if elapsed >= duration {
            if player.get_repeat() != 0 {
                elapsed -= duration;
            } else {
                player.set_status(0);
                elapsed = duration;
            }
        }
        player.set_elapsed(elapsed);
        sync_time(&app);
    });

    // The BeatLoader: one 16ms tick advancing beat-phase over the 1.9s full
    // cycle of the web's 950ms alternate keyframes, only while buffering.
    let beat_timer = Timer::default();
    let weak = app.as_weak();
    beat_timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        if player.get_status() < 2 {
            return;
        }
        player.set_beat_phase((player.get_beat_phase() + 0.016 / 1.9) % 1.0);
    });

    app.show()?;

    // The winit window only exists once the event loop is running, so the
    // Windows chrome (Mica, DWM shadow, rounded corners) is applied on the
    // first event-loop tick.
    let chrome_weak = app.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(app) = chrome_weak.upgrade() {
            apply_chrome(&app);
        }
    })?;

    // Keep the timers alive for the event loop's lifetime.
    let _timers = (progress_timer, beat_timer);
    app.run()?;
    Ok(())
}

/// Writes `progress` from `elapsed`/`duration`; the Lcd formats the clock
/// labels itself, the way useScrubber.ts's draw loop writes them each frame.
fn sync_time(app: &AppWindow) {
    let player = app.global::<Player>();
    let duration = player.get_duration();
    let elapsed = player.get_elapsed();
    player.set_progress(if duration > 0.0 { elapsed / duration } else { 0.0 });
}

/// state/player.ts's setVolume: clamped to [0, 1], rounded to whole percents
/// so repeated wheel steps don't accumulate float dust, and it clears mute —
/// scrolling up while muted unmutes.
fn set_volume(app: &AppWindow, requested: f32) {
    let player = app.global::<Player>();
    player.set_volume((requested.clamp(0.0, 1.0) * 100.0).round() / 100.0);
    player.set_muted(false);
}

/// The Windows 11 chrome the frameless window owes: Mica behind the
/// transparent stage, the DWM drop shadow frameless windows lose, and the
/// OS's rounded corners. (window-shadows can't be used: it targets
/// raw-window-handle 0.5, while Slint runs with 0.6 here.)
#[cfg(windows)]
fn apply_chrome(app: &AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};

    let handle = app.window().window_handle();
    // The app launches dark (tokens.css's dark palette is the default).
    let _ = window_vibrancy::apply_mica(&handle, Some(true));

    let Ok(handle) = handle.window_handle() else { return };
    let RawWindowHandle::Win32(win) = handle.as_raw() else { return };
    let hwnd = HWND(win.hwnd.get() as _);

    unsafe {
        // Slint's frameless windows get no OS shadow: extending a 1px sheet
        // of frame into the client area turns the DWM one back on. The
        // windows crate gates this function behind Win32_UI_Controls just
        // for its MARGINS type, so the ABI-identical 4×i32 struct and the
        // dwmapi import are declared here instead.
        #[repr(C)]
        struct Margins {
            cx_left_width: i32,
            cx_right_width: i32,
            cy_top_height: i32,
            cy_bottom_height: i32,
        }
        #[link(name = "dwmapi")]
        unsafe extern "system" {
            fn DwmExtendFrameIntoClientArea(hwnd: HWND, margins: *const Margins) -> i32;
        }
        let _ = DwmExtendFrameIntoClientArea(
            hwnd,
            &Margins {
                cx_left_width: 1,
                cx_right_width: 1,
                cy_top_height: 1,
                cy_bottom_height: 1,
            },
        );

        // Rounded corners (a no-op before Windows 11).
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &DWMWCP_ROUND as *const _ as *const core::ffi::c_void,
            core::mem::size_of_val(&DWMWCP_ROUND) as u32,
        );
    }
}

#[cfg(not(windows))]
fn apply_chrome(_: &AppWindow) {}
