---
title: Run at login
description: Keep bwsshd watching your vault on Linux, Windows and macOS.
section: Guides
order: 2
---

In watch mode bwsshd polls the agent every 10 seconds and rewrites the config only when the key set changes. When Bitwarden closes, it empties the generated config and removes its key folder until the app comes back.

## The tray icon

With `--tray`, bwsshd adds an icon to the Windows notification area, the macOS menu bar or the Linux system tray. Its right-click menu shows the version and when the agent was last read, and offers **Resync now** (rewrite the config right away), **Open generated config** (in a text editor) and **Quit bwsshd**. On Windows and macOS, a left click opens a small panel with the version, commit, build date and sync status. The Windows and macOS setups below turn it on; on Linux, KDE shows it natively and GNOME needs the AppIndicator extension.

## Linux: systemd user service

From an extracted release archive, or from a clone of the repository:

```sh
make install
```

This installs the binary into `~/.local/bin`, installs the user unit and enables it. Follow the logs with:

```sh
journalctl --user -u bwsshd -f
```

Remove it with `make uninstall`.

The service runs headless. On a desktop session, add the tray icon with an override:

```sh
systemctl --user edit bwsshd
```

```ini
[Service]
ExecStart=
ExecStart=%h/.local/bin/bwsshd --watch --tray
```

## Windows: scheduled task

A classic Windows service is the wrong tool: it runs at boot as `LocalSystem` in session 0, where it sees neither your Bitwarden pipe nor your `~/.ssh`. The equivalent of a systemd user service is a **per-user task at logon**, which the bundled script registers (no admin rights needed):

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

The task runs `bwsshd --watch --tray` hidden, in your session, and restarts it if it crashes. Logs go to `%LOCALAPPDATA%\bwsshd\bwsshd.log`:

```powershell
Get-Content -Wait "$env:LOCALAPPDATA\bwsshd\bwsshd.log"
```

Remove it with `uninstall.ps1`.

## macOS: LaunchAgent

The one-line installer sets this up for you. To do it by hand, save this as `~/Library/LaunchAgents/fr.infrasouveraine.bwsshd.plist`, adjusting the binary path and your home folder:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>fr.infrasouveraine.bwsshd</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/you/.local/bin/bwsshd</string>
        <string>--watch</string>
        <string>--tray</string>
        <string>--logfile</string>
        <string>/Users/you/Library/Logs/bwsshd.log</string>
    </array>
    <key>RunAtLoad</key><true/>
    <!-- Restart on crash, but not after Quit from the menu bar icon. -->
    <key>KeepAlive</key><dict><key>SuccessfulExit</key><false/></dict>
</dict>
</plist>
```

Then load it:

```sh
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/fr.infrasouveraine.bwsshd.plist
```
