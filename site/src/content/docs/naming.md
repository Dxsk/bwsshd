---
title: Naming convention
description: How a Bitwarden item name becomes a Host block.
section: Guides
order: 1
---

The name you give an SSH key item in Bitwarden becomes the key comment in the agent. bwsshd reads that comment and uses the **first token shaped like `[user@]host[:port]`**:

- the first hostname or IP becomes the `Host` and `HostName`,
- an optional `user@` prefix sets `User`,
- a trailing `:port` sets `Port` (otherwise the SSH default applies).

## Examples

| Bitwarden name | Host | User | Port |
|----------------|------|------|------|
| `pve1.example.com` | `pve1.example.com` | | |
| `ssh key ops pve1.example.com` | `pve1.example.com` | | |
| `prod pve1.example.com root` | `pve1.example.com` | | |
| `debian@pve1.example.com` | `pve1.example.com` | `debian` | |
| `debian@pve1.example.com:2222` | `pve1.example.com` | `debian` | `2222` |
| `db admin@10.0.0.5` | `10.0.0.5` | `admin` | |
| `deploy v1.2 server` | none | | |

Try your own names in the [playground](/#naming).

## Pick your own Host name

To type a short name after `ssh` while connecting to an address, write `(alias,address)` where the host would go:

```text
debian@(vps1,10.0.0.1):23487
```

```sshconfig
Host vps1
    HostName 10.0.0.1
    User debian
    Port 23487
    IdentityFile ~/.ssh/bwsshd-keys/vps1.pub
    IdentitiesOnly yes
```

Then `ssh vps1` connects to `10.0.0.1` on port 23487, as `debian`. The `user@` prefix and the `:port` suffix stay optional, as in the plain form.

| Bitwarden name | Host | HostName |
|----------------|------|----------|
| `(vps1,10.0.0.1)` | `vps1` | `10.0.0.1` |
| `debian@(vps2,[2001:db8::20]):23487` | `vps2` | `2001:db8::20` |
| `(nas,nas)` | `nas` | `nas` |
| `(db.prod,db1.internal.example.com)` | `db.prod` | `db1.internal.example.com` |

- The **alias** becomes the `Host` pattern and the `.pub` file name, so it only takes letters, digits, `.`, `-` and `_`. Wildcards such as `*` or `!` are rejected.
- The **address** is an IP (IPv6 with or without brackets) or a DNS name. Since the form is explicit, a single-label name like `nas` is accepted here.
- A `(…)` token that does not parse is ignored, and bwsshd keeps looking for a target in the rest of the name.

## What counts as a host

A token is a host when it is an IP literal (IPv4, or IPv6 with `[addr]:port` for a port), or a dotted name whose last label is alphabetic. That is why version strings such as `v1.2` or `1.0.3` are never mistaken for a hostname.

Tokens containing `/`, `\` or spaces are rejected, which also keeps derived file names safe.

## Skipping a key

Add `[nobwsshd]` anywhere in the name and bwsshd ignores the key entirely: no `.pub`, no block.

## Duplicates

When two keys map to the same host, the first one in fingerprint order wins and bwsshd logs the one it ignored. Rename or tag the extra key to make the choice explicit.
