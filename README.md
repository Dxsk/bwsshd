<div align="center">

# bwsshd

**Keep your `ssh_config` in sync with the SSH keys stored in your Bitwarden vault.**

*One key per host, signed from the vault, never written to disk.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Go](https://img.shields.io/github/go-mod/go-version/Dxsk/bwsshd?logo=go&logoColor=white)](go.mod)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-informational)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-dxsk-yellow?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/dxsk)

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

It also works the same whether your Bitwarden desktop app is the native package,
the Flatpak or the Snap, and whether your backend is Bitwarden cloud or a
self-hosted Vaultwarden.

The SSH agent socket is created by the desktop client, so the backend does not
change anything.

**Linux and Windows.** On Linux/macOS the agent is a unix socket; on Windows it
is the named pipe `\\.\pipe\openssh-ssh-agent`, which the Bitwarden desktop app
registers as the standard OpenSSH agent. bwsshd auto-discovers whichever applies,
and on Windows writes the generated config with the forward-slash paths and the
relative `Include` that the built-in OpenSSH client expects.

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

Add `[nobwsshd]` anywhere in the name to make bwsshd skip a key entirely.

---

<details>
<summary><strong>Install</strong></summary>

<br>

Grab a prebuilt binary from the [releases](https://github.com/Dxsk/bwsshd/releases)
(`.tar.gz` for Linux, `.zip` for Windows), or install with Go:

```sh
go install github.com/Dxsk/bwsshd@latest
```

Or build from source into `dist/`:

```sh
make build
```

On Windows, run `make build` from **Git Bash** (the Makefile uses a POSIX shell);
a plain `go build -o dist/bwsshd.exe .` works from any shell too.

On first run, bwsshd adds an `Include` of the generated file to the top of your
`~/.ssh/config` automatically (only once), so plain `ssh host` picks up the
generated blocks. On Linux that line is an absolute path:

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
bwsshd -watch
```

**Run at login (systemd user service)**

```sh
make install
```

This builds a stripped static binary into `~/.local/bin`, installs the user
service, and enables it.

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

It registers a task named `bwsshd` that runs `bwsshd -watch` at logon, in your
session, **hidden** (no console window), and restarts it if it crashes. Logs go
to `%LOCALAPPDATA%\bwsshd\bwsshd.log`:

```powershell
Get-Content -Wait "$env:LOCALAPPDATA\bwsshd\bwsshd.log"
```

Remove it with:

```powershell
powershell -ExecutionPolicy Bypass -File .\packaging\windows\uninstall.ps1
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
| `-sock` | auto-discover | SSH agent endpoint; empty auto-discovers the Bitwarden socket (native/Flatpak/Snap on Linux) or named pipe (Windows) |
| `-out` | `~/.ssh/config.automatic.bw` | Generated ssh_config file |
| `-keydir` | `~/.ssh/bwsshd-keys` | Where public keys are written |
| `-ssh-config` | `~/.ssh/config` | ssh_config to add the Include line to |
| `-watch` | off | Loop instead of a single run |
| `-interval` | `10s` | Poll interval in watch mode |

Auto-discovery also honors `BW_SSH_SOCK` and `BITWARDEN_SSH_AUTH_SOCK` if set.

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

<details>
<summary><strong>Requirements</strong></summary>

<br>

- Bitwarden desktop app with the SSH agent enabled
  (`Settings > SSH agent > Enable SSH agent`) - see
  [enabling the SSH agent](https://bitwarden.com/help/ssh-agent/#enable-ssh-agent)
- SSH key items in your vault, named with their target host - see
  [storing an SSH key](https://bitwarden.com/help/ssh-agent/#storing-an-ssh-key)
- An OpenSSH client. Linux/macOS ship one; on Windows it is the built-in OpenSSH
  client included with Windows 10/11

</details>

---

## Contributing

Issues and pull requests are welcome.

For a bug or a behavior change, opening an issue first helps, a small
reproduction goes a long way.

A few things keep the pipeline happy:

- Keep the code `gofmt`-clean and `go vet`-clean, CI checks both.
- Add a test when you change behavior, the filesystem paths are covered with
  `t.TempDir()` and an in-process SSH agent, follow the same style.
- Use [Conventional Commits](https://www.conventionalcommits.org) for commit
  messages (`fix:`, `feat:`, ...), the changelog and version bumps are generated
  from them automatically.

CI builds and tests every push and pull request, and releases are cut
automatically once a release PR is merged.

## Thanks

Thanks to [@Andralax](https://github.com/Andralax) for the careful bug reports
and review that shaped the first releases.

## Support

If this saved you some headaches, you can
[buy me a coffee](https://buymeacoffee.com/dxsk).

## License

MIT
