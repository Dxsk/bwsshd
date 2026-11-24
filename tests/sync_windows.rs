//! End-to-end sync against an in-process agent on a named pipe, the Bitwarden
//! transport on Windows. The unix socket side lives in sync_unix.rs.
#![cfg(windows)]

use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use bwsshd::agent::{Key, encode_identities_answer, put_string};
use bwsshd::sync::{STATE_ABSENT, sync};
use tempfile::TempDir;
use windows_sys::Win32::Foundation::{ERROR_PIPE_CONNECTED, GetLastError, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows_sys::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

/// Creates one instance of the pipe `name`; the handle is returned as an
/// integer so it can cross threads.
fn create_instance(name: &str) -> isize {
    let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
    // SAFETY: `wide` is NUL-terminated and outlives the call.
    let h = unsafe {
        CreateNamedPipeW(
            wide.as_ptr(),
            PIPE_ACCESS_DUPLEX,
            PIPE_TYPE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            0,
            std::ptr::null(),
        )
    };
    assert_ne!(h, INVALID_HANDLE_VALUE, "CreateNamedPipeW failed");
    h as isize
}

/// Serves `keys` on a fresh pipe and returns its path. The first instance
/// exists before this returns, so the client never sees "file not found".
fn serve_agent(keys: Vec<Key>) -> String {
    static N: AtomicU32 = AtomicU32::new(0);
    let name = format!(
        r"\\.\pipe\bwsshd-test-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    );
    let mut handle = create_instance(&name);
    let path = name.clone();
    thread::spawn(move || {
        loop {
            // SAFETY: `handle` is a live pipe instance owned by this thread.
            let ok = unsafe { ConnectNamedPipe(handle as _, std::ptr::null_mut()) } != 0;
            if !ok && unsafe { GetLastError() } != ERROR_PIPE_CONNECTED {
                return;
            }
            // Open the next instance before serving, so a second dial finds one.
            let next = create_instance(&name);
            // SAFETY: the handle is owned here and closed when `conn` drops.
            let mut conn = unsafe { File::from_raw_handle(handle as _) };
            let mut req = [0u8; 5];
            if conn.read_exact(&mut req).is_ok() {
                let _ = conn.write_all(&encode_identities_answer(&keys));
                let _ = conn.flush();
            }
            handle = next;
        }
    });
    path
}

fn ed25519(comment: &str) -> Key {
    let mut blob = Vec::new();
    put_string(&mut blob, b"ssh-ed25519");
    put_string(&mut blob, &[7u8; 32]);
    Key::new(blob, comment)
}

#[test]
fn sync_over_named_pipe() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("config.bw");
    let keydir = dir.path().join("keys");
    let pipe = serve_agent(vec![ed25519("debian@(vps1,[2001:db8::10]):23487")]);

    let (state, res) = sync(std::slice::from_ref(&pipe), &out, &keydir, "");
    res.unwrap();
    assert!(state.starts_with(&format!("{pipe}|")), "{state}");
    let cfg = std::fs::read_to_string(&out).unwrap();
    assert!(cfg.contains("Host vps1"), "{cfg}");
    assert!(cfg.contains("HostName 2001:db8::10"), "{cfg}");
    assert!(cfg.contains("Port 23487"), "{cfg}");
    // OpenSSH for Windows wants the pipe with forward slashes.
    assert!(cfg.contains("IdentityAgent //./pipe/bwsshd-test-"), "{cfg}");
    assert!(keydir.join("vps1.pub").exists());

    // Same state: nothing rewritten.
    let (again, res) = sync(std::slice::from_ref(&pipe), &out, &keydir, &state);
    res.unwrap();
    assert_eq!(again, state);

    // A missing pipe falls through to the next candidate.
    let dead = format!(r"{pipe}-dead");
    let (state2, res) = sync(&[dead.clone(), pipe.clone()], &out, &keydir, "");
    res.unwrap();
    assert!(state2.starts_with(&format!("{pipe}|")), "{state2}");

    // No pipe answers: config blanked, keydir removed.
    let (s, res) = sync(&[dead], &out, &keydir, &state);
    res.unwrap();
    assert_eq!(s, STATE_ABSENT);
    assert!(!keydir.exists());
    assert!(
        std::fs::read_to_string(&out)
            .unwrap()
            .contains("socket absent")
    );
}
