//! Notification area icon (`--tray`): shows when the agent was last read,
//! forces a resync, quits. Windows and macOS go through tray-icon; Linux talks
//! StatusNotifierItem over D-Bus with ksni, which keeps the musl build static.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[cfg(any(windows, target_os = "macos"))]
#[path = "desktop.rs"]
mod imp;
#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

/// What the watch loop reports after each tick.
pub enum Status {
    /// The agent answered and the config matches it.
    Synced(Instant),
    /// No agent endpoint answers (Bitwarden closed, locked or starting), with
    /// the time of the last successful sync.
    Absent(Option<Instant>),
    Failed(String),
}

pub type Notify = Box<dyn Fn(Status) + Send>;

/// Shows the icon and runs `watch` with a resync trigger and a status sink.
/// Never returns unless the icon cannot be created.
#[cfg(any(windows, target_os = "macos", target_os = "linux"))]
pub fn run<F>(out: PathBuf, watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<()>, Notify) + Send + 'static,
{
    imp::run(out, watch)
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn run<F>(_out: PathBuf, _watch: F) -> Result<(), String>
where
    F: FnOnce(Receiver<()>, Notify) + Send + 'static,
{
    Err("--tray is not supported on this platform".into())
}

pub const WAITING: &str = "Waiting for the first sync";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Empty when built outside a git checkout (crates.io).
pub const COMMIT: &str = env!("BWSSHD_COMMIT");
pub const COMMIT_DATE: &str = env!("BWSSHD_COMMIT_DATE");

/// `v0.2.0 · 1a2b3c4d5e · 2026-11-24`, without the parts the build lacks.
pub fn version_line() -> String {
    [format!("v{VERSION}"), COMMIT.into(), COMMIT_DATE.into()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

/// Opens the generated config in a text editor.
pub fn open_config(path: &Path) {
    let mut cmd = if cfg!(windows) {
        Command::new("notepad.exe")
    } else if cfg!(target_os = "macos") {
        let mut c = Command::new("open");
        c.arg("-t");
        c
    } else {
        Command::new("xdg-open")
    };
    if let Err(e) = cmd.arg(path).spawn() {
        info!("open {}: {e}", path.display());
    }
}

/// The status line shown at the top of the menu.
pub fn label(status: &Status, now: Instant) -> String {
    match status {
        Status::Synced(t) => format!("Last sync: {}", ago(now - *t)),
        Status::Absent(Some(t)) => {
            format!("Bitwarden agent not reachable, last sync {}", ago(now - *t))
        }
        Status::Absent(None) => "Bitwarden agent not reachable".into(),
        Status::Failed(e) => format!("Sync failed: {e}"),
    }
}

fn ago(d: Duration) -> String {
    let s = d.as_secs();
    if s < 5 {
        return "just now".into();
    }
    // Seconds are noise past the first minute.
    let s = if s >= 60 { s - s % 60 } else { s };
    format!("{} ago", humantime::format_duration(Duration::from_secs(s)))
}

/// The site logo, an isometric cube in three teal tones, as RGBA.
pub fn icon_rgba(size: u32) -> Vec<u8> {
    const TOP: [u8; 3] = [0x4f, 0xe3, 0xc8];
    const LEFT: [u8; 3] = [0x25, 0x8c, 0x7b];
    const RIGHT: [u8; 3] = [0x33, 0xae, 0x99];
    let c = size as f32 / 2.0;
    let r = c - 1.0;
    let half_w = r * 3f32.sqrt() / 2.0;
    let mut px = Vec::with_capacity((size * size * 4) as usize);
    for j in 0..size {
        for i in 0..size {
            let x = i as f32 + 0.5 - c;
            let y = c - (j as f32 + 0.5);
            // Pointy-top hexagon split into the three visible faces.
            let inside = x.abs() <= half_w && y.abs() <= r - x.abs() / 3f32.sqrt();
            let rgb = if y >= x.abs() / 3f32.sqrt() {
                TOP
            } else if x < 0.0 {
                LEFT
            } else {
                RIGHT
            };
            px.extend_from_slice(&rgb);
            px.push(if inside { 255 } else { 0 });
        }
    }
    px
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        let now = Instant::now();
        let at = |s| now - Duration::from_secs(s);
        assert_eq!(label(&Status::Synced(at(2)), now), "Last sync: just now");
        assert_eq!(label(&Status::Synced(at(42)), now), "Last sync: 42s ago");
        assert_eq!(
            label(&Status::Absent(Some(at(3725))), now),
            "Bitwarden agent not reachable, last sync 1h 2m ago"
        );
        assert_eq!(
            label(&Status::Absent(None), now),
            "Bitwarden agent not reachable"
        );
    }

    #[test]
    fn icon_has_three_faces_and_a_transparent_corner() {
        let px = icon_rgba(32);
        assert_eq!(px.len(), 32 * 32 * 4);
        let at = |x: usize, y: usize| &px[(y * 32 + x) * 4..][..4];
        assert_eq!(at(0, 0)[3], 0);
        assert_eq!(at(16, 4), [0x4f, 0xe3, 0xc8, 255]);
        assert_eq!(at(8, 22), [0x25, 0x8c, 0x7b, 255]);
        assert_eq!(at(24, 22), [0x33, 0xae, 0x99, 255]);
    }
}
