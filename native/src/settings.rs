//! Settings page wiring: managing audio quality choices, motion artwork toggles,
//! download folder inspection, and application engine settings.

use slint::ComponentHandle;
use crate::app::App;
use crate::backend::downloads;

pub fn wire(app: &App) {
    let app_window = app.weak.upgrade().expect("window alive during wiring");
    let settings = app_window.global::<crate::SettingsData>();

    // Set initial values
    let folder = downloads::folder();
    settings.set_download_folder(folder.to_string_lossy().to_string().into());
    settings.set_active_category("playback".into());
    settings.set_audio_quality("High (256 kbps AAC)".into());
    settings.set_animated_art(true);
    settings.set_discord_rpc(true);
    settings.set_cache_size("64 MB".into());
    settings.set_version("BeatFlow Native 0.1.0 (Direct3D 11 / Skia GPU)".into());

    // Switch Category Tab
    {
        let weak = app.weak.clone();
        settings.on_set_category(move |cat| {
            if let Some(app) = weak.upgrade() {
                app.global::<crate::SettingsData>().set_active_category(cat);
            }
        });
    }

    // Set Audio Quality
    {
        let weak = app.weak.clone();
        settings.on_set_audio_quality(move |quality| {
            if let Some(app) = weak.upgrade() {
                app.global::<crate::SettingsData>().set_audio_quality(quality);
            }
        });
    }

    // Toggle Animated Artwork
    {
        let weak = app.weak.clone();
        settings.on_toggle_animated_art(move || {
            if let Some(app) = weak.upgrade() {
                let s = app.global::<crate::SettingsData>();
                let cur = s.get_animated_art();
                s.set_animated_art(!cur);
            }
        });
    }

    // Toggle Discord RPC
    {
        let weak = app.weak.clone();
        settings.on_toggle_discord_rpc(move || {
            if let Some(app) = weak.upgrade() {
                let s = app.global::<crate::SettingsData>();
                let cur = s.get_discord_rpc();
                s.set_discord_rpc(!cur);
            }
        });
    }

    // Open Download Folder in Explorer
    settings.on_open_download_folder(move || {
        downloads::open_folder();
    });

    // Clear Cache
    {
        let app = app.clone();
        let weak = app.weak.clone();
        settings.on_clear_cache(move || {
            app.state.image_cache.lock().unwrap().clear();
            if let Some(app_win) = weak.upgrade() {
                app_win.global::<crate::SettingsData>().set_cache_size("0 MB".into());
            }
        });
    }
}
