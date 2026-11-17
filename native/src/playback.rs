//! Playback: resolving streams, the transport callbacks, and the timers that
//! drive progress, the BeatLoader and the audio event pump (was in main.rs).
//!
//! `play_track` bumps a generation, cancels the previous stream and hands a
//! decoder to the audio engine; every async step drops itself when a newer
//! track has taken over.

use std::sync::atomic::Ordering;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};

use crate::app::App;
use crate::backend::audio::{AudioCmd, AudioEvent};
use crate::backend::innertube;
use crate::backend::stream::{self, Ranging, Shared};
use crate::images;
use crate::state::TrackItem;

/// Start `video_id` at `at`, updating the UI to the buffering state at once.
pub fn play_track(app: &App, video_id: String, title: String, artist: String, at: Duration) {
    let app = app.clone();
    let generation = app.state.next_generation();
    app.audio.status.wanted_gen.store(generation, Ordering::Relaxed);

    // Cancel old stream download if any
    if let Some(old) = app.state.active_shared.lock().unwrap().take() {
        old.cancel();
    }
    app.audio.send(AudioCmd::Stop);

    // Update Slint UI to buffering state immediately
    if let Some(app_window) = app.weak.upgrade() {
        let player = app_window.global::<crate::Player>();
        player.set_has_track(true);
        player.set_title(title.clone().into());
        player.set_artist(artist.clone().into());
        player.set_status(2); // buffering
        player.set_beat_phase(0.0);
        player.set_progress(0.0);
        player.set_elapsed(at.as_secs_f32());
        let thumb_url = format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg");
        if let Some(cached) = app.state.image_cache.lock().unwrap().get(&thumb_url) {
            player.set_artwork(cached.to_slint());
            player.set_has_artwork(true);
        } else {
            player.set_has_artwork(false);
        }
    }

    // Spawn thumbnail loader for now-playing LCD
    let thumb_app = app.clone();
    let vid_img = video_id.clone();
    let gen_img = app.state.generation.clone();
    app.rt.spawn(async move {
        if let Some(raw) =
            images::get_video_thumbnail(&thumb_app.http, &thumb_app.state.image_cache, &vid_img).await
        {
            let weak = thumb_app.weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak.upgrade() {
                    if gen_img.load(Ordering::SeqCst) == generation {
                        let player = app.global::<crate::Player>();
                        player.set_artwork(raw.to_slint());
                        player.set_has_artwork(true);
                    }
                }
            });
        }
    });

    *app.state.current_track.lock().unwrap() = Some(TrackItem {
        video_id: video_id.clone(),
        title: title.clone(),
        artist: artist.clone(),
        duration: 0.0,
    });

    let active_shared = app.state.active_shared.clone();
    let current_track = app.state.current_track.clone();
    let gen_atomic = app.state.generation.clone();
    let weak = app.weak.clone();
    let http = app.http.clone();
    let audio = app.audio.clone();

    let rt = app.rt.clone();
    rt.spawn(async move {
        let api = app.api.clone();

        // Ensure visitor data for unciphered visionOS player
        let _ = api.ensure_visitor().await;
        let visitor = api.visitor_data(false).await.unwrap_or_default();

        let stream = match innertube::audio_stream(&http, &video_id, &visitor, true).await {
            Ok(s) => s,
            Err(e) => {
                log::warn!("visionOS player stream error ({e:#}); retrying with refreshed visitor data");
                let visitor = api.visitor_data(true).await.unwrap_or_default();
                match innertube::audio_stream(&http, &video_id, &visitor, true).await {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("stream resolve failed for {video_id}: {e:#}");
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(app) = weak.upgrade() {
                                if gen_atomic.load(Ordering::SeqCst) == generation {
                                    app.global::<crate::Player>().set_status(0);
                                }
                            }
                        });
                        return;
                    }
                }
            }
        };

        if gen_atomic.load(Ordering::SeqCst) != generation {
            return;
        }

        let shared = Shared::new(
            stream.size,
            stream.duration_ms,
            stream.loudness_db,
            stream::Format::from_opus(stream.opus),
            None,
            false,
        );
        *active_shared.lock().unwrap() = Some(shared.clone());

        // Spawn parallel chunked download
        let http_dl = http.clone();
        let url_dl = stream.url.clone();
        let shared_dl = shared.clone();
        tokio::spawn(async move {
            stream::download(
                http_dl,
                url_dl,
                innertube::UA.to_string(),
                shared_dl,
                None,
                Ranging::Query,
            )
            .await;
        });

        // Open decoder on blocking thread (Symphonia / rusty-opus)
        let shared_dec = shared.clone();
        let seek = !at.is_zero();
        let decoder_res = tokio::task::spawn_blocking(move || {
            let mut d = stream::decoder(shared_dec, seek)?;
            if seek {
                rodio::Source::try_seek(&mut d, at).map_err(|e| anyhow::anyhow!("seek: {e}"))?;
            }
            Ok::<_, anyhow::Error>(d)
        })
        .await;

        let decoder = match decoder_res {
            Ok(Ok(d)) => d,
            Ok(Err(e)) => {
                log::error!("decoder build failed: {e:#}");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = weak.upgrade() {
                        if gen_atomic.load(Ordering::SeqCst) == generation {
                            app.global::<crate::Player>().set_status(0);
                        }
                    }
                });
                return;
            }
            Err(e) => {
                log::error!("decoder spawn_blocking join error: {e}");
                return;
            }
        };

        if gen_atomic.load(Ordering::SeqCst) != generation {
            return;
        }

        let dur_secs = stream.duration_ms as f64 / 1000.0;
        if dur_secs > 0.0 {
            if let Some(cur) = current_track.lock().unwrap().as_mut() {
                cur.duration = dur_secs as f32;
            }
        }

        audio.send(AudioCmd::Start {
            generation,
            decoder,
            shared: shared.clone(),
            at,
        });

        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = weak.upgrade() {
                if gen_atomic.load(Ordering::SeqCst) == generation {
                    let player = app.global::<crate::Player>();
                    if dur_secs > 0.0 {
                        player.set_duration(dur_secs as f32);
                    }
                    player.set_status(1); // playing!
                }
            }
        });
    });
}

/// The next queue entry (wrapping), as PlayerBar's next does.
pub fn play_next(app: &App) {
    let next_track = {
        let queue = app.state.queue.lock().unwrap();
        if queue.is_empty() {
            None
        } else {
            let mut idx = app.state.queue_index.lock().unwrap();
            let next_idx = *idx + 1;
            if next_idx < queue.len() {
                *idx = next_idx;
                Some(queue[next_idx].clone())
            } else {
                *idx = 0;
                Some(queue[0].clone())
            }
        }
    };

    if let Some(track) = next_track {
        play_track(app, track.video_id, track.title, track.artist, Duration::ZERO);
    }
}

/// The previous queue entry; wrapping (the button itself rewinds first when
/// more than 3s have played, handled in the callback below).
pub fn play_prev(app: &App) {
    let prev_track = {
        let queue = app.state.queue.lock().unwrap();
        if queue.is_empty() {
            None
        } else {
            let mut idx = app.state.queue_index.lock().unwrap();
            if *idx > 0 {
                *idx -= 1;
                Some(queue[*idx].clone())
            } else {
                Some(queue[0].clone())
            }
        }
    };

    if let Some(track) = prev_track {
        play_track(app, track.video_id, track.title, track.artist, Duration::ZERO);
    }
}

/// Play an arbitrary track (a search row or shelf item): point the queue at
/// it when it's in there, so next/previous continue from its slot.
pub fn play_one_off(app: &App, track: TrackItem) {
    app.state.jump_to(&track.video_id);
    play_track(app, track.video_id, track.title, track.artist, Duration::ZERO);
}

/// The timers kept alive for the app's lifetime; drop them and they stop.
pub struct Timers {
    _progress: Timer,
    _beat: Timer,
    _events: Timer,
}

/// Wire the transport, the window controls, and the timers.
pub fn wire(app: &App, events_rx: crossbeam_channel::Receiver<AudioEvent>) -> Timers {
    wire_transport(app);
    wire_window_controls(app);
    Timers {
        _progress: wire_progress_timer(app),
        _beat: wire_beat_timer(app),
        _events: wire_event_timer(app, events_rx),
    }
}

fn wire_transport(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let player = app_window.global::<crate::Player>();

    // Toggle Play/Pause
    {
        let app = app.clone();
        player.on_toggle_play(move || {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            match player.get_status() {
                1 => {
                    // Playing -> Pause
                    app.audio.send(AudioCmd::Pause);
                    player.set_status(0);
                }
                0 => {
                    // Paused -> Resume or Start
                    if app.audio.status.duration().is_some()
                        && app.state.current_track.lock().unwrap().is_some()
                    {
                        app.audio.send(AudioCmd::Resume);
                        player.set_status(1);
                    } else if let Some(current) = app.state.current_track.lock().unwrap().clone() {
                        play_track(
                            &app,
                            current.video_id,
                            current.title,
                            current.artist,
                            Duration::ZERO,
                        );
                    } else if let Some(first) = app.state.queue.lock().unwrap().first().cloned() {
                        play_track(&app, first.video_id, first.title, first.artist, Duration::ZERO);
                    }
                }
                _ => {}
            }
        });
    }

    // Next
    {
        let app = app.clone();
        player.on_next(move || play_next(&app));
    }

    // Previous: rewind when more than 3s have played, else the earlier track
    {
        let app = app.clone();
        player.on_previous(move || {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            if app.audio.status.position() > Duration::from_secs(3) {
                player.set_elapsed(0.0);
                player.set_progress(0.0);
                app.audio.send(AudioCmd::Seek(Duration::ZERO));
            } else {
                play_prev(&app);
            }
        });
    }

    // Shuffle
    {
        let app = app.clone();
        player.on_toggle_shuffle(move || {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            player.set_shuffle(!player.get_shuffle());
        });
    }

    // Repeat (0 = off, 1 = all, 2 = one)
    {
        let app = app.clone();
        player.on_cycle_repeat(move || {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            player.set_repeat((player.get_repeat() + 1) % 3);
        });
    }

    // Seek
    {
        let app = app.clone();
        player.on_seeked(move |fraction| {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            let target = fraction * player.get_duration();
            player.set_elapsed(target);
            player.set_progress(fraction);
            app.audio.send(AudioCmd::Seek(Duration::from_secs_f32(target)));
        });
    }

    // Volume & Mute
    {
        let app = app.clone();
        player.on_set_volume(move |volume| {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            player.set_volume((volume.clamp(0.0, 1.0) * 100.0).round() / 100.0);
            player.set_muted(false);
            app.audio.send(AudioCmd::Volume(player.get_volume()));
        });
    }

    {
        let app = app.clone();
        player.on_toggle_mute(move || {
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            let new_muted = !player.get_muted();
            player.set_muted(new_muted);
            if new_muted {
                app.audio.send(AudioCmd::Volume(0.0));
            } else {
                app.audio.send(AudioCmd::Volume(player.get_volume()));
            }
        });
    }

    {
        let app = app.clone();
        player.on_wheel_volume(move |delta_y| {
            if delta_y == 0.0 {
                return;
            }
            let Some(app_window) = app.weak.upgrade() else { return };
            let player = app_window.global::<crate::Player>();
            let new_vol = player.get_volume() + if delta_y < 0.0 { 0.05 } else { -0.05 };
            let clamped = new_vol.clamp(0.0, 1.0);
            player.set_volume((clamped * 100.0).round() / 100.0);
            player.set_muted(false);
            app.audio.send(AudioCmd::Volume(player.get_volume()));
        });
    }
}

fn wire_window_controls(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");

    {
        let weak = app.weak.clone();
        app_window.on_minimize(move || {
            if let Some(app) = weak.upgrade() {
                app.window().set_minimized(true);
            }
        });
    }

    {
        let weak = app.weak.clone();
        app_window.on_toggle_maximize(move || {
            let Some(app) = weak.upgrade() else { return };
            let window = app.window();
            window.set_maximized(!window.is_maximized());
            app.global::<crate::Ui>().set_maximized(window.is_maximized());
        });
    }

    {
        let weak = app.weak.clone();
        app_window.on_close_requested(move || {
            if let Some(app) = weak.upgrade() {
                let _ = app.hide();
            }
            let _ = slint::quit_event_loop();
        });
    }
}

/// Lock-free smooth extrapolation from AudioStatus, 10×/s.
fn wire_progress_timer(app: &App) -> Timer {
    let timer = Timer::default();
    let app = app.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(100), move || {
        let Some(app_window) = app.weak.upgrade() else { return };
        let player = app_window.global::<crate::Player>();
        if player.get_status() != 1 {
            return;
        }
        let pos = app.audio.status.position_smooth();
        let secs = pos.as_secs_f64();
        player.set_elapsed(secs as f32);
        let dur = player.get_duration();
        player.set_progress(if dur > 0.0 { (secs as f32) / dur } else { 0.0 });
    });
    timer
}

/// BeatLoader phase driver (~60fps over the 1.9s full cycle).
fn wire_beat_timer(app: &App) -> Timer {
    let timer = Timer::default();
    let app = app.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        let Some(app_window) = app.weak.upgrade() else { return };
        let player = app_window.global::<crate::Player>();
        if player.get_status() < 2 {
            return;
        }
        player.set_beat_phase((player.get_beat_phase() + 0.016 / 1.9) % 1.0);
    });
    timer
}

/// Drain audio events: finished (repeat/queue advance), interrupted (restart
/// at the interruption point), errors.
fn wire_event_timer(app: &App, events_rx: crossbeam_channel::Receiver<AudioEvent>) -> Timer {
    let timer = Timer::default();
    let app = app.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(50), move || {
        while let Ok(ev) = events_rx.try_recv() {
            match ev {
                AudioEvent::Finished { generation } => {
                    if generation != app.state.generation.load(Ordering::SeqCst) {
                        continue;
                    }
                    let repeat = app
                        .weak
                        .upgrade()
                        .map(|a| a.global::<crate::Player>().get_repeat())
                        .unwrap_or(0);
                    if repeat == 2 {
                        // Repeat One: restart current track
                        if let Some(cur) = app.state.current_track.lock().unwrap().clone() {
                            play_track(
                                &app,
                                cur.video_id,
                                cur.title,
                                cur.artist,
                                Duration::ZERO,
                            );
                        }
                    } else {
                        // Advance queue or stop
                        let has_next = {
                            let queue = app.state.queue.lock().unwrap();
                            let idx = *app.state.queue_index.lock().unwrap();
                            !queue.is_empty() && (repeat == 1 || idx + 1 < queue.len())
                        };
                        if has_next {
                            play_next(&app);
                        } else if let Some(app_window) = app.weak.upgrade() {
                            let player = app_window.global::<crate::Player>();
                            player.set_status(0);
                            player.set_elapsed(0.0);
                            player.set_progress(0.0);
                        }
                    }
                }
                AudioEvent::Interrupted {
                    generation,
                    at,
                    reason,
                } => {
                    log::warn!("audio interrupted for gen {generation} at {at:?}: {reason}");
                    if generation == app.state.generation.load(Ordering::SeqCst) {
                        if let Some(cur) = app.state.current_track.lock().unwrap().clone() {
                            play_track(&app, cur.video_id, cur.title, cur.artist, at);
                        }
                    }
                }
                AudioEvent::Error(e) => {
                    log::error!("audio error: {e}");
                    if let Some(app_window) = app.weak.upgrade() {
                        app_window.global::<crate::Player>().set_status(0);
                    }
                }
                _ => {}
            }
        }
    });
    timer
}
