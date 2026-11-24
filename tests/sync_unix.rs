//! End-to-end sync against an in-process agent on a unix socket, the Bitwarden
//! transport on Linux and macOS. On Windows the transport is a named pipe and
//! is exercised against the real Bitwarden agent.
#![cfg(unix)]

use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::thread;

use bwsshd::agent::{Key, encode_identities_answer, put_string};
use bwsshd::sync::{STATE_ABSENT, sync};
use tempfile::TempDir;

/// Starts an agent serving `keys` and returns its socket path. The listener
/// thread lives until the test process exits.
fn serve_agent(dir: &TempDir, keys: Vec<Key>) -> String {
    let sock = dir.path().join("agent.sock");
    let listener = UnixListener::bind(&sock).unwrap();
    thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { return };
            let mut req = [0u8; 5];
            if conn.read_exact(&mut req).is_ok() {
                let _ = conn.write_all(&encode_identities_answer(&keys));
            }
        }
    });
    sock.to_string_lossy().into_owned()
}

fn ed25519(comment: &str) -> Key {
    let mut blob = Vec::new();
    put_string(&mut blob, b"ssh-ed25519");
    put_string(&mut blob, &[7u8; 32]);
    Key::new(blob, comment)
}

#[test]
fn sync_present_then_absent() {
    let dir = TempDir::new().unwrap();
    let out: PathBuf = dir.path().join("config.bw");
    let keydir = dir.path().join("keys");
    let socks = [serve_agent(&dir, vec![ed25519("debian@host.fr:2222")])];

    let (state, res) = sync(&socks, &out, &keydir, "");
    res.unwrap();
    assert_ne!(state, STATE_ABSENT);
    let cfg = std::fs::read_to_string(&out).unwrap();
    assert!(cfg.contains("Host host.fr"), "{cfg}");
    assert!(
        cfg.contains(&format!("IdentityAgent {}", socks[0])),
        "{cfg}"
    );

    // Same state: no-op, state unchanged.
    let (again, res) = sync(&socks, &out, &keydir, &state);
    res.unwrap();
    assert_eq!(again, state);

    // No reachable socket: config blanked and keydir removed.
    let gone = dir.path().join("nope.sock").to_string_lossy().into_owned();
    let (s, res) = sync(&[gone], &out, &keydir, &state);
    res.unwrap();
    assert_eq!(s, STATE_ABSENT);
    assert!(!keydir.exists(), "keydir not removed on absent agent");
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("socket absent")
    );
}

#[test]
fn sync_falls_through_to_next_candidate() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("config.bw");
    let keydir = dir.path().join("keys");
    let sock = serve_agent(&dir, vec![ed25519("pve1.example.com")]);
    let dead = dir.path().join("dead.sock").to_string_lossy().into_owned();

    let (state, res) = sync(&[dead, sock.clone()], &out, &keydir, "");
    res.unwrap();
    assert!(state.starts_with(&format!("{sock}|")), "{state}");
}
