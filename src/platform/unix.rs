//! Linux and macOS: the Bitwarden desktop agent is a unix-domain socket, and
//! running as root would write into the user's ~/.ssh as root.

use std::io;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{AgentStream, clean_abs};

/// A stuck agent must not hang the watch loop forever.
const IO_TIMEOUT: Duration = Duration::from_secs(5);

/// Refuses to run as root: keys and config would be written owned by root in
/// the user's ~/.ssh, defeating the whole point.
pub fn check_not_root() -> Result<(), String> {
    // SAFETY: geteuid has no preconditions and cannot fail.
    if unsafe { libc::geteuid() } == 0 {
        return Err("refusing to run as root; run as your own user".into());
    }
    Ok(())
}

/// Bitwarden desktop agent sockets to try, in order. The socket is created by
/// the desktop client, so a Vaultwarden backend uses the same paths.
#[cfg(target_os = "macos")]
pub fn default_agent_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".bitwarden-ssh-agent.sock"), // .dmg download
        home.join("Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock"), // Mac App Store
    ]
}

/// Bitwarden desktop agent sockets to try, in order, across native, Flatpak
/// and Snap installs. The socket is created by the desktop client, so a
/// Vaultwarden backend uses the same paths.
#[cfg(not(target_os = "macos"))]
pub fn default_agent_paths(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".bitwarden-ssh-agent.sock"), // native / deb / rpm
        home.join(".var/app/com.bitwarden.desktop/data/.bitwarden-ssh-agent.sock"), // flatpak
        home.join("snap/bitwarden/current/.bitwarden-ssh-agent.sock"), // snap
    ]
}

/// Connects to the agent at `path`, always a unix-domain socket here.
pub fn dial_agent(path: &str) -> io::Result<Box<dyn AgentStream>> {
    let s = UnixStream::connect(path)?;
    s.set_read_timeout(Some(IO_TIMEOUT))?;
    s.set_write_timeout(Some(IO_TIMEOUT))?;
    Ok(Box::new(s))
}

/// Text to write after `Include `. An absolute path works directly on unix.
pub fn format_include(_ssh_config_dir: &Path, out: &Path) -> String {
    clean_abs(out)
        .unwrap_or_else(|_| out.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn default_paths_end_with_snap() {
        let got = default_agent_paths(Path::new("/home/u"));
        assert_eq!(
            got.last().unwrap(),
            Path::new("/home/u/snap/bitwarden/current/.bitwarden-ssh-agent.sock")
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn default_paths_end_with_app_store() {
        let got = default_agent_paths(Path::new("/Users/u"));
        assert_eq!(
            got.last().unwrap(),
            Path::new(
                "/Users/u/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock"
            )
        );
    }

    #[test]
    fn format_include_is_absolute() {
        assert_eq!(
            format_include(
                Path::new("/home/u/.ssh"),
                Path::new("/home/u/.ssh/config.automatic.bw")
            ),
            "/home/u/.ssh/config.automatic.bw"
        );
    }
}
