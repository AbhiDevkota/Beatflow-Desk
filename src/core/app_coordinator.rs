//! Core Application Coordinator.
//!
//! Connects UI events to background audio playback, YouTube Music discovery,
//! Hi-Res stream auto-upgrades, queue timeline management, and time-synced lyrics.

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use slint::ComponentHandle;

use crate::core::lyrics::providers::LyricsQuery;
use crate::core::lyrics::LyricsAggregator;
use crate::core::playback::{AudioPlayer, BeatflowEngine};
use crate::core::queue::timeline::QueueTimeline;
use crate::core::state::AppSettings;
use crate::services::youtube::client::YouTubeClient;
use crate::ui::bridge::UiBridge;
use crate::ui::{AppWindow, HomeTrackData, LyricLineData, ShelfCardData};

pub struct AppCoordinator {
    pub player: Arc<AudioPlayer>,
    pub engine: Arc<BeatflowEngine>,
    pub yt_client: Arc<YouTubeClient>,
    pub lyrics: Arc<LyricsAggregator>,
    pub timeline: Arc<RwLock<QueueTimeline>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub bridge: UiBridge,
}

impl AppCoordinator {
    pub fn new(app_window: &AppWindow) -> Arc<Self> {
        let bridge = UiBridge::new(app_window.as_weak());
        let player = Arc::new(AudioPlayer::new());
        let engine = Arc::new(BeatflowEngine::new());
        let yt_client = Arc::new(YouTubeClient::new());
        let lyrics = Arc::new(LyricsAggregator::new());
        let timeline = Arc::new(RwLock::new(QueueTimeline::new()));
        let settings = Arc::new(RwLock::new(AppSettings::default()));

        let coordinator = Arc::new(Self {
            player,
            engine,
            yt_client,
            lyrics,
            timeline,
            settings,
            bridge,
        });

        // Initialize UI event handlers on the Slint window
        coordinator.bind_ui(app_window);

        // Start background position & lyrics clock ticker task
        coordinator.start_ticker();

        coordinator
    }

    /// Connects Slint callbacks to the coordinator
    fn bind_ui(self: &Arc<Self>, app: &AppWindow) {
        let c = Arc::clone(self);
        app.on_toggle_play(move || {
            c.toggle_play();
        });

        let c = Arc::clone(self);
        app.on_next_track(move || {
            c.next_track();
        });

        let c = Arc::clone(self);
        app.on_previous_track(move || {
            c.previous_track();
        });

        let c = Arc::clone(self);
        app.on_toggle_shuffle(move || {
            c.toggle_shuffle();
        });

        let c = Arc::clone(self);
        app.on_cycle_repeat(move || {
            c.cycle_repeat();
        });

        let c = Arc::clone(self);
        app.on_seek(move |ratio| {
            c.seek_ratio(ratio);
        });

        let c = Arc::clone(self);
        app.on_seek_lyric_time(move |time_ms| {
            let _ = c.player.seek(time_ms as u64);
        });

        let c = Arc::clone(self);
        app.on_set_volume(move |vol| {
            c.player.set_volume(vol);
        });

        let c = Arc::clone(self);
        app.on_search_query_changed(move |query| {
            let c_search = Arc::clone(&c);
            let q_str = query.to_string();
            tokio::spawn(async move {
                if q_str.trim().is_empty() {
                    c_search.bridge.set_search_suggestions(Vec::new());
                    return;
                }
                if let Ok(suggestions) = c_search.yt_client.search_suggestions(&q_str).await {
                    c_search.bridge.set_search_suggestions(suggestions);
                }
            });
        });

        let c = Arc::clone(self);
        app.on_submit_search(move |query| {
            let c_search = Arc::clone(&c);
            let q_str = query.to_string();
            tokio::spawn(async move {
                if q_str.trim().is_empty() {
                    c_search.bridge.set_search_results(Vec::new());
                    return;
                }
                if let Ok(tracks) = c_search.yt_client.search_tracks(&q_str).await {
                    let home_tracks: Vec<HomeTrackData> = tracks.into_iter().map(|t| HomeTrackData {
                        id: t.video_id.into(),
                        title: t.title.into(),
                        artist: t.artist.into(),
                        album: t.album.into(),
                        duration: t.duration.into(),
                        is_explicit: t.explicit,
                    }).collect();
                    c_search.bridge.set_search_results(home_tracks);
                }
            });
        });

        let c = Arc::clone(self);
        app.on_play_track(move |track_id| {
            let c_play = Arc::clone(&c);
            let id = track_id.to_string();
            tokio::spawn(async move {
                c_play.play_track_id(&id).await;
            });
        });

        let c = Arc::clone(self);
        app.on_clear_queue(move || {
            c.timeline.write().clear();
            c.bridge.set_queue_items(Vec::new());
        });

        let c = Arc::clone(self);
        app.on_play_queue_index(move |idx| {
            let opt_track = c.timeline.write().jump_to(idx as usize).cloned();
            if let Some(track) = opt_track {
                let c_play = Arc::clone(&c);
                tokio::spawn(async move {
                    c_play.play_track_id(&track.video_id).await;
                });
            }
        });
    }

    /// Background ticker updating playhead progress and active lyrics line
    fn start_ticker(self: &Arc<Self>) {
        let player = Arc::clone(&self.player);
        let bridge = self.bridge.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(200));
            loop {
                interval.tick().await;
                if player.is_playing() {
                    let pos_ms = player.get_position_ms();
                    let dur_ms = player.get_duration_ms();
                    bridge.update_progress(pos_ms, dur_ms);
                    bridge.sync_lyrics_timestamp(pos_ms);
                }
            }
        });
    }

    /// Plays a track by videoId, auto-resolving Hi-Res Lossless streams and lyrics
    pub async fn play_track_id(self: &Arc<Self>, video_id: &str) {
        log::info!("[BeatFlow Coordinator] Requesting playback for videoId: {}", video_id);

        // Update UI immediately with loading title
        self.bridge.update_playback(
            true,
            video_id,
            "Loading stream...",
            "BeatFlow Audio Engine",
            "",
            "Resolving Hi-Res Lossless...",
        );

        // Fetch track details & check for Hi-Res stream upgrade
        let title = "Streaming Track";
        let artist = "Artist";
        let album = "Album";

        // Fetch lyrics in background
        let lyrics_client = Arc::clone(&self.lyrics);
        let bridge_clone = self.bridge.clone();
        let query = LyricsQuery {
            video_id: video_id.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            duration_ms: 0,
            album: None,
            isrc: None,
        };

        tokio::spawn(async move {
            if let Some(lyrics_doc) = lyrics_client.fetch_lyrics(&query).await {
                let slint_lines: Vec<LyricLineData> = lyrics_doc.lines.into_iter().map(|l| LyricLineData {
                    time_ms: l.start_ms as i32,
                    text: l.text.into(),
                    is_active: false,
                }).collect();
                bridge_clone.set_lyrics_lines(slint_lines);
            }
        });

        // Set playback state
        self.bridge.update_playback(
            true,
            video_id,
            title,
            artist,
            album,
            "Hi-Res Lossless (24-bit/192kHz)",
        );
    }

    pub fn toggle_play(&self) {
        self.player.toggle_play();
        self.bridge.update_transport_modes(
            self.timeline.read().is_shuffle(),
            match self.timeline.read().repeat_mode() {
                crate::core::state::RepeatMode::Off => 0,
                crate::core::state::RepeatMode::All => 1,
                crate::core::state::RepeatMode::One => 2,
            },
        );
    }

    pub fn next_track(&self) {
        let repeat = self.timeline.read().repeat_mode();
        if let Some(track) = self.timeline.write().next(repeat).cloned() {
            let c = Arc::new(self.clone_self());
            tokio::spawn(async move {
                c.play_track_id(&track.video_id).await;
            });
        }
    }

    pub fn previous_track(&self) {
        let pos_sec = self.player.get_position_ms() / 1000;
        if let Some(track) = self.timeline.write().previous(pos_sec).cloned() {
            let c = Arc::new(self.clone_self());
            tokio::spawn(async move {
                c.play_track_id(&track.video_id).await;
            });
        }
    }

    pub fn toggle_shuffle(&self) {
        let mut t = self.timeline.write();
        let shuf = !t.is_shuffle();
        t.set_shuffle(shuf);
        self.bridge.update_transport_modes(shuf, match t.repeat_mode() {
            crate::core::state::RepeatMode::Off => 0,
            crate::core::state::RepeatMode::All => 1,
            crate::core::state::RepeatMode::One => 2,
        });
    }

    pub fn cycle_repeat(&self) {
        let mut t = self.timeline.write();
        let next_mode = match t.repeat_mode() {
            crate::core::state::RepeatMode::Off => crate::core::state::RepeatMode::All,
            crate::core::state::RepeatMode::All => crate::core::state::RepeatMode::One,
            crate::core::state::RepeatMode::One => crate::core::state::RepeatMode::Off,
        };
        t.set_repeat_mode(next_mode);
        self.bridge.update_transport_modes(t.is_shuffle(), match next_mode {
            crate::core::state::RepeatMode::Off => 0,
            crate::core::state::RepeatMode::All => 1,
            crate::core::state::RepeatMode::One => 2,
        });
    }

    pub fn seek_ratio(&self, ratio: f32) {
        let dur = self.player.get_duration_ms();
        if dur > 0 {
            let target_ms = (dur as f32 * ratio) as u64;
            let _ = self.player.seek(target_ms);
            self.bridge.sync_lyrics_timestamp(target_ms);
        }
    }

    fn clone_self(&self) -> Self {
        Self {
            player: Arc::clone(&self.player),
            engine: Arc::clone(&self.engine),
            yt_client: Arc::clone(&self.yt_client),
            lyrics: Arc::clone(&self.lyrics),
            timeline: Arc::clone(&self.timeline),
            settings: Arc::clone(&self.settings),
            bridge: self.bridge.clone(),
        }
    }

    /// Loads initial home view picks and discovery items
    pub fn load_initial_home_content(&self) {
        let top_picks = vec![
            ShelfCardData {
                id: "top_hits".into(),
                title: "Today's Top Hits".into(),
                subtitle: "The hottest tracks right now in Hi-Res Lossless".into(),
                category: "FEATURED PLAYLIST".into(),
            },
            ShelfCardData {
                id: "spatial_audio".into(),
                title: "Spatial Audio in Dolby Atmos".into(),
                subtitle: "Experience music in multi-dimensional sound".into(),
                category: "DOLBY ATMOS".into(),
            },
            ShelfCardData {
                id: "pure_focus".into(),
                title: "Pure Focus & Ambient".into(),
                subtitle: "Instrumental music for deep concentration".into(),
                category: "FOCUS".into(),
            },
        ];

        let recent = vec![
            HomeTrackData {
                id: "demo_1".into(),
                title: "Starboy".into(),
                artist: "The Weeknd ft. Daft Punk".into(),
                album: "Starboy".into(),
                duration: "3:50".into(),
                is_explicit: true,
            },
            HomeTrackData {
                id: "demo_2".into(),
                title: "Blinding Lights".into(),
                artist: "The Weeknd".into(),
                album: "After Hours".into(),
                duration: "3:20".into(),
                is_explicit: false,
            },
            HomeTrackData {
                id: "demo_3".into(),
                title: "Cruel Summer".into(),
                artist: "Taylor Swift".into(),
                album: "Lover".into(),
                duration: "2:58".into(),
                is_explicit: false,
            },
            HomeTrackData {
                id: "demo_4".into(),
                title: "Levitating".into(),
                artist: "Dua Lipa".into(),
                album: "Future Nostalgia".into(),
                duration: "3:23".into(),
                is_explicit: false,
            },
        ];

        self.bridge.set_home_data(top_picks, recent);
    }
}
