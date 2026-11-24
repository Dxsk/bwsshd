---
title: Run at login
description: Keep bwsshd watching your vault on Linux, Windows and macOS.
section: Guides
order: 2
---

In watch mode bwsshd polls the agent every 10 seconds and rewrites the config only when the key set changes. When Bitwarden closes, it empties the generated config and removes its key folder until the app comes back.

## One command

```sh
bwsshd install
```

This starts the binary you ran it with at every login, as you, with no administrator rights. The one-line installers and the Windows setup run it for you; after `cargo install bwsshd`, run it yourself once.

| Platform | What it sets up | Logs |
|----------|-----------------|------|
| Linux | systemd user service `bwsshd.service`, headless | `journalctl --user -u bwsshd -f` |
| macOS | LaunchAgent `fr.infrasouveraine.bwsshd`, with the menu bar icon | `~/Library/Logs/bwsshd.log` |
| Windows | logon task `bwsshd`, with the notification area icon | `%LOCALAPPDATA%\bwsshd\bwsshd.log` |

Flags given to `install` go into the service, with paths made absolute:

```sh
bwsshd install --interval 30s
bwsshd install --tray    # Linux: add the icon on a desktop session
```

Run `install` again to change them or after moving the binary. `bwsshd uninstall` removes the service and leaves the binary where it is.

A classic Windows service would be the wrong tool: it runs at boot as `LocalSystem` in session 0, where it sees neither your Bitwarden pipe nor your `~/.ssh`. The logon task runs in your session and restarts bwsshd if it crashes. On macOS the LaunchAgent does the same, but not after **Quit** from the menu bar.

## The tray icon

With `--tray`, bwsshd adds an icon to the Windows notification area, the macOS menu bar or the Linux system tray. Its right-click menu shows the version and when the agent was last read, and offers **Resync now** (rewrite the config right away), **Pause auto sync** (freeze the config as it is; the icon turns grey), **Open generated config** (in a text editor), a link to the GitHub project and **Quit bwsshd**. On Windows and macOS, a left click opens a small panel with the version, commit, build date and sync status, an auto sync switch, and a picker for the sync interval (5s to 5m). On Linux the interval sits in a **Sync every** submenu. A pause or an interval picked in the tray is kept across restarts and overrides `--interval`. It lives in a small `settings` file: `%APPDATA%\bwsshd` on Windows, `~/Library/Application Support/bwsshd` on macOS, `~/.config/bwsshd` on Linux. Delete it to go back to the command line value.

In tray mode, bwsshd also asks GitHub for the latest release at start and once a day, through the system `curl`. When a newer version exists, the menu and the panel say so and link to the release page; the log gets an `update available` line. Nothing is downloaded or installed for you. On Linux, KDE shows the icon natively and GNOME needs the AppIndicator extension.
