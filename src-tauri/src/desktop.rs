use app_core::Options;
use serde::Serialize;
use std::time::Duration;
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{Manager, State, ipc::Channel};

#[derive(Default)]
struct Downloads {
    active: Mutex<Option<Arc<Job>>>,
}
struct Job {
    child: Mutex<Child>,
    cancelled: AtomicBool,
}
#[derive(Clone, Serialize)]
struct DownloadEvent {
    kind: &'static str,
    text: String,
}
fn send(events: &Channel<DownloadEvent>, kind: &'static str, text: impl Into<String>) {
    let _ = events.send(DownloadEvent {
        kind,
        text: text.into(),
    });
}

// GUI apps on macOS do not inherit the interactive shell's PATH.
fn executable(name: &str) -> PathBuf {
    let filename = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|v| std::env::split_paths(&v).collect())
        .unwrap_or_default();
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"].map(PathBuf::from));
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        dirs.push(PathBuf::from(home).join(".local/bin"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("Microsoft/WinGet/Links"));
    }
    dirs.into_iter()
        .map(|dir| dir.join(&filename))
        .find(|p| p.is_file())
        .unwrap_or_else(|| filename.into())
}
fn command(name: &str) -> Command {
    let mut cmd = Command::new(executable(name));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|v| std::env::split_paths(&v).collect())
        .unwrap_or_default();
    dirs.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"].map(PathBuf::from));
    if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
        let home = PathBuf::from(home);
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".deno/bin"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("Microsoft/WinGet/Links"));
    }
    if let Ok(path) = std::env::join_paths(dirs) {
        cmd.env("PATH", path);
    }
    cmd
}
fn version(name: &str, arg: &str) -> Option<String> {
    command(name)
        .arg(arg)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Dependencies {
    yt_dlp: Option<String>,
    ffmpeg: bool,
    directory: String,
}
#[tauri::command]
async fn dependencies(app: tauri::AppHandle) -> Result<Dependencies, String> {
    tauri::async_runtime::spawn_blocking(move || Dependencies {
        yt_dlp: version("yt-dlp", "--version"),
        ffmpeg: version("ffmpeg", "-version").is_some(),
        directory: app
            .path()
            .download_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .to_string_lossy()
            .into(),
    })
    .await
    .map_err(|e| e.to_string())
}
#[derive(Serialize)]
struct Release {
    version: String,
    url: String,
}
#[tauri::command]
async fn latest_release() -> Result<Release, String> {
    let value: serde_json::Value =
        tauri::async_runtime::spawn_blocking(|| -> Result<serde_json::Value, String> {
            let response = command("curl")
                .args([
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--max-time",
                    "15",
                    "--proto",
                    "=https",
                    "--user-agent",
                    "Downpour/0.1.0",
                    "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest",
                ])
                .output()
                .map_err(|e| format!("Could not run curl to check GitHub: {e}"))?;
            if !response.status.success() {
                return Err(format!(
                    "Could not check GitHub: {}",
                    String::from_utf8_lossy(&response.stderr)
                ));
            }
            let body = String::from_utf8(response.stdout).map_err(|e| e.to_string())?;
            serde_json::from_str(&body).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())??;
    let version = value["tag_name"]
        .as_str()
        .ok_or("GitHub returned no version.")?
        .to_string();
    Ok(Release {
        url: format!("https://github.com/yt-dlp/yt-dlp/releases/tag/{version}"),
        version,
    })
}
#[tauri::command]
fn preview(options: Options) -> Result<Vec<String>, String> {
    options.args()
}
#[tauri::command]
async fn choose_directory() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(target_os="macos")]
        let output=Command::new("osascript").args(["-e", "POSIX path of (choose folder with prompt \"Save downloads to…\")"]).output();
        #[cfg(target_os="windows")]
        let output=command("powershell").args(["-NoProfile", "-STA", "-Command", "Add-Type -AssemblyName System.Windows.Forms; $picker = New-Object System.Windows.Forms.FolderBrowserDialog; if ($picker.ShowDialog() -eq 'OK') { [Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Output $picker.SelectedPath }"]).output();
        #[cfg(target_os="linux")]
        let output=Command::new("zenity").args(["--file-selection", "--directory", "--title=Save downloads to…"]).output().or_else(|_| Command::new("kdialog").args(["--getexistingdirectory", "."]).output());
        let output=output.map_err(|e| format!("Could not open a folder picker; enter the folder path manually. {e}"))?;
        if !output.status.success() { return Ok(None); }
        let path=String::from_utf8_lossy(&output.stdout).trim().to_owned();
        Ok(if path.is_empty() { None } else { Some(path) })
    }).await.map_err(|e| e.to_string())?
}
#[tauri::command]
fn open_directory(directory: String) -> Result<(), String> {
    if !Path::new(&directory).is_dir() {
        return Err("Download folder does not exist.".into());
    }
    #[cfg(target_os = "macos")]
    let mut cmd = Command::new("open");
    #[cfg(target_os = "windows")]
    let mut cmd = Command::new("explorer");
    #[cfg(target_os = "linux")]
    let mut cmd = Command::new("xdg-open");
    cmd.arg(directory)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
fn stop(state: &Downloads) -> Result<(), String> {
    let active = state.active.lock().map_err(|e| e.to_string())?;
    let job = active.as_ref().ok_or("No active download.")?;
    job.cancelled.store(true, Ordering::SeqCst);
    let mut child = job.child.lock().map_err(|e| e.to_string())?;
    // Stop FFmpeg as well as yt-dlp, so cancellation leaves no child process running.
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .args(["-TERM", "--", &format!("-{}", child.id())])
            .status();
    }
    #[cfg(windows)]
    {
        let _ = command("taskkill")
            .args(["/PID", &child.id().to_string(), "/T", "/F"])
            .status();
    }
    match child.try_wait().map_err(|e| e.to_string())? {
        Some(_) => Ok(()),
        None => child.kill().map_err(|e| e.to_string()),
    }
}
#[tauri::command]
fn cancel_download(state: State<'_, Downloads>) -> Result<(), String> {
    stop(&state)
}

#[tauri::command]
async fn start_download(
    options: Options,
    events: Channel<DownloadEvent>,
    state: State<'_, Downloads>,
) -> Result<(), String> {
    options.args()?;
    if version("ffmpeg", "-version").is_none() {
        return Err(
            "Install FFmpeg before downloading; it is required for merging and converting formats."
                .into(),
        );
    }
    std::fs::create_dir_all(&options.directory)
        .map_err(|e| format!("Cannot use download folder: {e}"))?;
    let (job, readers) = {
        let mut active = state.active.lock().map_err(|e| e.to_string())?;
        if active.is_some() {
            return Err("A download is already running.".into());
        }
        let ffmpeg = executable("ffmpeg");
        // Insert FFmpeg location before '--' (the final URL separator).
        let mut actual = options.args()?;
        let pos = actual.len() - 2;
        actual.splice(
            pos..pos,
            ["--ffmpeg-location".into(), ffmpeg.to_string_lossy().into()],
        );
        let mut cmd = command("yt-dlp");
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        let mut child = cmd
            .args(actual)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Could not start yt-dlp. Install it and restart Downpour. {e}"))?;
        let stdout = child.stdout.take().ok_or("Missing stdout")?;
        let stderr = child.stderr.take().ok_or("Missing stderr")?;
        let mut readers = Vec::new();
        for pipe in [
            Box::new(stdout) as Box<dyn std::io::Read + Send>,
            Box::new(stderr),
        ] {
            let events = events.clone();
            readers.push(std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines() {
                    match line {
                        Ok(text) => send(&events, "log", text),
                        Err(_) => break,
                    }
                }
            }));
        }
        let job = Arc::new(Job {
            child: Mutex::new(child),
            cancelled: AtomicBool::new(false),
        });
        *active = Some(job.clone());
        (job, readers)
    };
    let result = tauri::async_runtime::spawn_blocking({
        let job = job.clone();
        move || loop {
            match job
                .child
                .lock()
                .map_err(|e| e.to_string())?
                .try_wait()
                .map_err(|e| e.to_string())?
            {
                Some(status) => break Ok(status),
                None => std::thread::sleep(Duration::from_millis(120)),
            }
        }
    })
    .await
    .map_err(|e| e.to_string())
    .and_then(|r| r);
    for reader in readers {
        let _ = reader.join();
    }
    *state.active.lock().map_err(|e| e.to_string())? = None;
    if job.cancelled.load(Ordering::SeqCst) {
        send(
            &events,
            "cancelled",
            "Download cancelled. Partial files can be resumed by starting again.",
        );
    } else {
        match result {
            Ok(s) if s.success() => send(
                &events,
                "complete",
                "Download complete. Your files are ready.",
            ),
            Ok(s) => send(
                &events,
                "error",
                format!("yt-dlp exited with {s}. See the log for details."),
            ),
            Err(e) => send(&events, "error", e),
        }
    }
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .manage(Downloads::default())
        .invoke_handler(tauri::generate_handler![
            dependencies,
            latest_release,
            preview,
            choose_directory,
            open_directory,
            start_download,
            cancel_download
        ])
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::Destroyed) {
                let state = window.state::<Downloads>();
                let _ = stop(&state);
            }
        })
        .run(tauri::generate_context!())
        .expect("error running Downpour");
}
