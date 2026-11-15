---
title: Install
description: One-line installers, prebuilt archives, or Cargo.
section: Getting started
order: 3
---

## One-line install

On Linux and macOS:

```sh
curl -fsSL https://bwsshd.example/install.sh | sh
```

The script downloads the latest release for your platform into `~/.local/bin`, then sets bwsshd up to run at login: a systemd user service on Linux, a LaunchAgent on macOS. Set `BWSSHD_NO_SERVICE=1` to only install the binary, or `BWSSHD_BIN_DIR` to pick another folder.

On Windows, in PowerShell:

```powershell
irm https://bwsshd.example/install.ps1 | iex
```

It downloads the latest release and runs the bundled `install.ps1`, which copies `bwsshd.exe` to `%LOCALAPPDATA%\Programs\bwsshd` and registers the logon task. No administrator rights are needed.

> Prefer reading a script before piping it to a shell? Open [install.sh](/install.sh) or [install.ps1](/install.ps1) first, they are short.

## Prebuilt archives

Every [release](https://github.com/Dxsk/bwsshd/releases) ships one archive per platform, each for `amd64` and `arm64`:

| Platform | Archive |
|----------|---------|
| Linux (static, any distro) | `bwsshd_<version>_linux_<arch>.tar.gz` |
| macOS | `bwsshd_<version>_darwin_<arch>.tar.gz` |
| Windows | `bwsshd_<version>_windows_<arch>.zip` |

`checksums.txt` lists their SHA-256 sums. Each archive holds the binary plus the service helpers: a `Makefile` and systemd unit for Linux, `install.ps1` and `uninstall.ps1` for Windows.

## With Cargo

```sh
cargo install --locked --git https://github.com/Dxsk/bwsshd
```

## From source

```sh
git clone https://github.com/Dxsk/bwsshd
cd bwsshd
make build
```

The binary lands in `dist/`. On Windows run `make` from Git Bash, or use a plain `cargo build --release` from any shell.
