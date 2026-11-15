//! Turning a Bitwarden key name into an SSH target.

use std::net::IpAddr;

/// Marker that makes bwsshd ignore a key entirely.
const SKIP_MARKER: &str = "[nobwsshd]";

/// Where a key should be used, parsed from its Bitwarden name.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Target {
    pub host: String,
    pub user: Option<String>,
    pub port: Option<String>,
}

/// Reports whether a Bitwarden key name opts out of bwsshd.
pub fn skip_key(comment: &str) -> bool {
    comment.contains(SKIP_MARKER)
}

/// Returns the target from the first token shaped like `[user@]host[:port]`.
///
/// `debian@mtmg.example.com:2222` gives host `mtmg.example.com`, user `debian`
/// and port `2222`; `prod pve1.example.com` gives just the host. Version-like
/// tokens (`v1.2`) are ignored: a real domain ends in an alphabetic TLD.
pub fn parse_target(comment: &str) -> Option<Target> {
    comment.split_whitespace().find_map(|field| {
        let field = field.trim_matches('.');
        let (user, hostport) = match field.rfind('@') {
            Some(at) => (Some(&field[..at]), &field[at + 1..]),
            None => (None, field),
        };
        let (host, port) = split_host_port(hostport);
        looks_like_host(host).then(|| Target {
            host: host.to_string(),
            user: user.map(str::to_string),
            port: port.map(str::to_string),
        })
    })
}

/// Separates a trailing numeric `:port` (or `[ipv6]:port`) from the host,
/// leaving a bare IPv6 literal untouched.
fn split_host_port(s: &str) -> (&str, Option<&str>) {
    let split = if let Some(rest) = s.strip_prefix('[') {
        rest.split_once("]:")
            .filter(|(h, _)| !h.contains(['[', ']']))
    } else {
        s.rsplit_once(':')
            .filter(|(h, _)| !h.contains([':', '[', ']']))
    };
    match split {
        Some((host, port)) if is_digits(port) => (host, Some(port)),
        _ => (s, None),
    }
}

fn is_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

/// Accepts an IP literal, or a dotted name whose last label (TLD) is
/// alphabetic. That rejects version strings like `v1.2` or `1.0.3`.
pub fn looks_like_host(s: &str) -> bool {
    // Not a hostname; this also blocks path traversal in derived filenames.
    if s.contains(['/', '\\', ' ']) {
        return false;
    }
    if s.parse::<IpAddr>().is_ok() {
        return true;
    }
    match s.rsplit_once('.') {
        Some((_, tld)) => !tld.is_empty() && tld.chars().all(|c| c.is_ascii_alphabetic()),
        None => false,
    }
}

/// Keeps a name usable as a filename.
pub fn sanitize(s: &str) -> String {
    s.trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_cases() {
        let cases = [
            ("pve1.example.com", "pve1.example.com", "", ""),
            ("ssh key ops pve1.example.com", "pve1.example.com", "", ""),
            ("prod pve1.example.com root", "pve1.example.com", "", ""),
            ("debian@mtmg.example.com", "mtmg.example.com", "debian", ""),
            ("prod root@pve1.example.com", "pve1.example.com", "root", ""),
            ("db admin@10.0.0.5", "10.0.0.5", "admin", ""),
            ("debian@host.fr:2222", "host.fr", "debian", "2222"),
            ("host.fr:2222", "host.fr", "", "2222"),
            ("ops admin@10.0.0.5:22 prod", "10.0.0.5", "admin", "22"),
            ("admin@[::1]:2222", "::1", "admin", "2222"),
            ("bastion ::1", "::1", "", ""),
            (
                "ops admin@[2001:db8::10]:2222",
                "2001:db8::10",
                "admin",
                "2222",
            ),
            ("db 2001:db8::10", "2001:db8::10", "", ""),
            ("ansible", "", "", ""),
            ("shared key", "", "", ""),
            (".hidden", "", "", ""),            // dots trimmed, no inner dot
            ("deploy v1.2 server", "", "", ""), // version string, numeric TLD
            ("backup 1.0.3 build", "", "", ""), // version string
            ("deploy ../../tmp/pwn.com x", "", "", ""), // path separators rejected
        ];
        for (input, host, user, port) in cases {
            let got = parse_target(input).unwrap_or_default();
            assert_eq!(got.host, host, "host of {input:?}");
            assert_eq!(got.user.as_deref().unwrap_or(""), user, "user of {input:?}");
            assert_eq!(got.port.as_deref().unwrap_or(""), port, "port of {input:?}");
        }
    }

    #[test]
    fn skip_key_cases() {
        let cases = [
            ("backup [nobwsshd] old", true),
            ("[nobwsshd]", true),
            ("debian@mtmg.example.com", false),
            ("pve1.example.com [nobwsshd]", true),
            ("nobwsshd", false), // needs the brackets
        ];
        for (input, want) in cases {
            assert_eq!(skip_key(input), want, "{input:?}");
        }
    }

    #[test]
    fn sanitize_cases() {
        assert_eq!(sanitize(" my key/name "), "my_key_name");
        // Even if a host slipped through, the filename must never keep separators.
        assert!(!sanitize("/../../../tmp/pwn.com").contains(['/', '\\']));
    }
}
