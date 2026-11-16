//! Which kind of file system a path is on, and whether the app may write there (SAV-11, TD-22).
//!
//! Writes rely on an atomic rename over the original. On `\\wsl$` / `\\wsl.localhost` (Windows
//! reaching into WSL), DrvFs (`/mnt/c` inside WSL) and other 9p mounts that was not verified, so
//! those classes stay read-only until the rename spike (T11) records a reliable result for them
//! and the class is added to [`VERIFIED_ATOMIC`]. Recognition needs no `unsafe`: Linux reads
//! `/proc/self/mountinfo`, Windows looks at the path prefix (A10).

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum FsClass {
    /// Any file system not listed below: writes follow the normal pipeline.
    Local,
    /// `\\wsl$\…` or `\\wsl.localhost\…` seen from Windows.
    WslUnc,
    /// Windows drives mounted inside WSL (`drvfs`, or `9p` with `aname=drvfs`).
    DrvFs,
    /// Other 9p / v9fs mounts.
    NineP,
}

impl FsClass {
    pub fn as_str(self) -> &'static str {
        match self {
            FsClass::Local => "local",
            FsClass::WslUnc => "\\\\wsl$",
            FsClass::DrvFs => "drvfs",
            FsClass::NineP => "9p",
        }
    }
}

/// Classes where the T11 spike recorded a reliable atomic rename. Change it only together with
/// the Journal entry that shows the measurement (summary.md of quire-v0-2, "Medições", T11).
/// 2026-11-16: none of them was measured, so all stay read-only (P6).
pub const VERIFIED_ATOMIC: &[FsClass] = &[];

/// Whether the app may write on a file system of this class.
pub fn writable(class: FsClass) -> bool {
    class == FsClass::Local || VERIFIED_ATOMIC.contains(&class)
}

/// Unescapes the octal escapes of `/proc/self/mountinfo` fields (`\040` for a space…).
fn unescape(field: &str) -> String {
    let b = field.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 4 <= b.len() {
            let digits = &b[i + 1..i + 4];
            if digits.iter().all(|d| (b'0'..=b'7').contains(d)) {
                let value = digits
                    .iter()
                    .fold(0u32, |acc, d| acc * 8 + u32::from(d - b'0'));
                if let Ok(byte) = u8::try_from(value) {
                    out.push(byte);
                    i += 4;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The class of `path` (absolute, already canonical) from the text of `/proc/self/mountinfo`:
/// the mount with the longest mount point that contains the path decides.
pub fn classify_mountinfo(mountinfo: &str, path: &Path) -> FsClass {
    let mut best: Option<(usize, FsClass)> = None;
    for line in mountinfo.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let Some(sep) = fields.iter().position(|f| *f == "-") else {
            continue;
        };
        let (Some(mount_point), Some(fstype)) = (fields.get(4), fields.get(sep + 1)) else {
            continue;
        };
        let mount_point = PathBuf::from(unescape(mount_point));
        if !path.starts_with(&mount_point) {
            continue;
        }
        let super_opts = fields.get(sep + 3).copied().unwrap_or("");
        let class = match *fstype {
            "drvfs" => FsClass::DrvFs,
            "9p" | "v9fs" if super_opts.split([',', ';']).any(|o| o == "aname=drvfs") => {
                FsClass::DrvFs
            }
            "9p" | "v9fs" => FsClass::NineP,
            _ => FsClass::Local,
        };
        let depth = mount_point.components().count();
        if best.is_none_or(|(d, _)| depth >= d) {
            best = Some((depth, class));
        }
    }
    best.map_or(FsClass::Local, |(_, c)| c)
}

/// The class of a Windows path from its prefix; separators, case and the `\\?\` forms do not
/// matter.
pub fn classify_windows_path(path: &str) -> FsClass {
    let p = path.replace('/', "\\").to_ascii_lowercase();
    let p = if let Some(rest) = p.strip_prefix("\\\\?\\unc\\") {
        format!("\\\\{rest}")
    } else if let Some(rest) = p.strip_prefix("\\\\?\\") {
        rest.to_owned()
    } else {
        p
    };
    if p.starts_with("\\\\wsl$\\") || p.starts_with("\\\\wsl.localhost\\") {
        FsClass::WslUnc
    } else {
        FsClass::Local
    }
}

/// The class of the file system holding `path` (or its folder, when the file does not exist).
pub fn classify(path: &Path) -> FsClass {
    let existing = path
        .ancestors()
        .find(|p| p.exists())
        .unwrap_or(path)
        .to_path_buf();
    #[cfg(windows)]
    {
        let raw = classify_windows_path(&path.to_string_lossy());
        if raw != FsClass::Local {
            return raw;
        }
        let canon = std::fs::canonicalize(&existing).unwrap_or(existing);
        classify_windows_path(&canon.to_string_lossy())
    }
    #[cfg(target_os = "linux")]
    {
        let canon = std::fs::canonicalize(&existing).unwrap_or(existing);
        match std::fs::read_to_string("/proc/self/mountinfo") {
            Ok(text) => classify_mountinfo(&text, &canon),
            // Unreadable: the other guarantees of the pipeline still hold (TD-22).
            Err(_) => FsClass::Local,
        }
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        let _ = existing;
        FsClass::Local
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOUNTINFO: &str = "\
22 1 0:21 / / rw,relatime shared:1 - ext4 /dev/sdb rw,discard,errors=remount-ro,data=ordered
61 22 0:37 /@home /home rw,noatime shared:191 - btrfs /dev/nvme0n1p2 rw,compress=zstd:1,ssd
167 22 0:60 / /tmp rw,noatime shared:209 - tmpfs tmpfs rw,inode64,huge=advise
90 22 0:52 / /mnt/c rw,noatime - 9p C:\\134 rw,dirsync,aname=drvfs;path=C:\\;uid=1000;gid=1000;symlinkroot=/mnt/,mmap,access=client,msize=65536,trans=fd,rfd=5,wfd=5
91 22 0:53 / /mnt/d rw,noatime - drvfs D:\\134 rw,uid=1000
92 22 0:54 / /mnt/wslg rw,relatime - 9p wslg rw,trans=virtio
93 167 0:55 / /tmp/My\\040Specs rw - tmpfs tmpfs rw
94 90 0:56 /work /mnt/c/Users/me/bind rw - ext4 /dev/sdc rw
";

    fn c(p: &str) -> FsClass {
        classify_mountinfo(MOUNTINFO, Path::new(p))
    }

    #[test]
    fn linux_mounts_decide_by_the_longest_mount_point() {
        assert_eq!(c("/home/me/repo/.specs/x/spec.md"), FsClass::Local);
        assert_eq!(c("/tmp/a.md"), FsClass::Local);
        assert_eq!(c("/srv/a.md"), FsClass::Local);
        assert_eq!(c("/mnt/c/Users/me/repo/spec.md"), FsClass::DrvFs);
        assert_eq!(c("/mnt/d/spec.md"), FsClass::DrvFs);
        assert_eq!(c("/mnt/wslg/x.md"), FsClass::NineP);
        // A space escaped as \040; the bind mount inside /mnt/c is ext4.
        assert_eq!(c("/tmp/My Specs/a.md"), FsClass::Local);
        assert_eq!(c("/mnt/c/Users/me/bind/spec.md"), FsClass::Local);
        // Prefix by components, not by text: /mnt/cx is not under /mnt/c.
        assert_eq!(c("/mnt/cx/spec.md"), FsClass::Local);
        assert_eq!(classify_mountinfo("", Path::new("/x")), FsClass::Local);
        assert_eq!(
            classify_mountinfo("garbage line\n", Path::new("/x")),
            FsClass::Local
        );
    }

    #[test]
    fn real_lines_seen_by_the_rename_spike() {
        // T11, this machine: btrfs and tmpfs.
        let seen = "61 43 0:37 /@home /home rw,noatime shared:191 - btrfs /dev/nvme0n1p2 rw,compress=zstd:1,ssd,discard=async,space_cache=v2,subvolid=257,subvol=/@home\n167 43 0:60 / /tmp rw,noatime shared:209 - tmpfs tmpfs rw,inode64,huge=advise\n";
        assert_eq!(
            classify_mountinfo(seen, Path::new("/home/a/spec.md")),
            FsClass::Local
        );
        assert_eq!(
            classify_mountinfo(seen, Path::new("/tmp/a/spec.md")),
            FsClass::Local
        );
    }

    #[test]
    fn windows_prefixes() {
        for p in [
            r"\\wsl$\Ubuntu\home\me\spec.md",
            r"\\wsl.localhost\Ubuntu\home\me\spec.md",
            r"\\?\UNC\wsl$\Ubuntu\x.md",
            r"\\?\UNC\wsl.localhost\Ubuntu\x.md",
            "//wsl$/Ubuntu/x.md",
            r"\\WSL$\Ubuntu\x.md",
            r"\\Wsl.LocalHost\Ubuntu\x.md",
        ] {
            assert_eq!(classify_windows_path(p), FsClass::WslUnc, "{p}");
        }
        for p in [
            r"C:\Users\me\spec.md",
            r"\\?\C:\Users\me\spec.md",
            r"\\server\share\spec.md",
            r"\\wslx\a.md",
            r"C:\wsl$\a.md",
        ] {
            assert_eq!(classify_windows_path(p), FsClass::Local, "{p}");
        }
    }

    #[test]
    fn only_local_is_writable_until_the_spike_verifies_a_class() {
        // P6 decision of 2026-11-16 (summary.md, T11): no class was measured.
        assert!(VERIFIED_ATOMIC.is_empty());
        assert!(writable(FsClass::Local));
        for class in [FsClass::WslUnc, FsClass::DrvFs, FsClass::NineP] {
            assert!(!writable(class), "{class:?}");
        }
    }

    #[test]
    fn the_folder_of_this_test_is_local() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(classify(&d.path().join("missing.md")), FsClass::Local);
    }
}
