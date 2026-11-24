//! A tiny SSH agent protocol client. bwsshd only ever needs the key list, so
//! this implements a single request (`REQUEST_IDENTITIES`) instead of pulling
//! in a full agent library.
//!
//! Wire format (draft-miller-ssh-agent): every message is a big-endian `u32`
//! length followed by a type byte and its payload.

use std::io::{self, Read, Write};

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use sha2::{Digest, Sha256};

const REQUEST_IDENTITIES: u8 = 11;
const IDENTITIES_ANSWER: u8 = 12;

/// OpenSSH caps agent messages at 256 KiB; anything larger is a broken peer.
const MAX_MESSAGE: usize = 256 * 1024;

/// A public key as listed by the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Key {
    /// Wire-format public key blob.
    pub blob: Vec<u8>,
    /// Key comment; for Bitwarden this is the vault item name.
    pub comment: String,
}

impl Key {
    pub fn new(blob: Vec<u8>, comment: impl Into<String>) -> Self {
        Self {
            blob,
            comment: comment.into(),
        }
    }

    /// Key algorithm name (`ssh-ed25519`, `ssh-rsa`, ...), the first string in
    /// the blob.
    pub fn format(&self) -> String {
        let mut r = Reader(&self.blob);
        r.string()
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .unwrap_or_default()
    }

    /// OpenSSH `SHA256:...` fingerprint, as printed by `ssh-keygen -l`.
    pub fn fingerprint_sha256(&self) -> String {
        format!(
            "SHA256:{}",
            STANDARD_NO_PAD.encode(Sha256::digest(&self.blob))
        )
    }

    /// The key in `authorized_keys` / `.pub` format.
    pub fn to_openssh(&self) -> String {
        let mut s = format!("{} {}", self.format(), STANDARD.encode(&self.blob));
        if !self.comment.is_empty() {
            s.push(' ');
            s.push_str(&self.comment);
        }
        s
    }
}

/// Asks the agent on `stream` for its identities.
pub fn list_keys<S: Read + Write>(stream: &mut S) -> io::Result<Vec<Key>> {
    stream.write_all(&1u32.to_be_bytes())?;
    stream.write_all(&[REQUEST_IDENTITIES])?;
    stream.flush()?;

    let msg = read_message(stream)?;
    let (&kind, body) = msg
        .split_first()
        .ok_or_else(|| invalid("empty agent reply"))?;
    if kind != IDENTITIES_ANSWER {
        return Err(invalid(format!("unexpected agent reply type {kind}")));
    }
    parse_identities(body)
}

fn read_message<R: Read>(r: &mut R) -> io::Result<Vec<u8>> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len) as usize;
    if len == 0 || len > MAX_MESSAGE {
        return Err(invalid(format!("agent message length {len} out of range")));
    }
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

fn parse_identities(body: &[u8]) -> io::Result<Vec<Key>> {
    let mut r = Reader(body);
    let n = r.u32()?;
    let mut keys = Vec::new();
    for _ in 0..n {
        let blob = r.string()?.to_vec();
        let comment = String::from_utf8_lossy(r.string()?).into_owned();
        keys.push(Key { blob, comment });
    }
    Ok(keys)
}

/// Encodes an `IDENTITIES_ANSWER` message (with its length prefix). Used by the
/// test agents; kept here so the encoder and decoder stay side by side.
pub fn encode_identities_answer(keys: &[Key]) -> Vec<u8> {
    let mut body = vec![IDENTITIES_ANSWER];
    body.extend_from_slice(&(keys.len() as u32).to_be_bytes());
    for k in keys {
        put_string(&mut body, &k.blob);
        put_string(&mut body, k.comment.as_bytes());
    }
    let mut msg = (body.len() as u32).to_be_bytes().to_vec();
    msg.extend_from_slice(&body);
    msg
}

/// Appends an SSH wire `string` (u32 length + bytes).
pub fn put_string(buf: &mut Vec<u8>, s: &[u8]) {
    buf.extend_from_slice(&(s.len() as u32).to_be_bytes());
    buf.extend_from_slice(s);
}

struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.0.len() < n {
            return Err(invalid("truncated agent message"));
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }

    fn u32(&mut self) -> io::Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn string(&mut self) -> io::Result<&'a [u8]> {
        let n = self.u32()? as usize;
        self.take(n)
    }
}

fn invalid(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.into())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A well-formed ed25519 public key blob, unique per call. The key bytes
    /// only need to be distinct, never valid curve points.
    pub fn blob() -> Vec<u8> {
        static N: AtomicU64 = AtomicU64::new(0);
        let seed = N.fetch_add(1, Ordering::Relaxed);
        let key = Sha256::digest(seed.to_le_bytes());
        let mut b = Vec::new();
        put_string(&mut b, b"ssh-ed25519");
        put_string(&mut b, &key);
        b
    }

    pub fn key(comment: &str) -> Key {
        Key::new(blob(), comment)
    }

    /// A fake duplex stream: reads come from `input`, writes are captured.
    struct Duplex {
        input: Cursor<Vec<u8>>,
        output: Vec<u8>,
    }

    impl Read for Duplex {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.input.read(buf)
        }
    }

    impl Write for Duplex {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.output.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn list_keys_round_trip() {
        let keys = vec![key("debian@host.fr:2222"), key("ansible")];
        let mut s = Duplex {
            input: Cursor::new(encode_identities_answer(&keys)),
            output: Vec::new(),
        };
        assert_eq!(list_keys(&mut s).unwrap(), keys);
        assert_eq!(s.output, [0, 0, 0, 1, REQUEST_IDENTITIES]);
    }

    #[test]
    fn list_keys_rejects_failure_and_garbage() {
        // SSH_AGENT_FAILURE (5)
        let mut s = Duplex {
            input: Cursor::new(vec![0, 0, 0, 1, 5]),
            output: Vec::new(),
        };
        assert!(list_keys(&mut s).is_err());

        // Claims one key but carries nothing.
        let mut s = Duplex {
            input: Cursor::new(vec![0, 0, 0, 5, IDENTITIES_ANSWER, 0, 0, 0, 1]),
            output: Vec::new(),
        };
        assert!(list_keys(&mut s).is_err());
    }

    #[test]
    fn openssh_format() {
        let k = key("prod");
        assert_eq!(k.format(), "ssh-ed25519");
        let s = k.to_openssh();
        assert!(s.starts_with("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5"), "{s}");
        assert!(s.ends_with(" prod"));
        assert!(!Key::new(k.blob.clone(), "").to_openssh().ends_with(' '));
    }

    #[test]
    fn fingerprint_is_openssh_style() {
        // ssh-keygen prints unpadded base64 of the SHA-256 of the blob: 43 chars.
        let fp = key("").fingerprint_sha256();
        let b64 = fp.strip_prefix("SHA256:").expect("SHA256: prefix");
        assert_eq!(b64.len(), 43, "{fp}");
        assert!(!b64.ends_with('='));
    }
}
