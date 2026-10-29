# Downpour

A desktop and Android YouTube downloader with a focused interface for [yt-dlp](https://github.com/yt-dlp/yt-dlp), built from [NeonSupernova/tauri-app](https://github.com/NeonSupernova/tauri-app) using Tauri 2, Svelte 5, TypeScript, and Rust.

## Download

Get the macOS, Windows, or Linux installer from [Releases](https://github.com/AEtherForge-Zero-Labs/downpour/releases). GitHub Actions builds installers when the version in `package.json` changes, including the first publication. Builds are unsigned and not notarized.

## Features

- Video with audio: MP4, MKV, WebM; best available or a resolution cap from 480p to 4K.
- Audio extraction: MP3, M4A, WAV, FLAC, Opus; best quality or a selected bitrate. Source quality limits the result.
- Choose a destination folder, download a playlist, embed metadata, save English subtitles and thumbnails.
- Bandwidth limits, parallel fragments, and a preview of the generated arguments.
- Live progress, processing status, activity logs, cancellation, and completed downloads for the current session.
- Saves preferences locally, without keeping pasted URLs in persistent storage.
- Checks the installed yt-dlp version against GitHub's latest stable release on launch and on demand.

The latest stable release verified on **October 29, 2026** is [yt-dlp 2026.10.26](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.10.26). This is a reference, not a pinned engine: Downpour uses your installed yt-dlp and checks for newer releases. Update it using the package manager or installer used to install it, then click **Refresh tools**.

## Install the download tools

Downpour requires **yt-dlp** and **FFmpeg** installed separately. A [supported JavaScript runtime](https://github.com/yt-dlp/yt-dlp#dependencies), preferably Deno, is recommended for YouTube. The release checker uses `curl`, included on macOS and current Windows and available through Linux package managers.

macOS:

```sh
brew install yt-dlp ffmpeg deno
# Later:
brew upgrade yt-dlp
```

Windows:

```powershell
winget install yt-dlp.yt-dlp
winget install Gyan.FFmpeg
winget install DenoLand.Deno
# Later:
winget upgrade yt-dlp.yt-dlp
```

On Linux, install yt-dlp, FFmpeg, curl, and optionally Deno through your preferred package manager or the official installation instructions. Distribution packages may lag upstream. The folder picker uses Zenity or KDialog; you can always enter an absolute path manually.

Restart Downpour after installing tools if your PATH changed. On macOS it also searches Homebrew directories and `~/.local/bin`. On Windows it also searches the WinGet links directory.

Paste a YouTube video or playlist URL, choose the mode, format, quality, and folder, then select **Start download**. Enable **Download entire playlist** explicitly to download more than the selected video. Downloads skip existing output files. Repeating a cancelled download can resume its partial files. FFmpeg may need time to process files after the progress bar reaches 100%.

Download only content you own or have permission to save. Site availability, authentication requirements, and format availability are controlled by YouTube and yt-dlp. DRM circumvention is not provided.

## Android (experimental)

Android uses **embedded yt-dlp**, not the desktop subprocess backend. It currently supports **MP4 video with audio**, at Best, up to 720p, or up to 480p. Without FFmpeg, Android downloads an already-muxed single video/audio stream, so availability and quality differ from the desktop edition. Audio extraction, conversion, embedded metadata, thumbnails, subtitles, bandwidth limits, cancellation, and background downloads are not supported in this initial version. Keep the app open during downloads.

Completed videos are saved to **Files → Downloads → Downpour** using Android's scoped storage. The target is **Android 10+ on ARM64**. YouTube extraction may fail depending on site conditions or the bundled yt-dlp version.

The [Android APK workflow](.github/workflows/android.yml) builds a **debug APK** as a downloadable GitHub Actions artifact. It is not a Play Store release or a production-signed installer. The native bridge is maintained at `src-tauri/android/DownloaderPlugin.kt` and copied to Tauri's generated Android project by `scripts/prepare-android.mjs`. The embedded runtime uses [yt-dlp-android](https://github.com/ffmpegkit-maintained/yt-dlp-android) v2.0.2; its bundled components retain their upstream licenses.

Install the [Tauri Android prerequisites](https://v2.tauri.app/start/prerequisites/) (Android SDK and NDK, Java 17, Rust ARM64 target, and cargo-ndk):

```sh
npm install
npm run android:init
npm run android:prepare
npm run android:dev
# Or build a debug APK:
npm run android:apk
```

Repeat `npm run android:prepare` if `tauri android init` regenerates the Android project. **This Android implementation is unverified on a physical device**; test before distribution.

## Development

Requires Node.js 22+, Rust stable, and the [Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
npm install
npm run tdev
```

```sh
npm run check
npm run build
cargo test -p app-core --locked
cargo check -p app --locked
npm run tbuild
```

`npm run dev` starts a browser preview. A browser cannot run local yt-dlp or save downloads through this app; download controls are disabled and the preview explains how to use the desktop app.

The template's native Rust core is used to validate options and construct argument vectors. It has been adapted to a desktop downloader with a distinct experimental Android backend; unused WASM demos are omitted. Frontend dependency versions are pinned to the versions used for local validation. Install with npm; no frontend lockfile is included.

## Process and security design

Commands use argument vectors rather than a shell. URLs are restricted to HTTP(S) YouTube hosts; modes, formats, quality, fragment counts, paths, and bandwidth limits are validated in Rust. Arbitrary shell arguments, yt-dlp configuration files, and plugin directories are not loaded. Downloads run one job at a time and cancellation stops the process group on Unix or the process tree on Windows, including FFmpeg. Closing the window also stops the active job. The release checker only reads GitHub's official API; it does not modify or self-update the installed executable.

## Publishing

`.github/workflows/android.yml` builds an ARM64 Android debug APK artifact on push and pull requests. `.github/workflows/ci.yml` checks the frontend, Rust core tests, and formatting. `.github/workflows/release.yml` builds macOS app/DMG, Windows NSIS, and Linux DEB/AppImage installers and publishes a GitHub release. To release again, bump `package.json` and the workspace version in `Cargo.toml`, update `Cargo.lock`, and push to `main`. The release workflow can also be run manually.

MIT licensed. Template attribution: NeonSupernova/tauri-app. yt-dlp and FFmpeg are separate projects with their own licenses and are not bundled.
