---
title: Set up the Bitwarden agent
description: Enable the SSH agent in the Bitwarden desktop app and add your keys, before installing bwsshd.
section: Getting started
order: 2
---

bwsshd reads the keys that the Bitwarden desktop app serves over its SSH agent, so the agent has to be on first. Bitwarden documents every step in its own guide: [SSH Agent](https://bitwarden.com/help/ssh-agent/) and [About SSH](https://bitwarden.com/help/about-ssh/). This page sums up what matters for bwsshd.

## 1. Enable the SSH agent

In the Bitwarden desktop app, open **Settings** and turn on **Enable SSH agent**. Then choose when Bitwarden should ask you to authorize a key with **Ask for authorization when using SSH agent**.

See [Enable SSH Agent](https://bitwarden.com/help/ssh-agent/#enable-ssh-agent) in the Bitwarden docs.

## 2. Free the agent slot on Windows

On Windows, Bitwarden serves the agent on the standard OpenSSH pipe, so the built-in OpenSSH agent must stop claiming it:

1. Open **Services** and find **OpenSSH Authentication Agent**.
2. Set **Startup type** to **Disabled**.
3. Select **Apply**, then **OK**.

Linux and macOS need nothing here. See [Configure your system](https://bitwarden.com/help/ssh-agent/#configure-your-system-to-use-bitwarden-ssh-agent) in the Bitwarden docs.

## 3. Add your keys

Create an SSH key item with **New**, then **SSH key**, or paste an existing private key to import it. Name each item after the host it opens, for example `debian@pve1.example.com`: that name is what bwsshd turns into a `Host` block. The [naming convention](/docs/naming/) has every form.

See [Create a new SSH key](https://bitwarden.com/help/about-ssh/#create-a-new-ssh-key) and [Import key to Bitwarden](https://bitwarden.com/help/about-ssh/#import-key-to-bitwarden) in the Bitwarden docs.

## Do I need SSH_AUTH_SOCK?

Not for the hosts bwsshd manages: every generated block sets `IdentityAgent` to the Bitwarden socket itself. The Bitwarden guide also has you export `SSH_AUTH_SOCK`; do it if you want other tools (`git`, `ssh-add`, agent forwarding) to use the Bitwarden agent too.

| Install | Socket |
|---------|--------|
| Linux | `~/.bitwarden-ssh-agent.sock` |
| Linux, Snap | `~/snap/bitwarden/current/.bitwarden-ssh-agent.sock` |
| Linux, Flatpak | `~/.var/app/com.bitwarden.desktop/data/.bitwarden-ssh-agent.sock` |
| macOS, `.dmg` | `~/.bitwarden-ssh-agent.sock` |
| macOS, App Store | `~/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock` |
| Windows | `\\.\pipe\openssh-ssh-agent` |

bwsshd finds all of these on its own.

## Check that it works

List the keys the agent serves, pointing at your socket if `SSH_AUTH_SOCK` is not set:

```sh
SSH_AUTH_SOCK=~/.bitwarden-ssh-agent.sock ssh-add -L
```

On Windows, with the OpenSSH service disabled, a plain `ssh-add -L` talks to Bitwarden.

## Locked vault

While you are logged in, the agent keeps listing keys even when the vault is locked; only signing asks you to unlock. bwsshd only lists keys, so your config stays in place while the vault is locked. When you log out or close the app, the agent stops and bwsshd clears its config until it comes back.
