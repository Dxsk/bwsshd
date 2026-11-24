---
title: Safety notes
description: The guarantees bwsshd keeps around your ~/.ssh.
section: Reference
order: 3
---

- **No private keys on disk.** Only public keys are written. Signing happens in the Bitwarden app, which asks for approval.
- **Never as root.** bwsshd refuses to run as root, so every file it writes belongs to you.
- **Private permissions.** The config and keys are written `0600`, the key folder is `0700`.
- **Atomic writes.** Every file is written to a temporary file next to it and renamed into place. A crash never leaves a half-written `~/.ssh/config`.
- **A folder it owns, and only that.** The key folder is managed through a marker file. bwsshd refuses to use a non-empty folder without the marker (such as `--keydir ~/.ssh`) or a symlinked one, so it can never delete your own keys.

Found a problem? Please open an issue on [GitHub](https://github.com/Dxsk/bwsshd/issues).
