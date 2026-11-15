---
title: Quick start
description: From a fresh install to your first single-key connection.
section: Getting started
order: 3
---

## 1. Name a key in Bitwarden

Rename an SSH key item so its name contains the host, for example `debian@pve1.example.com`. The [naming convention](/docs/naming/) has all the forms.

## 2. Run bwsshd once

With the Bitwarden desktop app open and unlocked:

```sh
bwsshd
```

```text
bwsshd: added Include /home/you/.ssh/config.automatic.bw to /home/you/.ssh/config
bwsshd: wrote /home/you/.ssh/config.automatic.bw: 1 host(s), 0 key(s) without a hostname
```

On first run bwsshd prepends an `Include` of the generated file to your `~/.ssh/config`, once. It goes at the top because SSH uses the first value it finds for each option.

## 3. Connect

```sh
ssh pve1.example.com
```

Bitwarden pops up to approve the signature, and SSH offers exactly one key.

## 4. Keep it running

Run it as a daemon so the config follows your vault:

```sh
bwsshd --watch
```

To start it at login, see [Run at login](/docs/run-at-login/).
