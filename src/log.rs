//! Minimal line logger: `bwsshd: <message>` with no timestamp, since journald
//! (and the Windows log file reader) already timestamp each line.

use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

static SINK: Mutex<Option<File>> = Mutex::new(None);

/// Redirects log output to `file` instead of stderr.
pub fn set_output(file: File) {
    *SINK.lock().unwrap_or_else(|e| e.into_inner()) = Some(file);
}

#[doc(hidden)]
pub fn write(args: std::fmt::Arguments) {
    let line = format!("bwsshd: {args}\n");
    let mut sink = SINK.lock().unwrap_or_else(|e| e.into_inner());
    match sink.as_mut() {
        Some(f) => {
            let _ = f.write_all(line.as_bytes());
        }
        None => {
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => { $crate::log::write(format_args!($($arg)*)) };
}
