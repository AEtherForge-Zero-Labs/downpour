# Downpour

A desktop YouTube downloader with a focused interface for [yt-dlp](https://github.com/yt-dlp/yt-dlp), built from [NeonSupernova/tauri-app](https://github.com/NeonSupernova/tauri-app) using Tauri 2, Svelte 5, TypeScript, and Rust.

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

The template's native Rust core is used to validate options and construct argument vectors. It has been adapted to a desktop-only downloader; unused WASM demos and mobile targets are omitted. Frontend dependency versions are pinned to the versions used for local validation. Install with npm; no frontend lockfile is included.

## Process and security design

Commands use argument vectors rather than a shell. URLs are restricted to HTTP(S) YouTube hosts; modes, formats, quality, fragment counts, paths, and bandwidth limits are validated in Rust. Arbitrary shell arguments, yt-dlp configuration files, and plugin directories are not loaded. Downloads run one job at a time and cancellation stops the process group on Unix or the process tree on Windows, including FFmpeg. Closing the window also stops the active job. The release checker only reads GitHub's official API; it does not modify or self-update the installed executable.

## Publishing

`.github/workflows/ci.yml` checks the frontend, Rust core tests, and formatting. `.github/workflows/release.yml` builds macOS app/DMG, Windows NSIS, and Linux DEB/AppImage installers and publishes a GitHub release. To release again, bump `package.json` and the workspace version in `Cargo.toml`, update `Cargo.lock`, and push to `main`. The release workflow can also be run manually.

MIT licensed. Template attribution: NeonSupernova/tauri-app. yt-dlp and FFmpeg are separate projects with their own licenses and are not bundled.
