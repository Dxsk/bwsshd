<div align="center">

# bwsshd

**Keep your `ssh_config` in sync with the SSH keys stored in your Bitwarden vault.**

*One key per host, signed from the vault, never written to disk.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust&logoColor=white)](Cargo.toml)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-informational)

</div>

---

`bwsshd` watches the Bitwarden desktop SSH agent and generates a per-host
`ssh_config` that pins exactly one key per host.

SSH then offers the right key first, which matters on hardened servers where
`MaxAuthTries` is low (3 or less) and offering a dozen agent keys gets you
disconnected before the correct one is tried.

## Why this exists

The Bitwarden desktop app can act as an
[SSH agent](https://bitwarden.com/help/ssh-agent/#ssh-agent): your private keys
never leave the vault and signing happens in the app.

The catch is that the agent offers every key it holds, in no host-aware order.

With many keys and a hardened server, SSH hits the auth-attempt limit before
reaching the right key and the connection fails with
`Permission denied (publickey)`.

`bwsshd` fixes this by reading the key list from the agent and writing one
config block per host with `IdentitiesOnly yes`, so SSH offers a single,
deterministic key per host.

One key, one attempt, no wasted tries.

It also keeps several machines in step. The vault already syncs your keys to
every device where Bitwarden runs, but the `ssh_config` pointing at them used
to be copied and fixed by hand on each desktop and laptop. With bwsshd, a new
machine only needs the Bitwarden app and one install: the config rebuilds
itself from the vault, and a key renamed on one device updates everywhere.

It also works the same whether your Bitwarden desktop app is the native package,
the Flatpak or the Snap, and whether your backend is Bitwarden cloud or a
self-hosted Vaultwarden.

The SSH agent socket is created by the desktop client, so the backend does not
change anything.

**Linux, macOS and Windows, amd64 and arm64.** On Linux and macOS the agent is
a unix socket; on Windows it is the named pipe `\\.\pipe\openssh-ssh-agent`,
which the Bitwarden desktop app registers as the standard OpenSSH agent.
bwsshd auto-discovers whichever applies:

| Platform | Install | Agent endpoint |
|----------|---------|----------------|
| Linux | native / deb / rpm | `~/.bitwarden-ssh-agent.sock` |
| Linux | Flatpak | `~/.var/app/com.bitwarden.desktop/data/.bitwarden-ssh-agent.sock` |
| Linux | Snap | `~/snap/bitwarden/current/.bitwarden-ssh-agent.sock` |
| macOS | `.dmg` download | `~/.bitwarden-ssh-agent.sock` |
| macOS | Mac App Store | `~/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock` |
| Windows | any | `\\.\pipe\openssh-ssh-agent` |

On Windows it also writes the generated config with the forward-slash paths and
the relative `Include` that the built-in OpenSSH client expects.

## Before you start

bwsshd reads the keys served by the Bitwarden desktop SSH agent, so set that up
first. Bitwarden's [SSH Agent guide](https://bitwarden.com/help/ssh-agent/)
covers every step; in short:

1. **Enable the agent.** In the Bitwarden desktop app, open *Settings* and turn
   on *Enable SSH agent*
   ([Bitwarden docs](https://bitwarden.com/help/ssh-agent/#enable-ssh-agent)).
2. **On Windows, free the pipe.** Set the *OpenSSH Authentication Agent*
   service's startup type to *Disabled*, so Bitwarden can serve the standard
   agent pipe
   ([Bitwarden docs](https://bitwarden.com/help/ssh-agent/#configure-your-system-to-use-bitwarden-ssh-agent)).
3. **Add your keys.** Create or import SSH key items, named after their target
   host (see [Naming convention](#naming-convention) and
   [creating an SSH key](https://bitwarden.com/help/about-ssh/#create-a-new-ssh-key)).

You also need an OpenSSH client: Linux and macOS ship one, Windows 10 and 11
include it.

Bitwarden's guide also has you export `SSH_AUTH_SOCK`. bwsshd does not need it
for the hosts it manages, since every generated block sets `IdentityAgent`
itself; set it if you want other tools (`git`, `ssh-add`) to use the Bitwarden
agent too.

## How it works

The name you give an SSH key item in Bitwarden becomes the key comment in the
agent.

`bwsshd` reads that comment, extracts a target from it, and writes a block like
this:

```
Host pve1.example.com
    HostName pve1.example.com
    User debian
    IdentityAgent <bitwarden-agent-socket>
    IdentityFile ~/.ssh/bwsshd-keys/pve1.example.com.pub
    IdentitiesOnly yes
```

The private key stays in the vault.

The `.pub` file only tells SSH which key fingerprint to request a signature for.

When you connect, Bitwarden pops up to approve the signature, the key never
touches disk.

Keys whose Bitwarden name has no hostname in it (for example a shared `ansible`
key used on many servers) are written as `.pub` files and listed as comments in
the generated file.

Add a manual `Host` block for those in your own `~/.ssh/config`, referencing the
`.pub` that bwsshd dropped.

## Naming convention

Name your Bitwarden SSH key items with the target.

The first token shaped like `[user@]host[:port]` is used: the first hostname or
IP becomes the host, an optional `user@` prefix sets the SSH user, and a
trailing `:port` sets the port (otherwise the SSH default is used).

Examples:

- `pve1.example.com`
- `ssh key ops pve1.example.com`
- `prod pve1.example.com root`
- `debian@pve1.example.com` (also sets `User debian`)
- `debian@pve1.example.com:2222` (also sets `User debian` and `Port 2222`)

Version-like tokens such as `v1.2` are ignored, so they are never mistaken for a
hostname.

**Your own `Host` name.** Write `(alias,address)` where the host would go to
type a short name after `ssh` while connecting to an address:

```
debian@(vps1,10.0.0.1):23487        ->  Host vps1, HostName 10.0.0.1
debian@(vps2,[2001:db8::20]):23487  ->  Host vps2, HostName 2001:db8::20
```

The alias also names the `.pub` file, so it only takes letters, digits, `.`,
`-` and `_` (no wildcards). The address is an IP, IPv6 with or without
brackets, or a DNS name; a single-label name like `nas` is fine here.

Add `[nobwsshd]` anywhere in the name to make bwsshd skip a key entirely.

---

<details>
<summary><strong>Install</strong></summary>

<br>

Grab a prebuilt binary from the [releases](https://github.com/Dxsk/bwsshd/releases):
`.tar.gz` for Linux (static, any distro) and macOS, `.zip` for Windows, each for
`amd64` and `arm64`.

Or install with Cargo:

```sh
cargo install bwsshd
```

Or build from source into `dist/`:

```sh
make build
```

On Windows, run `make build` from **Git Bash** (the Makefile uses a POSIX shell);
a plain `cargo build --release` works from any shell too, the binary lands in
`target/release/`.

On first run, bwsshd adds an `Include` of the generated file to the top of your
`~/.ssh/config` automatically (only once), so plain `ssh host` picks up the
generated blocks. On Linux and macOS that line is an absolute path:

```
Include /home/you/.ssh/config.automatic.bw
```

On Windows OpenSSH resolves a relative `Include` against `~/.ssh`, so it is
written relative instead:

```
Include config.automatic.bw
```

</details>

<details>
<summary><strong>Usage</strong></summary>

<br>

Run once to generate the config now:

```sh
bwsshd
```

Run as a background daemon that regenerates on every key change:

```sh
bwsshd --watch
```

Add an icon to the notification area (Windows), the menu bar (macOS) or the
system tray (Linux). Its right-click menu shows the version and when bwsshd
last read the agent, and can resync now, open the generated config in a text
editor, or quit. On Windows and macOS a left click opens a small panel with
the version, commit, build date and sync status:

```sh
bwsshd --watch --tray
```

On Linux the icon uses StatusNotifierItem: KDE shows it out of the box, GNOME
needs the AppIndicator extension.

**Run at login (systemd user service)**

```sh
make install
```

This builds a stripped static binary into `~/.local/bin`, installs the user
service, and enables it. The service runs headless; on a desktop, add the icon
with `systemctl --user edit bwsshd` and an `ExecStart=` override ending in
`--watch --tray`.

Follow the logs with:

```sh
journalctl --user -u bwsshd -f
```

**Run at login (Windows)**

There is no systemd on Windows, and a classic Windows *service* is the wrong
tool here: it runs at boot as `LocalSystem` in session 0, where it would see
neither your per-session Bitwarden agent pipe nor your `~/.ssh`. The correct
equivalent of the Linux `systemctl --user` service is a **per-user Scheduled
Task at logon**, which the bundled script sets up for you (no admin needed):

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\install.ps1
```

It registers a task named `bwsshd` that runs `bwsshd --watch --tray` at logon, in your
session, **hidden** (no console window), and restarts it if it crashes. Logs go
to `%LOCALAPPDATA%\bwsshd\bwsshd.log`:

```powershell
Get-Content -Wait "$env:LOCALAPPDATA\bwsshd\bwsshd.log"
```

Remove it with:

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\uninstall.ps1
```

**Run at login (macOS)**

A per-user LaunchAgent does the job. Save this as
`~/Library/LaunchAgents/fr.infrasouveraine.bwsshd.plist`, adjusting the binary
path and your home directory:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key><string>fr.infrasouveraine.bwsshd</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/bwsshd</string>
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

The daemon polls the agent and rewrites the config only when the key set
changes.

When the Bitwarden app is closed the socket is absent, bwsshd clears the
generated config and removes its key directory until the app comes back.

</details>

<details>
<summary><strong>Flags</strong></summary>

<br>

| Flag | Default | Description |
|------|---------|-------------|
| `--sock` | auto-discover | SSH agent endpoint; empty auto-discovers the Bitwarden socket (Linux, macOS) or named pipe (Windows) |
| `--out` | `~/.ssh/config.automatic.bw` | Generated ssh_config file |
| `--keydir` | `~/.ssh/bwsshd-keys` | Where public keys are written |
| `--ssh-config` | `~/.ssh/config` | ssh_config to add the Include line to |
| `--watch` | off | Loop instead of a single run |
| `--tray` | off | Show a tray icon with the last sync, *Resync now*, *Open generated config* and *Quit* (implies `--watch`) |
| `--interval` | `10s` | Poll interval in watch mode (`30s`, `1m`, ...) |
| `--logfile` | stderr | Append logs to this file (used by the Windows scheduled task, which has no console) |
| `--version` | | Print the version and exit |

Auto-discovery also honors `BW_SSH_SOCK` and `BITWARDEN_SSH_AUTH_SOCK` if set.
On Windows, `--sock` must be a named pipe (`\\.\pipe\...`).

The single-dash spellings of the Go releases (`-watch`, `-interval 5s`, ...) are
still accepted, so existing service files keep working after an upgrade.

</details>

<details>
<summary><strong>Safety notes</strong></summary>

<br>

- Refuses to run as root, all files belong to your user.
- Config and keys are written `0600`, the key directory is `0700`.
- The key directory is fully managed by bwsshd via a marker file, and it
  refuses to touch a directory it does not own, so it can never delete your own
  keys.

</details>

---

## Contributing

Issues and pull requests are welcome.

For a bug or a behavior change, opening an issue first helps, a small
reproduction goes a long way.

A few things keep the pipeline happy:

- Keep the code `cargo fmt`-clean and `cargo clippy`-clean, CI checks both on
  Linux, macOS and Windows (`make lint` runs the same checks locally).
- Add a test when you change behavior, the filesystem paths are covered with
  `tempfile::TempDir` and an in-process SSH agent, follow the same style.
- Use [Conventional Commits](https://www.conventionalcommits.org) for commit
  messages (`fix:`, `feat:`, ...), the changelog and version bumps are generated
  from them automatically.

CI builds and tests every push and pull request, and releases are cut
automatically once a release PR is merged.

## Thanks

Thanks to [@Andralax](https://github.com/Andralax) for the careful bug reports
and review that shaped the first releases.

## License

MIT
