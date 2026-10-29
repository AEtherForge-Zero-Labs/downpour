use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    pub url: String,
    pub mode: String,
    pub format: String,
    pub quality: String,
    pub directory: String,
    pub playlist: bool,
    pub subtitles: bool,
    pub thumbnail: bool,
    pub metadata: bool,
    pub rate_limit: String,
    pub concurrent_fragments: u8,
}

impl Options {
    pub fn args(&self) -> Result<Vec<String>, String> {
        let url = url::Url::parse(self.url.trim()).map_err(|_| "Enter a valid YouTube URL.")?;
        let host = url.host_str().unwrap_or_default();
        if !matches!(url.scheme(), "https" | "http")
            || !(host == "youtu.be" || host == "youtube.com" || host.ends_with(".youtube.com"))
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Use a youtube.com or youtu.be video or playlist URL.".into());
        }
        if !std::path::Path::new(&self.directory).is_absolute() {
            return Err("Choose an absolute download folder path.".into());
        }
        if !(1..=16).contains(&self.concurrent_fragments) {
            return Err("Fragments must be between 1 and 16.".into());
        }
        let mut args: Vec<String> = [
            "--ignore-config",
            "--no-plugin-dirs",
            "--newline",
            "--progress",
            "--no-colors",
            "--no-overwrites",
            "--windows-filenames",
            "--paths",
            &self.directory,
            "--output",
            "%(title).180B [%(id)s].%(ext)s",
            "--concurrent-fragments",
            &self.concurrent_fragments.to_string(),
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        args.push(
            if self.playlist {
                "--yes-playlist"
            } else {
                "--no-playlist"
            }
            .into(),
        );
        if self.mode == "audio" {
            if !["mp3", "m4a", "wav", "flac", "opus"].contains(&self.format.as_str()) {
                return Err("Unsupported audio format.".into());
            }
            if !["0", "128K", "192K", "320K"].contains(&self.quality.as_str()) {
                return Err("Unsupported audio quality.".into());
            }
            args.extend([
                "-f".into(),
                "bestaudio/best".into(),
                "--extract-audio".into(),
                "--audio-format".into(),
                self.format.clone(),
                "--audio-quality".into(),
                self.quality.clone(),
            ]);
        } else if self.mode == "video" {
            if !["mp4", "mkv", "webm"].contains(&self.format.as_str()) {
                return Err("Unsupported video format.".into());
            }
            if !["best", "2160", "1440", "1080", "720", "480"].contains(&self.quality.as_str()) {
                return Err("Unsupported video quality.".into());
            }
            let cap = if self.quality == "best" {
                String::new()
            } else {
                format!("[height<=?{}]", self.quality)
            };
            let selector = if self.format == "mp4" {
                format!("bv*{cap}[ext=mp4]+ba[ext=m4a]/b{cap}[ext=mp4]/bv*{cap}+ba/b{cap}")
            } else if self.format == "webm" {
                format!("bv*{cap}[ext=webm]+ba[ext=webm]/b{cap}[ext=webm]")
            } else {
                format!("bv*{cap}+ba/b{cap}")
            };
            args.extend([
                "-f".into(),
                selector,
                "--merge-output-format".into(),
                self.format.clone(),
                "--remux-video".into(),
                self.format.clone(),
            ]);
        } else {
            return Err("Unsupported download mode.".into());
        }
        if self.subtitles {
            args.extend([
                "--write-subs".into(),
                "--write-auto-subs".into(),
                "--sub-langs".into(),
                "en.*".into(),
                "--sub-format".into(),
                "vtt".into(),
            ]);
        }
        if self.thumbnail {
            args.push("--write-thumbnail".into());
        }
        if self.metadata {
            args.push("--embed-metadata".into());
        }
        let rate = self.rate_limit.trim();
        if !rate.is_empty() {
            let digits = rate.trim_end_matches(['K', 'M']);
            if digits.is_empty()
                || !digits.bytes().all(|b| b.is_ascii_digit())
                || digits.parse::<u64>().unwrap_or(0) == 0
            {
                return Err(
                    "Rate limit must be a positive integer, optionally followed by K or M.".into(),
                );
            }
            args.extend(["--limit-rate".into(), rate.into()]);
        }
        args.extend(["--".into(), self.url.trim().into()]);
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn options() -> Options {
        Options {
            url: "https://youtube.com/watch?v=example".into(),
            mode: "video".into(),
            format: "mp4".into(),
            quality: "1080".into(),
            directory: std::env::temp_dir()
                .join("a folder")
                .to_string_lossy()
                .into(),
            playlist: false,
            subtitles: false,
            thumbnail: false,
            metadata: true,
            rate_limit: "".into(),
            concurrent_fragments: 4,
        }
    }
    #[test]
    fn video_has_height_cap_and_exact_extension() {
        let a = options().args().unwrap();
        assert!(a.iter().any(|s| s.contains("height<=?1080")));
        assert!(a.windows(2).any(|p| p == ["--remux-video", "mp4"]));
        assert_eq!(a[a.len() - 2], "--");
    }
    #[test]
    fn audio_extracts_requested_format() {
        let mut o = options();
        o.mode = "audio".into();
        o.format = "flac".into();
        o.quality = "0".into();
        let a = o.args().unwrap();
        assert!(a.contains(&"--extract-audio".into()));
        assert!(a.windows(2).any(|p| p == ["--audio-format", "flac"]));
    }
    #[test]
    fn rejects_other_hosts_and_flag_injection() {
        for u in [
            "--exec=evil",
            "file:///etc/passwd",
            "https://youtube.com.evil.org/x",
            "https://evil.youtube.com@evil.org/x",
        ] {
            let mut o = options();
            o.url = u.into();
            assert!(o.args().is_err());
        }
        let mut o = options();
        o.rate_limit = "1M --exec foo".into();
        assert!(o.args().is_err());
        o = options();
        o.format = "--exec".into();
        assert!(o.args().is_err());
    }
    #[test]
    fn flags_and_paths_are_individual_arguments() {
        let mut o = options();
        o.playlist = true;
        o.subtitles = true;
        o.rate_limit = "2M".into();
        let a = o.args().unwrap();
        assert!(a.contains(&o.directory));
        assert!(a.contains(&"--yes-playlist".into()));
        assert!(a.windows(2).any(|p| p == ["--limit-rate", "2M"]));
    }
}
