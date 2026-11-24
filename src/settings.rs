//! Tray choices that survive a restart: auto sync paused, sync interval.
//! A plain `key=value` file; unknown or broken lines are ignored.

use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::config::{create_private_dir, write_file};

#[derive(Debug, Default, PartialEq)]
pub struct Settings {
    pub paused: bool,
    /// Overrides `--interval` once picked in the tray.
    pub interval: Option<Duration>,
}

/// `%APPDATA%\bwsshd\settings` on Windows, `~/Library/Application
/// Support/bwsshd/settings` on macOS, `$XDG_CONFIG_HOME/bwsshd/settings`
/// (default `~/.config`) elsewhere.
pub fn path(home: &Path) -> PathBuf {
    let env = |v| {
        std::env::var_os(v)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    };
    let base = if cfg!(windows) {
        env("APPDATA").unwrap_or_else(|| home.join("AppData/Roaming"))
    } else if cfg!(target_os = "macos") {
        home.join("Library/Application Support")
    } else {
        env("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"))
    };
    base.join("bwsshd").join("settings")
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        Self::parse(&text)
    }

    fn parse(text: &str) -> Self {
        let mut s = Self::default();
        for (k, v) in text.lines().filter_map(|l| l.split_once('=')) {
            match k.trim() {
                "paused" => s.paused = v.trim() == "true",
                "interval" => {
                    s.interval = humantime::parse_duration(v.trim())
                        .ok()
                        .filter(|d| !d.is_zero())
                }
                _ => {}
            }
        }
        s
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            create_private_dir(dir)?;
        }
        let mut text = format!("paused={}\n", self.paused);
        if let Some(d) = self.interval {
            text += &format!("interval={}\n", humantime::format_duration(d));
        }
        write_file(path, text.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let dir = tempfile::TempDir::new().unwrap();
        let p = dir.path().join("sub/settings");
        assert_eq!(Settings::load(&p), Settings::default());
        let s = Settings {
            paused: true,
            interval: Some(Duration::from_secs(300)),
        };
        s.save(&p).unwrap();
        assert_eq!(Settings::load(&p), s);
    }

    #[test]
    fn ignores_garbage() {
        let s = Settings::parse("junk\npaused=yes\ninterval=0s\nother=1\n");
        assert_eq!(s, Settings::default());
        let s = Settings::parse(" interval = 30s \npaused=true");
        assert_eq!(s.interval, Some(Duration::from_secs(30)));
        assert!(s.paused);
    }
}
