//! Turning a Bitwarden key name into an SSH target.

use std::net::IpAddr;

/// Marker that makes bwsshd ignore a key entirely.
const SKIP_MARKER: &str = "[nobwsshd]";

/// Where a key should be used, parsed from its Bitwarden name.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Target {
    /// The `Host` pattern: what you type after `ssh`.
    pub host: String,
    /// The `HostName`: where SSH connects. Same as `host` unless an alias
    /// was given with `(alias,address)`.
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<String>,
}

/// Reports whether a Bitwarden key name opts out of bwsshd.
pub fn skip_key(comment: &str) -> bool {
    comment.contains(SKIP_MARKER)
}

/// Returns the target from the first token shaped like `[user@]host[:port]`
/// or `[user@](alias,address)[:port]`.
///
/// `debian@mtmg.example.com:2222` gives host `mtmg.example.com`, user `debian`
/// and port `2222`; `prod pve1.example.com` gives just the host. Version-like
/// tokens (`v1.2`) are ignored: a real domain ends in an alphabetic TLD.
///
/// `debian@(vps1,10.0.0.1):2222` names the block `vps1` and connects to
/// `10.0.0.1`, so `ssh vps1` works for a host known only by its address.
pub fn parse_target(comment: &str) -> Option<Target> {
    comment.split_whitespace().find_map(|field| {
        let field = field.trim_matches('.');
        let (user, hostport) = match field.rfind('@') {
            Some(at) => (Some(&field[..at]), &field[at + 1..]),
            None => (None, field),
        };
        let target = |host: &str, hostname: &str, port: Option<&str>| Target {
            host: host.to_string(),
            hostname: hostname.to_string(),
            user: user.map(str::to_string),
            port: port.map(str::to_string),
        };
        if hostport.starts_with('(') {
            // An explicit alias that does not parse is a typo, not a hostname.
            let (alias, address, port) = split_alias(hostport)?;
            return Some(target(alias, address, port));
        }
        let (host, port) = split_host_port(hostport);
        looks_like_host(host).then(|| target(host, host, port))
    })
}

/// Parses `(alias,address)[:port]`. The alias becomes a `Host` pattern, so it
/// is held to plain name characters: no wildcards, negation or separators.
/// The address may be a single-label name (`nas`), since the alias form is
/// explicit and does not need the TLD heuristic.
fn split_alias(s: &str) -> Option<(&str, &str, Option<&str>)> {
    let (inner, after) = s.strip_prefix('(')?.split_once(')')?;
    let port = match after {
        "" => None,
        p => Some(p.strip_prefix(':').filter(|p| is_digits(p))?),
    };
    let (alias, address) = inner.split_once(',')?;
    let address = address
        .strip_prefix('[')
        .and_then(|a| a.strip_suffix(']'))
        .unwrap_or(address);
    (is_alias(alias) && is_address(address)).then_some((alias, address, port))
}

fn is_alias(s: &str) -> bool {
    !s.is_empty()
        && !s.starts_with(['.', '-'])
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// An IP literal or a DNS name made of letters, digits and inner hyphens.
fn is_address(s: &str) -> bool {
    s.parse::<IpAddr>().is_ok()
        || (!s.is_empty()
            && s.split('.').all(|label| {
                !label.is_empty()
                    && !label.starts_with('-')
                    && !label.ends_with('-')
                    && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            }))
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
            assert_eq!(got.hostname, host, "hostname of {input:?}");
            assert_eq!(got.user.as_deref().unwrap_or(""), user, "user of {input:?}");
            assert_eq!(got.port.as_deref().unwrap_or(""), port, "port of {input:?}");
        }
    }

    #[test]
    fn parse_target_alias_cases() {
        // (input, Host, HostName, user, port)
        let cases = [
            (
                "debian@(vps1,10.0.0.1):23487",
                "vps1",
                "10.0.0.1",
                "debian",
                "23487",
            ),
            ("prod (vps1,10.0.0.1)", "vps1", "10.0.0.1", "", ""),
            ("(nas,nas)", "nas", "nas", "", ""),
            (
                "(db.prod,db1.internal.example.com)",
                "db.prod",
                "db1.internal.example.com",
                "",
                "",
            ),
            (
                "root@(edge,2001:db8::10):22",
                "edge",
                "2001:db8::10",
                "root",
                "22",
            ),
            ("(edge,[2001:db8::10])", "edge", "2001:db8::10", "", ""),
            (
                "debian@(vps1,[2001:db8::10]):23487",
                "vps1",
                "2001:db8::10",
                "debian",
                "23487",
            ),
            // A broken alias token is skipped; a later plain host still counts.
            (
                "(vps*,10.0.0.1) pve1.example.com",
                "pve1.example.com",
                "pve1.example.com",
                "",
                "",
            ),
        ];
        for (input, host, hostname, user, port) in cases {
            let got = parse_target(input).unwrap_or_default();
            assert_eq!(got.host, host, "host of {input:?}");
            assert_eq!(got.hostname, hostname, "hostname of {input:?}");
            assert_eq!(got.user.as_deref().unwrap_or(""), user, "user of {input:?}");
            assert_eq!(got.port.as_deref().unwrap_or(""), port, "port of {input:?}");
        }

        let rejected = [
            "(vps1,)",
            "(,10.0.0.1)",
            "(vps1 10.0.0.1)",
            "(vps1,10.0.0.1):ssh",
            "(vps1,10.0.0.1)x",
            "(../vps1,10.0.0.1)",
            "(!vps1,10.0.0.1)",
            "(vps1,10.0.0.1/24)",
            "(vps1,-bad.example.com)",
        ];
        for input in rejected {
            assert_eq!(parse_target(input), None, "{input:?}");
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
