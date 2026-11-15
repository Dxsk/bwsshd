---
title: Introduction
description: What bwsshd does, and why your SSH agent needs it.
section: Getting started
order: 1
---

The Bitwarden desktop app can act as an [SSH agent](https://bitwarden.com/help/ssh-agent/): your private keys never leave the vault and signing happens in the app, behind its approval prompt.

The catch is that the agent offers **every** key it holds, in no host-aware order. With many keys and a hardened server (`MaxAuthTries 3` or less), SSH runs out of attempts before it reaches the right key:

```text
Received disconnect from 203.0.113.10 port 22:2: Too many authentication failures
```

## Before you start

- The Bitwarden desktop app with its SSH agent enabled, and on Windows the OpenSSH Authentication Agent service disabled. [Set up the Bitwarden agent](/docs/bitwarden-setup/) walks through it, with links to Bitwarden's own [SSH Agent guide](https://bitwarden.com/help/ssh-agent/).
- SSH key items in your vault, named after their target host. See the [naming convention](/docs/naming/).
- An OpenSSH client. Linux and macOS ship one; Windows 10 and 11 include it.

## What bwsshd does

bwsshd reads the key list from the agent and writes one `Host` block per key, with `IdentitiesOnly yes`, so SSH offers a single, deterministic key per host:

```sshconfig
Host pve1.example.com
    HostName pve1.example.com
    User debian
    IdentityAgent ~/.bitwarden-ssh-agent.sock
    IdentityFile ~/.ssh/bwsshd-keys/pve1.example.com.pub
    IdentitiesOnly yes
```

The private key stays in the vault. The `.pub` file only tells SSH which key to ask the agent to sign with. When you connect, Bitwarden asks you to approve the signature.

## Every machine, same setup

Your keys already follow you through the vault to every device where Bitwarden runs, but the `ssh_config` that points at them used to be copied and fixed by hand on each one. With bwsshd, a new desktop or laptop only needs the Bitwarden app and one install line: the config rebuilds itself from the vault, and a key renamed on one device updates the config everywhere.

## Where it runs

- **Linux**: native, Flatpak and Snap installs of the desktop app.
- **macOS**: the `.dmg` download and the Mac App Store version.
- **Windows**: the OpenSSH named pipe that Bitwarden registers.

The SSH agent socket is created by the desktop app, so the backend does not matter: Bitwarden cloud and a self-hosted Vaultwarden behave the same.
