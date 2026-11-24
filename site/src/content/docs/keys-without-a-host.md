---
title: Keys without a host
description: Shared keys used on many servers, and how to wire them up.
section: Guides
order: 3
---

Some keys have no single target, for example a shared `ansible` key used on dozens of servers. bwsshd still writes their `.pub` file, and lists them as comments at the end of the generated config:

```sshconfig
# Keys with no hostname in their Bitwarden name. Add a manual Host
# block referencing the .pub below:
#   "ansible" -> /home/you/.ssh/bwsshd-keys/ansible.pub
```

Add a block for them in your own `~/.ssh/config`, below the `Include` line:

```sshconfig
Host *.lab.example.com
    IdentityAgent ~/.bitwarden-ssh-agent.sock
    IdentityFile ~/.ssh/bwsshd-keys/ansible.pub
    IdentitiesOnly yes
```

The `.pub` file is rewritten on every change and removed when the key leaves the vault, so point at it rather than copying it.
