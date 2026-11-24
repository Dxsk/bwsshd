//! Per-OS glue: where the Bitwarden agent lives, how to reach it, and how the
//! local OpenSSH client wants paths spelled in its config.

use std::io::{self, Read, Write};
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

/// A connected, bidirectional agent transport (unix socket or named pipe).
pub trait AgentStream: Read + Write {}
impl<T: Read + Write> AgentStream for T {}

/// Absolute, lexically cleaned path (`..` and `.` folded), the equivalent of
/// Go's `filepath.Abs`. Symlinks are not resolved, on purpose: the Include
/// line must name the path the user configured.
pub fn clean_abs(p: &Path) -> io::Result<PathBuf> {
    let abs = std::path::absolute(p)?;
    let mut out = PathBuf::new();
    for c in abs.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(out.components().next_back(), Some(Component::Normal(_))) {
                    out.pop();
                }
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// Path as text with forward slashes. OpenSSH on Windows accepts `//./pipe/...`
/// and `C:/...` but treats a backslash as an escape; on unix this is a no-op.
pub fn to_slash(p: impl AsRef<Path>) -> String {
    let s = p.as_ref().to_string_lossy();
    if cfg!(windows) {
        s.replace('\\', "/")
    } else {
        s.into_owned()
    }
}
