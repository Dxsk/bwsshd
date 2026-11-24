//! Command line entry point.

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use clap::Parser;

use crate::config::{create_private_dir, ensure_include};
use crate::platform::check_not_root;
use crate::sync::{STATE_ABSENT, candidate_socks, sync};
use crate::tray::{self, Control, Status};

/// Keep your ssh_config in sync with the SSH keys stored in your Bitwarden vault.
#[derive(Debug, Parser)]
#[command(name = "bwsshd", disable_version_flag = true)]
pub struct Cli {
    /// SSH agent endpoint; empty auto-discovers the Bitwarden socket or pipe
    #[arg(long, value_name = "PATH")]
    pub sock: Option<String>,

    /// Generated ssh_config file [default: ~/.ssh/config.automatic.bw]
    #[arg(long, value_name = "FILE")]
    pub out: Option<PathBuf>,

    /// Where public keys are written [default: ~/.ssh/bwsshd-keys]
    #[arg(long, value_name = "DIR")]
    pub keydir: Option<PathBuf>,

    /// ssh_config to add the Include line to [default: ~/.ssh/config]
    #[arg(long, value_name = "FILE")]
    pub ssh_config: Option<PathBuf>,

    /// Loop instead of a single run
    #[arg(long)]
    pub watch: bool,

    /// Show an icon in the notification area (implies --watch): last sync,
    /// resync now, quit
    #[arg(long)]
    pub tray: bool,

    /// Poll interval in watch mode (e.g. 10s, 1m)
    #[arg(long, value_name = "DURATION", default_value = "10s", value_parser = parse_interval)]
    pub interval: Duration,

    /// Append logs to this file instead of stderr (used by the Windows scheduled
    /// task, which has no console)
    #[arg(long, value_name = "FILE")]
    pub logfile: Option<PathBuf>,

    /// Print version and exit
    #[arg(long, short = 'V')]
    pub version: bool,
}

/// Long flags, as accepted with a single dash by the Go releases.
const LONG_FLAGS: [&str; 9] = [
    "sock",
    "out",
    "keydir",
    "ssh-config",
    "watch",
    "interval",
    "logfile",
    "version",
    "help",
];

/// Rewrites Go-style single-dash long flags (`-watch`, `-sock=x`) to their
/// `--` form, so systemd units and scheduled tasks installed by the Go
/// releases keep working after an upgrade.
pub fn normalize_legacy_args<I: IntoIterator<Item = OsString>>(args: I) -> Vec<OsString> {
    args.into_iter()
        .map(|a| {
            let Some(s) = a.to_str() else { return a };
            let Some(rest) = s.strip_prefix('-').filter(|r| !r.starts_with('-')) else {
                return a;
            };
            let name = rest.split_once('=').map_or(rest, |(n, _)| n);
            if LONG_FLAGS.contains(&name) {
                format!("--{rest}").into()
            } else {
                a
            }
        })
        .collect()
}

fn parse_interval(s: &str) -> Result<Duration, String> {
    let d = humantime::parse_duration(s).map_err(|e| e.to_string())?;
    if d.is_zero() {
        return Err("interval must be > 0".into());
    }
    Ok(d)
}

/// The user's home directory, as OpenSSH sees it.
fn home_dir() -> Result<PathBuf, String> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| format!("${var} is not set"))
}

/// Parses the process arguments, runs, and returns the exit code.
pub fn run() -> i32 {
    let cli = Cli::parse_from(normalize_legacy_args(std::env::args_os()));
    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    match run_with(cli) {
        Ok(()) => 0,
        Err(e) => {
            info!("{e}");
            1
        }
    }
}

fn run_with(cli: Cli) -> Result<(), String> {
    let home = home_dir()?;
    let ssh_dir = home.join(".ssh");
    let out = cli
        .out
        .unwrap_or_else(|| ssh_dir.join("config.automatic.bw"));
    let keydir = cli.keydir.unwrap_or_else(|| ssh_dir.join("bwsshd-keys"));
    let ssh_config = cli.ssh_config.unwrap_or_else(|| ssh_dir.join("config"));

    // When launched with no console (the Windows scheduled task runs hidden),
    // stderr goes nowhere; --logfile keeps a journal instead.
    if let Some(path) = &cli.logfile {
        open_logfile(path)?;
    }

    // Running as root would write keys and config owned by root in the user's
    // ~/.ssh and defeats the whole point.
    check_not_root()?;

    ensure_include(&ssh_config, &out)?;

    let candidates = candidate_socks(cli.sock.as_deref(), &home);

    if !cli.watch && !cli.tray {
        return sync(&candidates, &out, &keydir, "").1;
    }

    info!(
        "watching for the Bitwarden agent every {}",
        humantime::format_duration(cli.interval)
    );
    let interval = cli.interval;
    if cli.tray {
        return tray::run(out.clone(), interval, move |control, notify| {
            watch(&candidates, &out, &keydir, interval, &control, &*notify)
        });
    }
    // Never sent: keeps the control channel open so it only acts as a timer.
    let (_keep, control) = mpsc::channel();
    watch(&candidates, &out, &keydir, interval, &control, &|_| {});
    Ok(())
}

/// The watch loop. A resync forgets the last state, so the config is
/// rewritten even if nothing changed; it also runs while paused. Resuming
/// resyncs right away.
fn watch(
    candidates: &[String],
    out: &Path,
    keydir: &Path,
    mut interval: Duration,
    control: &Receiver<Control>,
    notify: &dyn Fn(Status),
) {
    let mut last = String::new();
    let mut synced = None;
    let mut paused = false;
    let mut force = false;
    loop {
        if !paused || force {
            // A forced resync while paused keeps showing "paused".
            let quiet: &dyn Fn(Status) = &|_| {};
            let sink = if paused { quiet } else { notify };
            tick(candidates, out, keydir, &mut last, &mut synced, sink);
        }
        if paused {
            notify(Status::Paused(synced));
        }
        force = false;
        match control.recv_timeout(interval) {
            Ok(Control::Resync) => {
                info!("resync requested");
                last.clear();
                force = true;
            }
            Ok(Control::Interval(d)) => {
                info!("syncing every {}", humantime::format_duration(d));
                interval = d;
            }
            Ok(Control::Pause(p)) => {
                info!("auto sync {}", if p { "paused" } else { "resumed" });
                paused = p;
                last.clear();
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

fn tick(
    candidates: &[String],
    out: &Path,
    keydir: &Path,
    last: &mut String,
    synced: &mut Option<Instant>,
    notify: &dyn Fn(Status),
) {
    match sync(candidates, out, keydir, last) {
        (state, Ok(())) => {
            if state == STATE_ABSENT {
                notify(Status::Absent(*synced));
            } else {
                let now = Instant::now();
                *synced = Some(now);
                notify(Status::Synced(now));
            }
            *last = state;
        }
        (_, Err(e)) => {
            info!("{e}");
            notify(Status::Failed(e));
        }
    }
}

fn open_logfile(path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        create_private_dir(dir)?;
    }
    let mut opts = OpenOptions::new();
    opts.create(true).append(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    let f = opts
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    crate::log::set_output(f);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<OsString> {
        v.iter().map(OsString::from).collect()
    }

    fn parse(v: &[&str]) -> Cli {
        Cli::try_parse_from(normalize_legacy_args(args(v))).unwrap()
    }

    #[test]
    fn legacy_flags_are_rewritten() {
        let got = normalize_legacy_args(args(&[
            "bwsshd",
            "-watch",
            "-interval",
            "5s",
            "-sock=/tmp/a.sock",
            "-ssh-config",
            "-",
            "--out",
            "x",
            "-V",
            "-nope",
        ]));
        assert_eq!(
            got,
            args(&[
                "bwsshd",
                "--watch",
                "--interval",
                "5s",
                "--sock=/tmp/a.sock",
                "--ssh-config",
                "-",
                "--out",
                "x",
                "-V",
                "-nope",
            ])
        );
    }

    #[test]
    fn go_style_command_line_parses() {
        let cli = parse(&[
            "bwsshd",
            "-watch",
            "-interval",
            "1m",
            "-logfile",
            "C:/log.txt",
        ]);
        assert!(cli.watch);
        assert_eq!(cli.interval, Duration::from_secs(60));
        assert_eq!(cli.logfile.as_deref(), Some(Path::new("C:/log.txt")));
    }

    #[test]
    fn defaults() {
        let cli = parse(&["bwsshd"]);
        assert!(!cli.watch && !cli.version);
        assert_eq!(cli.interval, Duration::from_secs(10));
        assert!(cli.sock.is_none() && cli.out.is_none());
    }

    #[test]
    fn interval_must_be_positive() {
        assert!(Cli::try_parse_from(args(&["bwsshd", "--interval", "0s"])).is_err());
        assert!(Cli::try_parse_from(args(&["bwsshd", "--interval", "soon"])).is_err());
    }
}
