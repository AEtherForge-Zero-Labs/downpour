//! Embedded yt-dlp Android backend. No desktop executables are run on mobile.
use app_core::Options;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{thread, time::Duration};
use tauri::{
    Manager, State, Wry,
    ipc::Channel,
    plugin::{PluginHandle, TauriPlugin},
};

pub struct AndroidDownloader(PluginHandle<Wry>);

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("android-downloader")
        .setup(|app, api| {
            let plugin =
                api.register_android_plugin("com.aetherforge.downpour", "DownloaderPlugin")?;
            app.manage(AndroidDownloader(plugin));
            Ok(())
        })
        .build()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependencies {
    yt_dlp: Option<String>,
    ffmpeg: bool,
    directory: String,
}

#[tauri::command]
pub fn dependencies() -> Dependencies {
    Dependencies {
        yt_dlp: Some("embedded (Android)".into()),
        ffmpeg: false,
        // Logical MediaStore destination, not a directly writable filesystem path.
        directory: "/Downloads/Downpour".into(),
    }
}

#[tauri::command]
pub fn latest_release() -> Result<Value, String> {
    Err("Android bundles yt-dlp; update it by upgrading Downpour.".into())
}

#[tauri::command]
pub fn preview(_options: Options) -> Result<Vec<String>, String> {
    Err("The Android engine runs in-process, not as a shell command.".into())
}

#[tauri::command]
pub fn choose_directory() -> Option<String> {
    None
}

#[tauri::command]
pub fn open_directory(_directory: String) -> Result<(), String> {
    Err("Open Android Files and select Downloads / Downpour.".into())
}

#[tauri::command]
pub fn cancel_download() -> Result<(), String> {
    Err("Cancellation is not yet supported by the embedded Android engine.".into())
}

#[derive(Serialize)]
pub struct DownloadEvent {
    kind: &'static str,
    text: String,
}

fn send(events: &Channel<DownloadEvent>, kind: &'static str, text: impl Into<String>) {
    let _ = events.send(DownloadEvent {
        kind,
        text: text.into(),
    });
}

#[derive(Deserialize)]
struct Progress {
    status: String,
    progress: f32,
    message: String,
}

#[tauri::command]
pub async fn start_download(
    options: Options,
    events: Channel<DownloadEvent>,
    state: State<'_, AndroidDownloader>,
) -> Result<(), String> {
    // Reuse the desktop URL and option validation before crossing into Kotlin.
    options.args()?;
    if options.mode != "video" || options.format != "mp4" {
        return Err("Android currently supports MP4 video with audio only.".into());
    }
    if !matches!(options.quality.as_str(), "best" | "720" | "480") {
        return Err("Android quality must be Best, 720p, or 480p.".into());
    }
    if options.subtitles || options.thumbnail || options.metadata || !options.rate_limit.is_empty()
    {
        return Err(
            "Subtitles, thumbnails, metadata embedding, and rate limits require desktop.".into(),
        );
    }

    let handle = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        handle
            .run_mobile_plugin::<Value>(
                "start",
                json!({ "url": options.url, "quality": options.quality, "playlist": options.playlist }),
            )
            .map_err(|e| e.to_string())?;
        let mut last_percent: i32 = -1;
        loop {
            let p: Progress = handle
                .run_mobile_plugin("status", json!({}))
                .map_err(|e| e.to_string())?;
            let percent = p.progress.clamp(0.0, 100.0) as i32;
            if percent != last_percent {
                send(&events, "log", format!("[download] {:.1}%", p.progress));
                last_percent = percent;
            }
            match p.status.as_str() {
                "complete" => {
                    send(&events, "complete", p.message);
                    return Ok(());
                }
                "error" => {
                    send(&events, "error", p.message);
                    return Ok(());
                }
                "running" => {}
                other => return Err(format!("Unexpected Android download state: {other}")),
            }
            thread::sleep(Duration::from_millis(350));
        }
    })
    .await
    .map_err(|e| e.to_string())?
}
