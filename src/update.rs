//! Checks GitHub for a newer release. It shells out to curl (bundled with
//! Windows 10+, macOS and most Linux desktops) rather than linking a TLS stack.

use std::process::Command;

const API: &str = "https://api.github.com/repos/Dxsk/bwsshd/releases/latest";
pub const REPO: &str = "https://github.com/Dxsk/bwsshd";
pub const RELEASES: &str = "https://github.com/Dxsk/bwsshd/releases/latest";

/// The latest release tag, e.g. `v0.3.0`. `None` when offline, rate-limited
/// or curl is missing.
pub fn latest() -> Option<String> {
    let mut cmd = Command::new("curl");
    cmd.args(["-fsSL", "--max-time", "15"]).args([
        "-H",
        "Accept: application/vnd.github+json",
        API,
    ]);
    #[cfg(windows)]
    {
        // The daemon runs hidden; keep curl from flashing a console.
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().ok().filter(|o| o.status.success())?;
    tag_name(&String::from_utf8_lossy(&out.stdout))
}

fn tag_name(json: &str) -> Option<String> {
    let rest = &json[json.find("\"tag_name\"")? + 10..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    Some(rest[..rest.find('"')?].to_string())
}

/// Whether `tag` (`v1.2.3`) is a newer version than `current` (`1.2.3`).
pub fn is_newer(tag: &str, current: &str) -> bool {
    let parse = |s: &str| -> Option<Vec<u64>> {
        s.trim_start_matches('v')
            .split(['.', '-', '+'])
            .take(3)
            .map(|p| p.parse().ok())
            .collect()
    };
    matches!((parse(tag), parse(current)), (Some(a), Some(b)) if a > b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_tag_name() {
        let json = r#"{"html_url": "x", "tag_name" : "v0.3.0", "name": "v0.3.0"}"#;
        assert_eq!(tag_name(json).as_deref(), Some("v0.3.0"));
        assert_eq!(tag_name(r#"{"message":"Not Found"}"#), None);
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("v0.3.0", "0.2.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(is_newer("v0.2.10", "0.2.9"));
        assert!(!is_newer("v0.2.0", "0.2.0"));
        assert!(!is_newer("v0.1.9", "0.2.0"));
        assert!(!is_newer("nightly", "0.2.0"));
    }
}
