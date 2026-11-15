---
title: Flags
description: Every option bwsshd takes, and the environment it reads.
section: Reference
order: 1
---

```sh
bwsshd --watch --interval 30s
```

## All flags

| Flag | Default | Description |
|------|---------|-------------|
| `--sock` | auto-discover | SSH agent endpoint. Empty finds the Bitwarden socket or named pipe. |
| `--out` | `~/.ssh/config.automatic.bw` | Generated ssh_config file. |
| `--keydir` | `~/.ssh/bwsshd-keys` | Where public keys are written. |
| `--ssh-config` | `~/.ssh/config` | Config that receives the `Include` line. |
| `--watch` | off | Keep running and regenerate on every key change. |
| `--interval` | `10s` | Poll interval in watch mode (`30s`, `1m`, ...). |
| `--logfile` | stderr | Append logs to a file, for the hidden Windows task. |
| `--version` | | Print the version and exit. |

The single-dash spellings of the 0.x releases (`-watch`, `-interval 5s`, ...) are still accepted, so existing service files keep working after an upgrade.

## Environment

Auto-discovery checks `BW_SSH_SOCK`, then `BITWARDEN_SSH_AUTH_SOCK`, then the default paths below. An explicit `--sock` wins over all of them.

| Platform | Default endpoints, in order |
|----------|-----------------------------|
| Linux | `~/.bitwarden-ssh-agent.sock`, then the Flatpak and Snap sockets |
| macOS | `~/.bitwarden-ssh-agent.sock`, then `~/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock` |
| Windows | `\\.\pipe\openssh-ssh-agent` |

> On Windows, `--sock` must be a named pipe. bwsshd writes it with forward slashes in the config (`//./pipe/openssh-ssh-agent`), which is what the built-in OpenSSH client expects.
