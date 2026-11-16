//! BeatFlow's native shell (Slint) — UI + native audio engine.
//! Playback uses the audio/stream modules adapted from refrence/rust.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::unbounded;
use slint::{ComponentHandle, Timer, TimerMode};
use tokio::runtime::Runtime;

slint::include_modules!();

mod backend;

use backend::audio::{self, AudioCmd, AudioEvent, AudioHandle};
use backend::innertube;
use backend::stream::{self, Ranging, Shared};
use backend::ytm;
use backend::ytmapi::Api;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppWindow::new()?;

    // Tokio runtime for async stream resolution/download.
    let rt = Arc::new(Runtime::new()?);

    // Audio engine.
    let (events_tx, events_rx) = unbounded::<AudioEvent>();
    let audio_handle = Arc::new(audio::spawn(
        1.0,
        false,
        None,
        events_tx,
        || {
            let _ = slint::invoke_from_event_loop(|| {});
        },
    ));

    // Mock track metadata (a real resolver will replace this with Stage 4).
    let player = app.global::<Player>();
    player.set_title("Demo Track".into());
    player.set_artist("Demo Artist".into());
    player.set_duration(222.0);

    let weak = app.as_weak();
    let audio = audio_handle.clone();
    let rt2 = rt.clone();
    player.on_toggle_play(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        match player.get_status() {
            0 => {
                // paused -> buffering -> resolve & play
                player.set_status(2);
                player.set_beat_phase(0.0);
                let audio = audio.clone();
                let weak = app.as_weak();
                let rt = rt2.clone();
                std::thread::spawn(move || {
                    let video_id = "dQw4w9WgXcQ";
                    let client = reqwest::Client::new();
                    let stream = match rt.block_on(innertube::audio_stream(&client, video_id, "", true)) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("resolve failed: {e}");
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = weak.upgrade() {
                                    app.global::<Player>().set_status(0);
                                }
                            });
                            return;
                        }
                    };
                    let shared = Shared::new(
                        stream.size,
                        stream.duration_ms,
                        stream.loudness_db,
                        backend::stream::Format::from_opus(stream.opus),
                        None,
                        false,
                    );
                    rt.block_on(stream::download(
                        client.clone(),
                        stream.url.clone(),
                        innertube::UA.to_string(),
                        shared.clone(),
                        None,
                        Ranging::Query,
                    ));
                    if let Some(e) = shared.error() {
                        eprintln!("download failed: {e}");
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = weak.upgrade() {
                                app.global::<Player>().set_status(0);
                            }
                        });
                        return;
                    }
                    let decoder = match stream::decoder(shared.clone(), false) {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("decode failed: {e}");
                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(app) = weak.upgrade() {
                                    app.global::<Player>().set_status(0);
                                }
                            });
                            return;
                        }
                    };
                    let dur = stream.duration_ms as f64 / 1000.0;
                    audio.send(AudioCmd::Start {
                        generation: 1,
                        decoder,
                        shared: shared.clone(),
                        at: Duration::ZERO,
                    });
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = weak.upgrade() {
                            let player = app.global::<Player>();
                            if dur > 0.0 {
                                player.set_duration(dur as f32);
                            }
                            player.set_status(1);
                        }
                    });
                });
            }
            _ => {
                audio.send(AudioCmd::Pause);
                player.set_status(0);
            }
        }
    });

    // next / previous
    let weak = app.as_weak();
    let audio = audio_handle.clone();
    player.on_next(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_elapsed(0.0);
        player.set_progress(0.0);
        audio.send(AudioCmd::Seek(Duration::ZERO));
    });
    let weak = app.as_weak();
    let audio = audio_handle.clone();
    player.on_previous(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_elapsed(0.0);
        player.set_progress(0.0);
        audio.send(AudioCmd::Seek(Duration::ZERO));
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
        player.set_repeat((player.get_repeat() + 1) % 3);
    });

    let weak = app.as_weak();
    let audio = audio_handle.clone();
    player.on_seeked(move |fraction| {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        let dur = player.get_duration();
        let target = fraction * dur;
        player.set_elapsed(target);
        player.set_progress(fraction);
        audio.send(AudioCmd::Seek(Duration::from_secs_f32(target)));
    });

    let weak = app.as_weak();
    let audio = audio_handle.clone();
    player.on_set_volume(move |volume| {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_volume((volume.clamp(0.0, 1.0) * 100.0).round() / 100.0);
        player.set_muted(false);
        audio.send(AudioCmd::Volume(player.get_volume()));
    });
    let weak = app.as_weak();
    player.on_toggle_mute(move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        player.set_muted(!player.get_muted());
    });
    let weak = app.as_weak();
    let audio = audio_handle.clone();
    player.on_wheel_volume(move |delta_y| {
        if delta_y == 0.0 { return; }
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        let new_vol = player.get_volume() + if delta_y < 0.0 { 0.05 } else { -0.05 };
        let clamped = new_vol.clamp(0.0, 1.0);
        player.set_volume((clamped * 100.0).round() / 100.0);
        player.set_muted(false);
        audio.send(AudioCmd::Volume(player.get_volume()));
    });

    // Window chrome callbacks
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
        app.global::<Ui>().set_maximized(window.is_maximized());
    });
    let weak = app.as_weak();
    app.on_close_requested(move || {
        if let Some(app) = weak.upgrade() {
            let _ = app.hide();
        }
        let _ = slint::quit_event_loop();
    });

    // Progress timer: read from audio status.
    let progress_timer = Timer::default();
    let weak = app.as_weak();
    let audio = audio_handle.clone();
    progress_timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        let Some(app) = weak.upgrade() else { return };
        let player = app.global::<Player>();
        if player.get_status() != 1 {
            return;
        }
        let pos = audio.status.position_smooth();
        let secs = pos.as_secs_f64();
        player.set_elapsed(secs as f32);
        let dur = player.get_duration();
        player.set_progress(if dur > 0.0 { (secs as f32) / dur } else { 0.0 });
    });

    // Beat loader timer.
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

    // Event drain timer.
    let event_timer = Timer::default();
    let weak = app.as_weak();
    let events = events_rx;
    event_timer.start(TimerMode::Repeated, Duration::from_millis(50), move || {
        while let Ok(ev) = events.try_recv() {
            match ev {
                AudioEvent::Finished { .. } => {
                    if let Some(app) = weak.upgrade() {
                        let player = app.global::<Player>();
                        if player.get_repeat() != 0 {
                            player.set_elapsed(0.0);
                            player.set_progress(0.0);
                        } else {
                            player.set_status(0);
                        }
                    }
                }
                AudioEvent::Interrupted { at, .. } => {
                    eprintln!("interrupted at {at:?}");
                }
                AudioEvent::Error(e) => {
                    eprintln!("audio error: {e}");
                    if let Some(app) = weak.upgrade() {
                        app.global::<Player>().set_status(0);
                    }
                }
                _ => {}
            }
        }
    });

    // Search: wire Search global to ytm.
let api = Arc::new(Api::new(std::env::temp_dir().as_path()));
let http = reqwest::Client::new();
let weak = app.as_weak();
let api2 = api.clone();
let http2 = http.clone();
app.global::<Search>().on_search(move |q| {
    let weak = weak.clone();
    let api = api2.clone();
    let http = http2.clone();
    let query = q.to_string();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(api.ensure_visitor());
        let visitor = rt.block_on(api.visitor_data(false)).unwrap_or_default();
        let results = rt.block_on(ytm::search_songs(&api, &query, false)).unwrap_or_default();
        // Convert to Slint SearchResult
let slint_results: Vec<SearchResult> = results
            .iter()
            .map(|t| SearchResult {
                title: t.title.clone().into(),
                artist: t.artist_line().into(),
                video_id: t.id.clone().into(),
            })
            .collect();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = weak.upgrade() {
                let search = app.global::<Search>();
                search.set_query(query.into());
                let model = slint::ModelRc::new(slint::VecModel::from(slint_results));
                search.set_results(model);
            }
        });
    });
});

app.show()?;

    let chrome_weak = app.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(app) = chrome_weak.upgrade() {
            apply_chrome(&app);
        }
    })?;

    let _timers = (progress_timer, beat_timer, event_timer);
    app.run()?;
    Ok(())
}

#[cfg(windows)]
fn apply_chrome(app: &AppWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};

    let handle = app.window().window_handle();
    let _ = window_vibrancy::apply_mica(&handle, Some(true));

    let Ok(handle) = handle.window_handle() else { return };
    let RawWindowHandle::Win32(win) = handle.as_raw() else { return };
    let hwnd = HWND(win.hwnd.get() as _);

    unsafe {
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