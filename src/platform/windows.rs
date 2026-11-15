//! Windows: the Bitwarden desktop agent is a named pipe, not a unix socket, and
//! there is no root user to refuse.

use std::fs::OpenOptions;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::{AgentStream, clean_abs, to_slash};

/// The pipe Bitwarden registers as the standard OpenSSH agent, which makes it a
/// drop-in replacement for Windows' own ssh-agent.
pub const OPENSSH_AGENT_PIPE: &str = r"\\.\pipe\openssh-ssh-agent";

/// No-op on Windows: there is no euid, and files are written under the user's
/// own profile regardless.
pub fn check_not_root() -> Result<(), String> {
    Ok(())
}

pub fn default_agent_paths(_home: &Path) -> Vec<PathBuf> {
    vec![PathBuf::from(OPENSSH_AGENT_PIPE)]
}

/// Win32 ERROR_PIPE_BUSY: every pipe instance is taken, typically by an ssh
/// session talking to the agent at that moment. Reporting it as "agent absent"
/// would wipe the config until the next tick, so retry for up to ~2 s like
/// go-winio's DialPipe did.
const ERROR_PIPE_BUSY: i32 = 231;
const BUSY_RETRIES: u32 = 20;
const BUSY_WAIT: Duration = Duration::from_millis(100);

/// Connects to the agent at `path`. A named pipe opens like a regular file, so
/// std is enough; no Win32 bindings needed.
pub fn dial_agent(path: &str) -> io::Result<Box<dyn AgentStream>> {
    if !path.starts_with(r"\\") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("{path}: only named pipes (\\\\.\\pipe\\...) are supported on Windows"),
        ));
    }
    let mut attempt = 0;
    loop {
        match OpenOptions::new().read(true).write(true).open(path) {
            Ok(pipe) => return Ok(Box::new(pipe)),
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) && attempt < BUSY_RETRIES => {
                attempt += 1;
                std::thread::sleep(BUSY_WAIT);
            }
            Err(e) => return Err(e),
        }
    }
}

/// Text to write after `Include `. OpenSSH on Windows treats a token that does
/// not start with '/' as relative to ~/.ssh and mangles backslashes as glob
/// escapes, so an absolute `C:\` path never matches. Emit a forward-slash path
/// relative to the config directory, falling back to an absolute forward-slash
/// path when the generated file lives outside it.
pub fn format_include(ssh_config_dir: &Path, out: &Path) -> String {
    let (Ok(dir), Ok(out)) = (clean_abs(ssh_config_dir), clean_abs(out)) else {
        return to_slash(out);
    };
    match out.strip_prefix(&dir) {
        Ok(rel) if !rel.as_os_str().is_empty() => to_slash(rel),
        _ => to_slash(out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_paths_end_with_pipe() {
        let got = default_agent_paths(Path::new(r"C:\Users\u"));
        assert_eq!(got.last().unwrap(), Path::new(OPENSSH_AGENT_PIPE));
    }

    #[test]
    fn format_include_is_config_relative() {
        assert_eq!(
            format_include(
                Path::new(r"C:\Users\u\.ssh"),
                Path::new(r"C:\Users\u\.ssh\config.automatic.bw")
            ),
            "config.automatic.bw"
        );
        assert_eq!(
            format_include(Path::new(r"C:\Users\u\.ssh"), Path::new(r"D:\gen\bw.conf")),
            "D:/gen/bw.conf"
        );
    }

    #[test]
    fn dial_rejects_non_pipe() {
        assert!(dial_agent("/tmp/agent.sock").is_err());
    }
}
