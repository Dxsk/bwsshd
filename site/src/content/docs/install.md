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

The script downloads the latest release for your platform into `~/.local/bin`, then runs [`bwsshd install`](/docs/run-at-login/) to start it at login: a systemd user service on Linux, a LaunchAgent on macOS. Set `BWSSHD_NO_SERVICE=1` to only install the binary, or `BWSSHD_BIN_DIR` to pick another folder.

On Windows, in PowerShell:

```powershell
irm https://bwsshd.example/install.ps1 | iex
```

It downloads the latest release, copies `bwsshd.exe` to `%LOCALAPPDATA%\Programs\bwsshd` and runs `bwsshd install`, which registers the logon task. No administrator rights are needed.

## Windows installer

Prefer a regular setup? Each release has a `.msi` for `amd64` and `arm64`. It installs for your account only (no administrator rights), registers the logon task and starts bwsshd. Remove it from **Settings > Apps** like any other program.

> The `.msi` is not code-signed yet, so SmartScreen may warn on first run: **More info**, then **Run anyway**.

> Prefer reading a script before piping it to a shell? Open [install.sh](/install.sh) or [install.ps1](/install.ps1) first, they are short.

## Prebuilt archives

Every [release](https://github.com/Dxsk/bwsshd/releases) ships one archive per platform, each for `amd64` and `arm64`:

| Platform | Archive |
|----------|---------|
| Linux (static, any distro) | `bwsshd_<version>_linux_<arch>.tar.gz` |
| macOS | `bwsshd_<version>_darwin_<arch>.tar.gz` |
| Windows | `bwsshd_<version>_windows_<arch>.zip` or `.msi` |

`checksums.txt` lists their SHA-256 sums. Each archive holds the binary, the README and the license; run `bwsshd install` from where you put the binary to start it at login.

## With Cargo

```sh
cargo install bwsshd
bwsshd install
```

The second line starts it at login, from `~/.cargo/bin`.

## From source

```sh
git clone https://github.com/Dxsk/bwsshd
cd bwsshd
make build
```

The binary lands in `dist/`. On Windows run `make` from Git Bash, or use a plain `cargo build --release` from any shell.
