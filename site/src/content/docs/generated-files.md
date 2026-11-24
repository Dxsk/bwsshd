---
title: Generated files
description: What bwsshd writes, where, and what it never touches.
section: Reference
order: 2
---

## ~/.ssh/config.automatic.bw

The generated config: one `Host` block per mapped key, then the unmapped keys as comments. It is rewritten only when the key set or a key name changes. When the agent is gone it is reduced to a single comment, so your `Include` never points at a missing file.

## ~/.ssh/bwsshd-keys/

One `.pub` per key, named after its host (`pve1.example.com.pub`) or, for unmapped keys, after the sanitized key name. Colliding names get a `-2`, `-3` suffix instead of overwriting each other. Stale files are removed on every run.

The folder holds a `.bwsshd` marker file. bwsshd only ever creates, cleans or removes a folder that carries it.

## ~/.ssh/config

bwsshd adds two lines at the top, once:

```sshconfig
# Added by bwsshd
Include /home/you/.ssh/config.automatic.bw
```

On Windows, OpenSSH resolves a relative `Include` against `~/.ssh` and mangles backslashes, so the line is written relative instead: `Include config.automatic.bw`.

The rest of your config is never modified.
