//! Identity of the content of a file (TD-13, A3): size and xxh3-128 of the raw bytes.
//!
//! Equality is by size and hash. `mtime` is kept for diagnostics only: an equal `mtime` does
//! not prove the content is the same (FAT/exFAT have 2 s granularity, tools preserve it), and a
//! different one does not prove it changed.

use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Fingerprint {
    pub len: u64,
    /// Modification time in nanoseconds since the epoch, when read from disk.
    pub mtime_ns: Option<i128>,
    pub hash: u128,
}

impl PartialEq for Fingerprint {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.hash == other.hash
    }
}

impl Fingerprint {
    /// The fingerprint of bytes about to be written (no `mtime`).
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self {
            len: bytes.len() as u64,
            mtime_ns: None,
            hash: xxhash_rust::xxh3::xxh3_128(bytes),
        }
    }

    /// Reads the file (following links). `None` when it does not exist.
    pub fn of(path: &Path) -> std::io::Result<Option<Self>> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let mtime_ns = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_nanos() as i128);
        Ok(Some(Self {
            mtime_ns,
            ..Self::of_bytes(&bytes)
        }))
    }

    /// Opaque text form for the UI, which hands it back on save: `<len>-<hash hex>`.
    pub fn token(&self) -> String {
        format!("{}-{:032x}", self.len, self.hash)
    }

    pub fn from_token(token: &str) -> Option<Self> {
        let (len, hash) = token.split_once('-')?;
        if hash.len() != 32 {
            return None;
        }
        Some(Self {
            len: len.parse().ok()?,
            mtime_ns: None,
            hash: u128::from_str_radix(hash, 16).ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_is_content_not_time() {
        let a = Fingerprint::of_bytes(b"abc");
        let b = Fingerprint {
            mtime_ns: Some(42),
            ..a
        };
        assert_eq!(a, b);
        assert_ne!(a, Fingerprint::of_bytes(b"abd"));
        assert_ne!(a, Fingerprint::of_bytes(b"abc "));
    }

    #[test]
    fn token_round_trips_and_rejects_garbage() {
        let a = Fingerprint::of_bytes("ação\r\n".as_bytes());
        assert_eq!(Fingerprint::from_token(&a.token()), Some(a));
        for bad in ["", "1", "x-00", "3-zz", "3-0123"] {
            assert_eq!(Fingerprint::from_token(bad), None, "{bad}");
        }
    }

    #[test]
    fn of_reads_the_disk_and_none_means_missing() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("a.md");
        assert_eq!(Fingerprint::of(&f).unwrap(), None);
        std::fs::write(&f, b"hello").unwrap();
        let fp = Fingerprint::of(&f).unwrap().unwrap();
        assert_eq!(fp, Fingerprint::of_bytes(b"hello"));
        assert!(fp.mtime_ns.is_some());
    }
}
