//! Finding the agent and reconciling the generated config with it.

use std::path::Path;

use crate::agent::{Key, list_keys};
use crate::config::{clean, generate};
use crate::platform::{default_agent_paths, dial_agent};

/// State reported when no agent endpoint answers.
pub const STATE_ABSENT: &str = "absent";

/// Environment overrides for the agent endpoint, in priority order.
pub const SOCK_ENV_VARS: [&str; 2] = ["BW_SSH_SOCK", "BITWARDEN_SSH_AUTH_SOCK"];

/// Agent endpoints to try, in order. An explicit `--sock` wins outright;
/// otherwise the env overrides come first, then the platform's default
/// Bitwarden endpoints.
pub fn candidate_socks(explicit: Option<&str>, home: &Path) -> Vec<String> {
    if let Some(s) = explicit.filter(|s| !s.is_empty()) {
        return vec![s.to_string()];
    }
    SOCK_ENV_VARS
        .iter()
        .filter_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty()))
        .chain(
            default_agent_paths(home)
                .into_iter()
                .map(|p| p.to_string_lossy().into_owned()),
        )
        .collect()
}

/// Reconciles the config with the agent. It only writes when the observed
/// state differs from `last`, so the watch loop doesn't churn files every tick.
/// Returns the new state.
pub fn sync(
    candidates: &[String],
    out: &Path,
    keydir: &Path,
    last: &str,
) -> (String, Result<(), String>) {
    let Some((keys, sock)) = probe(candidates) else {
        if last == STATE_ABSENT {
            return (last.to_string(), Ok(()));
        }
        return (STATE_ABSENT.to_string(), clean(out, keydir));
    };
    // The socket is part of the state so switching installs regenerates.
    let state = format!("{sock}|{}", fingerprint(&keys));
    if state == last {
        return (state, Ok(()));
    }
    let res = generate(&keys, out, keydir, sock);
    (state, res)
}

/// Tries each endpoint and returns the key list from the first that answers,
/// with the endpoint that worked (needed for the IdentityAgent line). `None`
/// when none respond (Bitwarden closed or still starting).
pub fn probe(candidates: &[String]) -> Option<(Vec<Key>, &str)> {
    candidates.iter().find_map(|s| {
        let mut conn = dial_agent(s).ok()?;
        let keys = list_keys(&mut conn).ok()?;
        Some((keys, s.as_str()))
    })
}

/// A stable signature of the key set, used for change detection. The comment
/// is included because it carries host/user/port and `[nobwsshd]`: a rename
/// must regenerate (issue #1).
pub fn fingerprint(keys: &[Key]) -> String {
    let mut fps: Vec<String> = keys
        .iter()
        .map(|k| format!("{}\0{}", k.fingerprint_sha256(), k.comment))
        .collect();
    fps.sort();
    format!("present:{}", fps.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::tests::blob;

    #[test]
    fn fingerprint_includes_comment() {
        let b = blob();
        assert_ne!(
            fingerprint(&[Key::new(b.clone(), "prod")]),
            fingerprint(&[Key::new(b, "debian@host.fr:2222")]),
            "a key rename would be invisible in watch mode"
        );
    }

    #[test]
    fn fingerprint_is_order_independent() {
        let (a, b) = (Key::new(blob(), "a"), Key::new(blob(), "b"));
        assert_eq!(fingerprint(&[a.clone(), b.clone()]), fingerprint(&[b, a]));
    }

    // Env handling lives in one test: tests run in parallel and share the
    // process environment.
    #[test]
    fn candidate_socks_precedence() {
        let home = Path::new("/home/u");
        assert_eq!(candidate_socks(Some("/explicit"), home), ["/explicit"]);

        // SAFETY: no other test reads or writes these variables.
        unsafe { std::env::set_var("BW_SSH_SOCK", "/env/sock") };
        let got = candidate_socks(None, home);
        unsafe { std::env::remove_var("BW_SSH_SOCK") };
        assert_eq!(got[0], "/env/sock");
        assert!(got.len() >= 2, "env sock plus platform defaults: {got:?}");

        let defaults = candidate_socks(Some(""), home);
        assert_eq!(
            defaults.last().map(String::as_str),
            default_agent_paths(home).last().and_then(|p| p.to_str())
        );
    }

    #[test]
    fn sync_absent_cleans_once() {
        let dir = tempfile::TempDir::new().unwrap();
        let (out, keydir) = (dir.path().join("config.bw"), dir.path().join("keys"));
        let gone = vec![dir.path().join("nope.sock").to_string_lossy().into_owned()];
        let (state, res) = sync(&gone, &out, &keydir, "");
        res.unwrap();
        assert_eq!(state, STATE_ABSENT);
        assert!(
            std::fs::read_to_string(&out)
                .unwrap()
                .contains("socket absent")
        );

        // Already absent: nothing is rewritten.
        std::fs::remove_file(&out).unwrap();
        let (state, res) = sync(&gone, &out, &keydir, STATE_ABSENT);
        res.unwrap();
        assert_eq!(state, STATE_ABSENT);
        assert!(!out.exists());
    }
}
