//! `bwsshd install` and `bwsshd uninstall`: start the binary that runs the
//! command at login, as the current user. A systemd user service on Linux, a
//! LaunchAgent on macOS, a logon task on Windows (a Windows service would run
//! in session 0, away from the user's Bitwarden pipe and ~/.ssh).

use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
use crate::config::write_file;

#[cfg(windows)]
const NAME: &str = "bwsshd";
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
const LABEL: &str = "fr.infrasouveraine.bwsshd";

/// Where the service logs when nothing else collects its output.
pub fn default_log(home: &Path) -> Option<PathBuf> {
    if cfg!(windows) {
        let base = std::env::var_os("LOCALAPPDATA")
            .map_or_else(|| home.join("AppData/Local"), PathBuf::from);
        Some(base.join("bwsshd").join("bwsshd.log"))
    } else if cfg!(target_os = "macos") {
        Some(home.join("Library/Logs/bwsshd.log"))
    } else {
        None
    }
}

fn run(cmd: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let msg = String::from_utf8_lossy(if out.stderr.is_empty() {
        &out.stdout
    } else {
        &out.stderr
    })
    .trim()
    .to_string();
    Err(format!("{cmd} {}: {msg}", args.join(" ")))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Linux: systemd user service.

#[cfg(target_os = "linux")]
fn unit_path(home: &Path) -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map_or_else(|| home.join(".config"), PathBuf::from)
        .join("systemd/user/bwsshd.service")
}

#[cfg(target_os = "linux")]
pub fn install(exe: &Path, args: &[String], home: &Path) -> Result<String, String> {
    run("systemctl", &["--user", "show-environment"]).map_err(|_| {
        "no systemd user session: start `bwsshd --watch` from your session startup instead"
            .to_string()
    })?;
    let path = unit_path(home);
    write_file(&path, systemd_unit(exe, args).as_bytes())?;
    run("systemctl", &["--user", "daemon-reload"])?;
    run("systemctl", &["--user", "enable", "bwsshd.service"])?;
    run("systemctl", &["--user", "restart", "bwsshd.service"])?;
    Ok(format!(
        "installed {}\nlogs: journalctl --user -u bwsshd -f",
        path.display()
    ))
}

#[cfg(target_os = "linux")]
pub fn uninstall(home: &Path) -> Result<String, String> {
    let path = unit_path(home);
    if !path.exists() {
        return Err(format!("no service at {}", path.display()));
    }
    let _ = run(
        "systemctl",
        &["--user", "disable", "--now", "bwsshd.service"],
    );
    std::fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))?;
    let _ = run("systemctl", &["--user", "daemon-reload"]);
    Ok(format!("removed {}", path.display()))
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
fn systemd_unit(exe: &Path, args: &[String]) -> String {
    let quote = |a: &str| {
        // `%` and `$` are specifiers and variables in ExecStart.
        let a = a.replace('%', "%%").replace('$', "$$");
        if a.is_empty() || a.contains(|c: char| c.is_whitespace() || "\"'\\".contains(c)) {
            format!("\"{}\"", a.replace('\\', "\\\\").replace('"', "\\\""))
        } else {
            a
        }
    };
    let cmd = std::iter::once(exe.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|a| quote(&a))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "[Unit]
Description=bwsshd - keep ssh_config in sync with the Bitwarden SSH agent
After=default.target

[Service]
ExecStart={cmd}
Restart=on-failure
RestartSec=5
NoNewPrivileges=true
UMask=0077

[Install]
WantedBy=default.target
"
    )
}

// macOS: LaunchAgent.

#[cfg(target_os = "macos")]
fn plist_path(home: &Path) -> PathBuf {
    home.join("Library/LaunchAgents")
        .join(format!("{LABEL}.plist"))
}

#[cfg(target_os = "macos")]
pub fn install(exe: &Path, args: &[String], home: &Path) -> Result<String, String> {
    let path = plist_path(home);
    write_file(&path, launch_agent(exe, args).as_bytes())?;
    // SAFETY: getuid has no preconditions.
    let domain = format!("gui/{}", unsafe { libc::getuid() });
    let _ = run("launchctl", &["bootout", &format!("{domain}/{LABEL}")]);
    let plist = path.to_string_lossy();
    // bootout returns before the old agent is gone, and bootstrap fails until it is.
    let mut res = Ok(());
    for _ in 0..10 {
        res = run("launchctl", &["bootstrap", &domain, &plist]);
        if res.is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    res?;
    Ok(format!("installed {}", path.display()))
}

#[cfg(target_os = "macos")]
pub fn uninstall(home: &Path) -> Result<String, String> {
    let path = plist_path(home);
    if !path.exists() {
        return Err(format!("no LaunchAgent at {}", path.display()));
    }
    // SAFETY: getuid has no preconditions.
    let _ = run(
        "launchctl",
        &[
            "bootout",
            &format!("gui/{}/{LABEL}", unsafe { libc::getuid() }),
        ],
    );
    std::fs::remove_file(&path).map_err(|e| format!("remove {}: {e}", path.display()))?;
    Ok(format!("removed {}", path.display()))
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn launch_agent(exe: &Path, args: &[String]) -> String {
    let strings: String = std::iter::once(exe.to_string_lossy().into_owned())
        .chain(args.iter().cloned())
        .map(|a| format!("        <string>{}</string>\n", xml_escape(&a)))
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>{LABEL}</string>
    <key>ProgramArguments</key>
    <array>
{strings}    </array>
    <key>RunAtLoad</key><true/>
    <!-- Restart on crash, but not after Quit from the menu bar icon. -->
    <key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>
</dict>
</plist>
"#
    )
}

// Windows: per-user logon task.

#[cfg(windows)]
pub fn install(exe: &Path, args: &[String], _home: &Path) -> Result<String, String> {
    let var = |k| std::env::var(k).unwrap_or_default();
    let user = format!("{}\\{}", var("USERDOMAIN"), var("USERNAME"));
    let file = std::env::temp_dir().join(format!("bwsshd-task-{}.xml", std::process::id()));
    std::fs::write(&file, utf16(&task_xml(&user, exe, args)))
        .map_err(|e| format!("write {}: {e}", file.display()))?;
    let _ = run("schtasks", &["/End", "/TN", NAME]);
    let res = run(
        "schtasks",
        &[
            "/Create",
            "/TN",
            NAME,
            "/XML",
            &file.to_string_lossy(),
            "/F",
        ],
    );
    let _ = std::fs::remove_file(&file);
    res?;
    run("schtasks", &["/Run", "/TN", NAME])?;
    Ok(format!("installed the logon task {NAME} and started it"))
}

/// schtasks wants the task file in UTF-16, with a byte order mark.
#[cfg(windows)]
fn utf16(s: &str) -> Vec<u8> {
    [0xFF, 0xFE]
        .into_iter()
        .chain(s.encode_utf16().flat_map(u16::to_le_bytes))
        .collect()
}

#[cfg(windows)]
pub fn uninstall(_home: &Path) -> Result<String, String> {
    let _ = run("schtasks", &["/End", "/TN", NAME]);
    run("schtasks", &["/Delete", "/TN", NAME, "/F"])?;
    Ok(format!("removed the logon task {NAME}"))
}

#[cfg_attr(not(windows), allow(dead_code))]
fn task_xml(user: &str, exe: &Path, args: &[String]) -> String {
    let quote = |a: &String| {
        if a.is_empty() || a.contains(char::is_whitespace) {
            format!("\"{a}\"")
        } else {
            a.clone()
        }
    };
    let args = args.iter().map(quote).collect::<Vec<_>>().join(" ");
    let (user, exe, args) = (
        xml_escape(user),
        xml_escape(&exe.to_string_lossy()),
        xml_escape(&args),
    );
    // Interactive token: the task runs in the user's session, where the
    // Bitwarden pipe and the tray live, without storing a password.
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Keeps ssh_config in sync with the Bitwarden SSH agent.</Description>
  </RegistrationInfo>
  <Triggers>
    <LogonTrigger>
      <Enabled>true</Enabled>
      <UserId>{user}</UserId>
    </LogonTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{user}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <RestartOnFailure>
      <Interval>PT1M</Interval>
      <Count>3</Count>
    </RestartOnFailure>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
      <Arguments>{args}</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    )
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn install(_exe: &Path, _args: &[String], _home: &Path) -> Result<String, String> {
    Err("install is not supported on this platform".into())
}

#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
pub fn uninstall(_home: &Path) -> Result<String, String> {
    Err("uninstall is not supported on this platform".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn unit_quotes_what_systemd_would_split() {
        let unit = systemd_unit(
            Path::new("/home/me/.cargo/bin/bwsshd"),
            &args(&["--watch", "--out", "/home/me/my ssh/%h$x"]),
        );
        assert!(
            unit.contains(
                r#"ExecStart=/home/me/.cargo/bin/bwsshd --watch --out "/home/me/my ssh/%%h$$x""#
            ),
            "{unit}"
        );
    }

    #[test]
    fn launch_agent_escapes_arguments() {
        let plist = launch_agent(
            Path::new("/Users/me/bin/bwsshd"),
            &args(&["--watch", "--out", "/Users/me/a&b"]),
        );
        assert!(plist.contains("<string>/Users/me/bin/bwsshd</string>"));
        assert!(plist.contains("<string>/Users/me/a&amp;b</string>"));
    }

    #[test]
    fn task_quotes_and_escapes() {
        let xml = task_xml(
            r"PC\me",
            Path::new(r"C:\Users\me\.cargo\bin\bwsshd.exe"),
            &args(&["--tray", "--logfile", r"C:\Users\me & co\bwsshd.log"]),
        );
        assert!(xml.contains(r"<Command>C:\Users\me\.cargo\bin\bwsshd.exe</Command>"));
        assert!(xml.contains(
            r"<Arguments>--tray --logfile &quot;C:\Users\me &amp; co\bwsshd.log&quot;</Arguments>"
        ));
        assert!(xml.contains(r"<UserId>PC\me</UserId>"));
    }

    /// The task file is what schtasks accepts: register it under a throwaway
    /// name (never started), then remove it.
    #[cfg(windows)]
    #[test]
    fn schtasks_accepts_the_task() {
        let var = |k| std::env::var(k).unwrap_or_default();
        let user = format!("{}\\{}", var("USERDOMAIN"), var("USERNAME"));
        let exe = std::env::current_exe().unwrap();
        let bytes = utf16(&task_xml(&user, &exe, &args(&["--version"])));
        let dir = tempfile::TempDir::new().unwrap();
        let file = dir.path().join("task.xml");
        std::fs::write(&file, bytes).unwrap();
        let name = format!("bwsshd-test-{}", std::process::id());
        run(
            "schtasks",
            &[
                "/Create",
                "/TN",
                &name,
                "/XML",
                &file.to_string_lossy(),
                "/F",
            ],
        )
        .unwrap();
        run("schtasks", &["/Delete", "/TN", &name, "/F"]).unwrap();
    }
}
