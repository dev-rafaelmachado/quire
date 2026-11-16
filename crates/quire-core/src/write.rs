//! Safe writes of spec files (TD-12): the only way the app writes a file of a spec.
//!
//! The pipeline never writes into the original: it writes a hidden temporary file in the same
//! folder, forces it to disk, renames it over the original and forces the folder to disk. A
//! failure at any step leaves the original intact and removes the temporary file of that call
//! (SAV-01, SAV-02). Text keeps its format: BOM, line endings and final newline (SAV-03), and
//! the file keeps its permissions (SAV-04); a symbolic link keeps pointing to the file that
//! received the content (SAV-05). Two checkpoints, right before the temporary file and right
//! before the rename, let the caller refuse the write when the disk changed (CNF-03, T13).

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::fingerprint::Fingerprint;

/// Files above this size are not editable (A8, confirmed by the T9 spike).
pub const EDIT_LIMIT_BYTES: usize = 2 * 1024 * 1024;

const BOM: &[u8] = b"\xEF\xBB\xBF";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum Eol {
    Lf,
    Crlf,
}

impl Eol {
    fn as_str(self) -> &'static str {
        match self {
            Eol::Lf => "\n",
            Eol::Crlf => "\r\n",
        }
    }
}

/// How a text file is laid out on disk, so a save writes it back the same way (SAV-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TextFormat {
    /// Starts with a UTF-8 byte order mark.
    pub bom: bool,
    /// The line ending written on save: the one most used in the file (A4).
    pub eol: Eol,
    /// The file mixes line endings (CRLF and LF, or a lone CR). Saving normalizes to `eol`,
    /// which the user confirms on the first save (A4, SAV-03).
    pub mixed: bool,
    /// Ends with a line ending (informative: the text itself carries it).
    pub final_newline: bool,
}

/// Why a file cannot be opened in the editor (EDT-08).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error, serde::Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum NotEditable {
    #[error("the file is not valid UTF-8")]
    NotUtf8,
    #[error("the file looks binary")]
    Binary,
    #[error("the file is larger than the editing limit")]
    TooLarge,
}

/// Splits the bytes of a file into the text the editor shows (LF line endings, no BOM) and
/// the format to write it back with.
pub fn decode(bytes: &[u8]) -> Result<(String, TextFormat), NotEditable> {
    if bytes.len() > EDIT_LIMIT_BYTES {
        return Err(NotEditable::TooLarge);
    }
    let (bom, body) = match bytes.strip_prefix(BOM) {
        Some(rest) => (true, rest),
        None => (false, bytes),
    };
    if body.contains(&0) {
        return Err(NotEditable::Binary);
    }
    let text = std::str::from_utf8(body).map_err(|_| NotEditable::NotUtf8)?;
    let (mut crlf, mut lf, mut lone_cr) = (0usize, 0usize, 0usize);
    let mut first: Option<Eol> = None;
    let b = text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\r' if b.get(i + 1) == Some(&b'\n') => {
                crlf += 1;
                first.get_or_insert(Eol::Crlf);
                i += 2;
                continue;
            }
            b'\r' => lone_cr += 1,
            b'\n' => {
                lf += 1;
                first.get_or_insert(Eol::Lf);
            }
            _ => {}
        }
        i += 1;
    }
    // Ties go to the first line ending of the file (spec, edge cases).
    let eol = match crlf.cmp(&lf) {
        std::cmp::Ordering::Greater => Eol::Crlf,
        std::cmp::Ordering::Less => Eol::Lf,
        std::cmp::Ordering::Equal => first.unwrap_or(Eol::Lf),
    };
    let mixed = (crlf > 0 && lf > 0) || lone_cr > 0;
    let final_newline = text.ends_with('\n') || text.ends_with('\r');
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    Ok((
        normalized,
        TextFormat {
            bom,
            eol,
            mixed,
            final_newline,
        },
    ))
}

/// The bytes to write for `text` (LF line endings, as the editor keeps it) in `format`.
/// `encode(decode(b))` gives back `b` exactly whenever `b` does not mix line endings.
pub fn encode(text: &str, format: &TextFormat) -> Vec<u8> {
    let body = match format.eol {
        Eol::Lf => text.to_owned(),
        Eol::Crlf => text.replace('\n', format.eol.as_str()),
    };
    let mut out = Vec::with_capacity(body.len() + BOM.len());
    if format.bom {
        out.extend_from_slice(BOM);
    }
    out.extend_from_slice(body.as_bytes());
    out
}

/// Why a write did not happen. In every case the original file is untouched.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WriteError {
    /// The name is not a Markdown file of the spec folder (SAV-09).
    #[error("refused: {0}")]
    Refused(String),
    #[error("the symbolic link points to a missing file")]
    BrokenLink,
    #[error("the file is read-only")]
    ReadOnly,
    #[error("the file belongs to another user")]
    ForeignOwner,
    /// The file system of the path has no verified atomic rename (SAV-11, TD-22).
    #[error("writes are disabled on this file system ({0})")]
    UnreliableFs(String),
    /// The disk is not what the caller loaded (CNF-02, CNF-03). `disk` is `None` when the file
    /// does not exist.
    #[error("the file changed on disk")]
    Conflict { disk: Option<Fingerprint> },
    /// Another process holds the file open without allowing the replacement (SAV-08, Windows).
    #[error("the file is in use by another program")]
    InUse,
    #[error("{step}: {message}")]
    Io { step: Step, message: String },
}

impl WriteError {
    /// Stable code for the UI (DEB-02).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Refused(_) => "write_refused",
            Self::BrokenLink => "write_broken_link",
            Self::ReadOnly => "write_read_only",
            Self::ForeignOwner => "write_foreign_owner",
            Self::UnreliableFs(_) => "write_unreliable_fs",
            Self::Conflict { .. } => "write_conflict",
            Self::InUse => "write_in_use",
            Self::Io { .. } => "write_io",
        }
    }
}

/// Steps of the pipeline, named in errors and used by the failure-injection tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Resolve,
    CreateTemp,
    WriteTemp,
    Permissions,
    SyncTemp,
    Rename,
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Step::Resolve => "resolving the file",
            Step::CreateTemp => "creating the temporary file",
            Step::WriteTemp => "writing the temporary file",
            Step::Permissions => "copying the permissions",
            Step::SyncTemp => "flushing the temporary file to disk",
            Step::Rename => "replacing the file",
        })
    }
}

/// Where the caller can stop a write (T13 fills them with the fingerprint checks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checkpoint {
    /// Before the temporary file exists (first check, CNF-03).
    BeforeTemp,
    /// After the temporary file is on disk, right before the rename (second check, CNF-03).
    BeforeRename,
}

/// A check run at both checkpoints, with the path that will be replaced.
pub trait Guard {
    fn check(&self, at: Checkpoint, target: &Path) -> Result<(), WriteError>;
}

/// No check: used by tests of the bare pipeline.
pub struct NoGuard;
impl Guard for NoGuard {
    fn check(&self, _: Checkpoint, _: &Path) -> Result<(), WriteError> {
        Ok(())
    }
}

/// The file operations of the pipeline. The real one is [`RealFs`]; tests inject failures.
pub(crate) trait Fs {
    fn create_new(&self, path: &Path) -> std::io::Result<File>;
    fn write_all(&self, file: &mut File, bytes: &[u8]) -> std::io::Result<()>;
    fn set_permissions(&self, path: &Path, perm: fs::Permissions) -> std::io::Result<()>;
    fn sync(&self, file: &File) -> std::io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    /// Creates `to` as a second name of `from`; fails if `to` exists (no-overwrite create).
    fn hard_link(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn sync_dir(&self, dir: &Path) -> std::io::Result<()>;
    /// Whether `ours` (just created by this process) and `theirs` have the same owner.
    fn same_owner(&self, ours: &Path, theirs: &Path) -> std::io::Result<bool>;
}

pub(crate) struct RealFs;

impl Fs for RealFs {
    fn create_new(&self, path: &Path) -> std::io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }
    fn write_all(&self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
        file.write_all(bytes)
    }
    fn set_permissions(&self, path: &Path, perm: fs::Permissions) -> std::io::Result<()> {
        fs::set_permissions(path, perm)
    }
    fn sync(&self, file: &File) -> std::io::Result<()> {
        file.sync_all()
    }
    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        fs::rename(from, to)
    }
    fn hard_link(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        fs::hard_link(from, to)
    }
    #[cfg(unix)]
    fn sync_dir(&self, dir: &Path) -> std::io::Result<()> {
        File::open(dir)?.sync_all()
    }
    #[cfg(not(unix))]
    fn sync_dir(&self, _dir: &Path) -> std::io::Result<()> {
        // No folder handle to flush without FFI; NTFS journals the rename (design, step 8).
        Ok(())
    }
    #[cfg(unix)]
    fn same_owner(&self, ours: &Path, theirs: &Path) -> std::io::Result<bool> {
        use std::os::unix::fs::MetadataExt;
        Ok(fs::metadata(ours)?.uid() == fs::metadata(theirs)?.uid())
    }
    #[cfg(not(unix))]
    fn same_owner(&self, _ours: &Path, _theirs: &Path) -> std::io::Result<bool> {
        Ok(true)
    }
}

/// Prefix and suffix of the temporary files of the pipeline:
/// `.<name>.quire-<pid>-<counter>.tmp`, hidden so the watcher and the file list skip them.
pub fn temp_name(name: &str, pid: u32, counter: u64) -> String {
    format!(".{name}.quire-{pid}-{counter}.tmp")
}

/// A temporary file left by a crash between the write and the rename (SAV-10).
pub fn is_quire_temp(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('.').and_then(|r| r.strip_suffix(".tmp")) else {
        return false;
    };
    let Some(pos) = rest.rfind(".quire-") else {
        return false;
    };
    let (base, tail) = (&rest[..pos], &rest[pos + ".quire-".len()..]);
    let mut parts = tail.split('-');
    let numeric =
        |p: Option<&str>| p.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
    !base.is_empty() && numeric(parts.next()) && numeric(parts.next()) && parts.next().is_none()
}

/// Temporary files of the pipeline left in `dir` (SAV-10). Never removed by the app on its own.
pub fn orphan_temps(dir: &Path) -> std::io::Result<Vec<String>> {
    let mut out: Vec<String> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| is_quire_temp(n))
        .collect();
    out.sort();
    Ok(out)
}

/// Removes one orphan temporary file, on an explicit click (SAV-10). Only names of the
/// pipeline's temporary files are accepted.
pub fn remove_orphan_temp(dir: &Path, name: &str) -> Result<(), WriteError> {
    if !is_quire_temp(name) || name.contains(['/', '\\']) {
        return Err(WriteError::Refused(format!(
            "{name} is not a temporary file of Quire"
        )));
    }
    fs::remove_file(dir.join(name)).map_err(|e| WriteError::Io {
        step: Step::Resolve,
        message: e.to_string(),
    })
}

/// The path a write command may replace, or why not (SAV-09): one plain name, a Markdown
/// file, not one of the JSONL logs, not a temporary file, and listed in the spec folder.
pub fn editable_path(
    spec_dir: &Path,
    name: &str,
    listed: &[String],
) -> Result<PathBuf, WriteError> {
    let refuse = |why: &str| Err(WriteError::Refused(format!("{name}: {why}")));
    let mut components = Path::new(name).components();
    let single = matches!(
        (components.next(), components.next()),
        (Some(std::path::Component::Normal(_)), None)
    );
    if !single || name.contains(['/', '\\']) {
        return refuse("not a file name of the spec folder");
    }
    let lower = name.to_ascii_lowercase();
    if lower == "gates.jsonl" || lower == "lessons.jsonl" || lower.ends_with(".jsonl") {
        return refuse("gates.jsonl and lessons.jsonl are read-only");
    }
    if !lower.ends_with(".md") {
        return refuse("only Markdown files are editable");
    }
    if is_quire_temp(name) {
        return refuse("temporary file");
    }
    if !listed.iter().any(|l| l == name) {
        return refuse("not listed in the spec folder");
    }
    Ok(spec_dir.join(name))
}

/// What `save` expects to find, besides the checks of the guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expect {
    /// The file exists (or is a link to one): replace it.
    Existing,
    /// The file must not exist: create it (conflict copy, CNF-08; confirmed re-creation, CNF-07).
    Absent,
}

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Follows a symbolic link to the file that receives the content (SAV-05).
fn resolve(target: &Path, expect: Expect) -> Result<PathBuf, WriteError> {
    match fs::symlink_metadata(target) {
        Ok(m) if m.file_type().is_symlink() => fs::canonicalize(target).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                WriteError::BrokenLink
            } else {
                WriteError::Io {
                    step: Step::Resolve,
                    message: e.to_string(),
                }
            }
        }),
        Ok(m) if m.is_dir() => Err(WriteError::Refused(format!(
            "{} is a folder",
            target.display()
        ))),
        Ok(_) => Ok(target.to_path_buf()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound && expect == Expect::Absent => {
            Ok(target.to_path_buf())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Err(WriteError::Conflict { disk: None })
        }
        Err(e) => Err(WriteError::Io {
            step: Step::Resolve,
            message: e.to_string(),
        }),
    }
}

/// Writes `bytes` to `target` through the pipeline. Returns the fingerprint of what was
/// written (computed from the bytes, without reading the disk again).
pub fn save(
    target: &Path,
    bytes: &[u8],
    expect: Expect,
    guard: &dyn Guard,
) -> Result<Fingerprint, WriteError> {
    save_with(&RealFs, target, bytes, expect, guard)
}

/// `save` with the file operations given; the failure-injection tests use it.
pub(crate) fn save_with(
    fs_ops: &dyn Fs,
    target: &Path,
    bytes: &[u8],
    expect: Expect,
    guard: &dyn Guard,
) -> Result<Fingerprint, WriteError> {
    // 1. The file that receives the content.
    let dest = resolve(target, expect)?;
    let dir = dest
        .parent()
        .filter(|d| !d.as_os_str().is_empty())
        .ok_or_else(|| WriteError::Refused(format!("{} has no folder", dest.display())))?;
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| WriteError::Refused(format!("{} has no file name", dest.display())))?;

    // 2. Permissions of the original.
    let original = match fs::metadata(&dest) {
        Ok(m) => Some(m),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Err(WriteError::Io {
                step: Step::Resolve,
                message: e.to_string(),
            });
        }
    };
    if let Some(m) = &original {
        if expect == Expect::Absent {
            return Err(WriteError::Conflict {
                disk: Fingerprint::of(&dest).ok().flatten(),
            });
        }
        if m.permissions().readonly() {
            return Err(WriteError::ReadOnly);
        }
    }

    // 3. First check.
    guard.check(Checkpoint::BeforeTemp, &dest)?;

    // 4. Hidden temporary file in the same folder.
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(temp_name(&name, std::process::id(), counter));
    let file = fs_ops.create_new(&tmp).map_err(|e| WriteError::Io {
        step: Step::CreateTemp,
        message: e.to_string(),
    })?;
    // From here on, any failure removes the temporary file of this call (SAV-02).
    if let Err(e) = fill_and_place(
        fs_ops,
        file,
        &tmp,
        &dest,
        bytes,
        original.as_ref(),
        expect,
        guard,
    ) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    // 8. Durability of the rename. The content is already in place: a failure here is not
    // reported as a failed save (the caller would think the file is unchanged).
    let _ = fs_ops.sync_dir(dir);
    // 9. The new base for the caller.
    Ok(Fingerprint::of_bytes(bytes))
}

/// Steps 5 to 7: content, permissions, flush, second check, replace (or create).
#[allow(clippy::too_many_arguments)]
fn fill_and_place(
    fs_ops: &dyn Fs,
    mut file: File,
    tmp: &Path,
    dest: &Path,
    bytes: &[u8],
    original: Option<&fs::Metadata>,
    expect: Expect,
    guard: &dyn Guard,
) -> Result<(), WriteError> {
    if original.is_some()
        && !fs_ops.same_owner(tmp, dest).map_err(|e| WriteError::Io {
            step: Step::Resolve,
            message: e.to_string(),
        })?
    {
        return Err(WriteError::ForeignOwner);
    }
    // 5. Content, permissions of the original, flush to disk.
    fs_ops
        .write_all(&mut file, bytes)
        .map_err(|e| WriteError::Io {
            step: Step::WriteTemp,
            message: e.to_string(),
        })?;
    if let Some(m) = original {
        fs_ops
            .set_permissions(tmp, m.permissions())
            .map_err(|e| WriteError::Io {
                step: Step::Permissions,
                message: e.to_string(),
            })?;
    }
    fs_ops.sync(&file).map_err(|e| WriteError::Io {
        step: Step::SyncTemp,
        message: e.to_string(),
    })?;
    // Closed before the rename: Windows refuses to move a file some handles keep open.
    drop(file);
    // 6. Second check, right before the rename.
    guard.check(Checkpoint::BeforeRename, dest)?;
    // 7. Replace, retrying while another program holds the file (Windows, SAV-08); or create
    // without ever replacing (conflict copy, re-creation).
    match expect {
        Expect::Existing => rename_with_retry(fs_ops, tmp, dest),
        Expect::Absent => create_new_name(fs_ops, tmp, dest),
    }
}

/// Gives `tmp` the name `dest` only if `dest` does not exist. A hard link fails atomically when
/// the name is taken; where links are not supported, the absence is checked once more right
/// before a rename (the residual window of A6).
fn create_new_name(fs_ops: &dyn Fs, tmp: &Path, dest: &Path) -> Result<(), WriteError> {
    match fs_ops.hard_link(tmp, dest) {
        Ok(()) => {
            // The content now has its final name; the temporary name goes away.
            let _ = fs::remove_file(tmp);
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(WriteError::Conflict {
            disk: Fingerprint::of(dest).ok().flatten(),
        }),
        Err(_) => {
            if fs::symlink_metadata(dest).is_ok() {
                return Err(WriteError::Conflict {
                    disk: Fingerprint::of(dest).ok().flatten(),
                });
            }
            rename_with_retry(fs_ops, tmp, dest)
        }
    }
}

/// The conflict check of both checkpoints (CNF-03, TD-13): the disk must still hold exactly the
/// content the caller loaded, compared by size and hash of the raw bytes, never by `mtime`.
pub struct Unchanged(pub Fingerprint);

impl Guard for Unchanged {
    fn check(&self, _at: Checkpoint, target: &Path) -> Result<(), WriteError> {
        match Fingerprint::of(target) {
            Ok(Some(disk)) if disk == self.0 => Ok(()),
            Ok(disk) => Err(WriteError::Conflict { disk }),
            Err(e) => Err(WriteError::Io {
                step: Step::Resolve,
                message: e.to_string(),
            }),
        }
    }
}

/// Saves over a file the caller loaded with fingerprint `base` (CNF-02, CNF-03).
pub fn save_over(
    target: &Path,
    bytes: &[u8],
    base: &Fingerprint,
) -> Result<Fingerprint, WriteError> {
    save(target, bytes, Expect::Existing, &Unchanged(*base))
}

/// Creates a file that must not exist (conflict copy, confirmed re-creation); never replaces.
pub fn save_new(target: &Path, bytes: &[u8]) -> Result<Fingerprint, WriteError> {
    save(target, bytes, Expect::Absent, &NoGuard)
}

/// `<stem>.conflito-<stamp>.md`, then `-2`, `-3`… (CNF-08). `stamp` is the local time the UI
/// gives, `AAAAMMDD-HHMMSS`.
pub fn conflict_copy_name(name: &str, stamp: &str, n: u32) -> Result<String, WriteError> {
    let valid = stamp.len() == 15
        && stamp.as_bytes()[8] == b'-'
        && stamp
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 8 || b.is_ascii_digit());
    if !valid {
        return Err(WriteError::Refused(format!("bad time stamp {stamp:?}")));
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    };
    Ok(if n <= 1 {
        format!("{stem}.conflito-{stamp}{ext}")
    } else {
        format!("{stem}.conflito-{stamp}-{n}{ext}")
    })
}

/// Writes `bytes` as a new conflict copy of `name` in `dir`, with the first free name; never
/// over an existing file. Returns the name used and the fingerprint written.
pub fn save_conflict_copy(
    dir: &Path,
    name: &str,
    bytes: &[u8],
    stamp: &str,
) -> Result<(String, Fingerprint), WriteError> {
    for n in 1..=99 {
        let candidate = conflict_copy_name(name, stamp, n)?;
        match save_new(&dir.join(&candidate), bytes) {
            Ok(fp) => return Ok((candidate, fp)),
            Err(WriteError::Conflict { .. }) => continue,
            Err(e) => return Err(e),
        }
    }
    Err(WriteError::Refused(format!(
        "no free name for a conflict copy of {name}"
    )))
}

/// Total wait for a file held by another program before giving up (SAV-08).
const IN_USE_WAIT: Duration = Duration::from_secs(1);

fn in_use(e: &std::io::Error) -> bool {
    // ERROR_ACCESS_DENIED (5) and ERROR_SHARING_VIOLATION (32) while another handle has the
    // file open without FILE_SHARE_DELETE; ERROR_LOCK_VIOLATION (33).
    cfg!(windows) && matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

fn rename_with_retry(fs_ops: &dyn Fs, tmp: &Path, dest: &Path) -> Result<(), WriteError> {
    let start = std::time::Instant::now();
    let mut pause = Duration::from_millis(10);
    loop {
        match fs_ops.rename(tmp, dest) {
            Ok(()) => return Ok(()),
            Err(e) if in_use(&e) => {
                if start.elapsed() >= IN_USE_WAIT {
                    return Err(WriteError::InUse);
                }
                std::thread::sleep(pause);
                pause = (pause * 2).min(Duration::from_millis(200));
            }
            Err(e) => {
                return Err(WriteError::Io {
                    step: Step::Rename,
                    message: e.to_string(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic pseudo-random bytes for the property tests (no extra dependency).
    struct Rng(u64);
    impl Rng {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }

    #[test]
    fn decode_then_encode_gives_back_every_byte() {
        // The 8 combinations of BOM, line ending and final newline (SAV-03).
        for bom in [false, true] {
            for crlf in [false, true] {
                for final_nl in [false, true] {
                    let eol = if crlf { "\r\n" } else { "\n" };
                    let mut text = format!("# Título{eol}{eol}ação çãõ — linha{eol}- [ ] T1: x");
                    if final_nl {
                        text.push_str(eol);
                    }
                    let mut bytes = if bom { BOM.to_vec() } else { vec![] };
                    bytes.extend_from_slice(text.as_bytes());
                    let (shown, fmt) = decode(&bytes).unwrap();
                    assert!(!shown.contains('\r'));
                    assert_eq!(fmt.bom, bom);
                    assert_eq!(fmt.eol, if crlf { Eol::Crlf } else { Eol::Lf });
                    assert_eq!(fmt.final_newline, final_nl);
                    assert!(!fmt.mixed);
                    assert_eq!(
                        encode(&shown, &fmt),
                        bytes,
                        "bom={bom} crlf={crlf} nl={final_nl}"
                    );
                }
            }
        }
    }

    #[test]
    fn mixed_line_endings_are_flagged_and_normalized_to_the_most_used() {
        let (text, fmt) = decode(b"a\r\nb\r\nc\nd").unwrap();
        assert!(fmt.mixed);
        assert_eq!(fmt.eol, Eol::Crlf);
        assert_eq!(encode(&text, &fmt), b"a\r\nb\r\nc\r\nd");
        // Tie: the first line ending of the file wins.
        assert_eq!(decode(b"a\nb\r\n").unwrap().1.eol, Eol::Lf);
        assert_eq!(decode(b"a\r\nb\n").unwrap().1.eol, Eol::Crlf);
        // A lone CR also counts as mixed (the editor would turn it into a line break).
        let (text, fmt) = decode(b"a\rb\n").unwrap();
        assert!(fmt.mixed);
        assert_eq!(text, "a\nb\n");
        assert!(!decode(b"").unwrap().1.mixed);
    }

    #[test]
    fn non_text_and_large_files_are_not_editable() {
        assert_eq!(decode(b"\xff\xfe a"), Err(NotEditable::NotUtf8));
        assert_eq!(decode(b"abc\0def"), Err(NotEditable::Binary));
        assert_eq!(
            decode(&vec![b'a'; EDIT_LIMIT_BYTES + 1]),
            Err(NotEditable::TooLarge)
        );
        assert!(decode(&vec![b'a'; EDIT_LIMIT_BYTES]).is_ok());
    }

    #[test]
    fn property_decode_encode_round_trips_random_texts() {
        // Random sequences of text, accents, CR, LF, CRLF and an optional BOM: without mixed
        // line endings the bytes come back identical; with them, only line endings change.
        let pieces: [&[u8]; 9] = [
            b"abc",
            "ação ".as_bytes(),
            "ç€𝄞".as_bytes(),
            b" ",
            b"- [x] T1",
            b"\n",
            b"\r\n",
            b"\r",
            b"#",
        ];
        let mut rng = Rng(0x0123_4567_89ab_cdef);
        for case in 0..3_000 {
            let mut bytes = if rng.next().is_multiple_of(4) {
                BOM.to_vec()
            } else {
                vec![]
            };
            let n = rng.next() % 40;
            for _ in 0..n {
                bytes.extend_from_slice(pieces[(rng.next() % pieces.len() as u64) as usize]);
            }
            let (text, fmt) = decode(&bytes).unwrap();
            let back = encode(&text, &fmt);
            if fmt.mixed {
                let norm = |b: &[u8]| {
                    String::from_utf8_lossy(b)
                        .replace("\r\n", "\n")
                        .replace('\r', "\n")
                };
                assert_eq!(norm(&back), norm(&bytes), "case {case}");
                assert_eq!(back.starts_with(BOM), bytes.starts_with(BOM), "case {case}");
            } else {
                assert_eq!(
                    back,
                    bytes,
                    "case {case}: {:?}",
                    String::from_utf8_lossy(&bytes)
                );
            }
        }
    }

    #[test]
    fn temp_names_are_recognized_and_nothing_else() {
        let name = temp_name("spec.md", 1234, 7);
        assert_eq!(name, ".spec.md.quire-1234-7.tmp");
        assert!(is_quire_temp(&name));
        for other in [
            "spec.md",
            ".spec.md.tmp",
            ".spec.md.quire-12.tmp",
            ".spec.md.quire-a-1.tmp",
            ".quire-1-2.tmp",
            "spec.md.quire-1-2.tmp",
            ".spec.md.quire-1-2-3.tmp",
        ] {
            assert!(!is_quire_temp(other), "{other}");
        }
    }

    #[test]
    fn only_listed_markdown_files_of_the_folder_are_writable() {
        let dir = Path::new("spec-dir");
        let listed: Vec<String> = [
            "spec.md",
            "tasks.md",
            "gates.jsonl",
            "notes.txt",
            "Notes.MD",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
        assert_eq!(
            editable_path(dir, "spec.md", &listed),
            Ok(dir.join("spec.md"))
        );
        assert!(editable_path(dir, "Notes.MD", &listed).is_ok());
        for bad in [
            "gates.jsonl",
            "lessons.jsonl",
            "notes.txt",
            "../x.md",
            "a/b.md",
            "a\\b.md",
            "..",
            ".",
            "",
            "missing.md",
            ".spec.md.quire-1-2.tmp",
        ] {
            assert!(
                matches!(
                    editable_path(dir, bad, &listed),
                    Err(WriteError::Refused(_))
                ),
                "{bad}"
            );
        }
    }

    // ---------------------------------------------------------------- pipeline on disk

    fn tmpdir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn leftovers(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect()
    }

    #[test]
    fn save_replaces_the_content_and_leaves_no_temp() {
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"old").unwrap();
        let fp = save(&f, b"new content", Expect::Existing, &NoGuard).unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"new content");
        assert_eq!(fp, Fingerprint::of(&f).unwrap().unwrap());
        assert!(leftovers(d.path()).is_empty());
    }

    #[test]
    fn a_large_file_is_saved_whole() {
        let d = tmpdir();
        let f = d.path().join("big.md");
        let big: Vec<u8> = (0..EDIT_LIMIT_BYTES)
            .map(|i| b"abcdefgh\n"[i % 9])
            .collect();
        fs::write(&f, b"x").unwrap();
        save(&f, &big, Expect::Existing, &NoGuard).unwrap();
        assert_eq!(fs::read(&f).unwrap(), big);
    }

    #[test]
    fn create_only_when_absent_and_never_over_an_existing_file() {
        let d = tmpdir();
        let f = d.path().join("copy.md");
        save(&f, b"first", Expect::Absent, &NoGuard).unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"first");
        assert!(matches!(
            save(&f, b"second", Expect::Absent, &NoGuard),
            Err(WriteError::Conflict { disk: Some(_) })
        ));
        assert_eq!(fs::read(&f).unwrap(), b"first");
        // Replacing a file that vanished is a conflict, not a silent re-creation (CNF-07).
        let gone = d.path().join("gone.md");
        assert_eq!(
            save(&gone, b"x", Expect::Existing, &NoGuard),
            Err(WriteError::Conflict { disk: None })
        );
        assert!(!gone.exists());
        assert!(leftovers(d.path()).is_empty());
    }

    #[test]
    fn read_only_files_are_refused_and_untouched() {
        let d = tmpdir();
        let f = d.path().join("ro.md");
        fs::write(&f, b"keep").unwrap();
        let mut p = fs::metadata(&f).unwrap().permissions();
        p.set_readonly(true);
        fs::set_permissions(&f, p).unwrap();
        assert_eq!(
            save(&f, b"new", Expect::Existing, &NoGuard),
            Err(WriteError::ReadOnly)
        );
        assert_eq!(fs::read(&f).unwrap(), b"keep");
        assert!(leftovers(d.path()).is_empty());
        let mut p = fs::metadata(&f).unwrap().permissions();
        #[allow(clippy::permissions_set_readonly_false)]
        p.set_readonly(false);
        fs::set_permissions(&f, p).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_mode_is_kept() {
        use std::os::unix::fs::PermissionsExt;
        let d = tmpdir();
        for mode in [0o600, 0o644, 0o755, 0o640] {
            let f = d.path().join(format!("m{mode:o}.md"));
            fs::write(&f, b"x").unwrap();
            fs::set_permissions(&f, fs::Permissions::from_mode(mode)).unwrap();
            save(&f, b"y", Expect::Existing, &NoGuard).unwrap();
            assert_eq!(
                fs::metadata(&f).unwrap().permissions().mode() & 0o777,
                mode,
                "{mode:o}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symbolic_link_stays_a_link_and_its_target_gets_the_content() {
        let d = tmpdir();
        let real_dir = d.path().join("real");
        fs::create_dir(&real_dir).unwrap();
        let real = real_dir.join("spec.md");
        fs::write(&real, b"old").unwrap();
        let link = d.path().join("spec.md");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        save(&link, b"new", Expect::Existing, &NoGuard).unwrap();
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read(&real).unwrap(), b"new");
        // The temporary file was made next to the target, and is gone.
        assert!(leftovers(&real_dir).is_empty() && leftovers(d.path()).is_empty());

        let broken = d.path().join("broken.md");
        std::os::unix::fs::symlink(d.path().join("nowhere.md"), &broken).unwrap();
        assert_eq!(
            save(&broken, b"x", Expect::Existing, &NoGuard),
            Err(WriteError::BrokenLink)
        );
        assert!(
            fs::symlink_metadata(&broken)
                .unwrap()
                .file_type()
                .is_symlink()
        );
    }

    #[test]
    fn checkpoints_run_in_order_and_can_stop_the_write() {
        struct Stop(Checkpoint, std::cell::RefCell<Vec<Checkpoint>>);
        impl Guard for Stop {
            fn check(&self, at: Checkpoint, _: &Path) -> Result<(), WriteError> {
                self.1.borrow_mut().push(at);
                if at == self.0 {
                    Err(WriteError::Conflict { disk: None })
                } else {
                    Ok(())
                }
            }
        }
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"agent").unwrap();
        for at in [Checkpoint::BeforeTemp, Checkpoint::BeforeRename] {
            let g = Stop(at, Default::default());
            assert!(matches!(
                save(&f, b"mine", Expect::Existing, &g),
                Err(WriteError::Conflict { .. })
            ));
            assert_eq!(fs::read(&f).unwrap(), b"agent");
            assert!(leftovers(d.path()).is_empty(), "{at:?}");
        }
        let all = Stop(Checkpoint::BeforeTemp, Default::default());
        let _ = save(&f, b"mine", Expect::Existing, &all);
        assert_eq!(*all.1.borrow(), vec![Checkpoint::BeforeTemp]);
    }

    /// Fails one step of the pipeline; everything else is the real file system.
    struct Failing(Step);
    impl Fs for Failing {
        fn create_new(&self, path: &Path) -> std::io::Result<File> {
            if self.0 == Step::CreateTemp {
                return Err(std::io::Error::other("injected: no space left"));
            }
            RealFs.create_new(path)
        }
        fn write_all(&self, file: &mut File, bytes: &[u8]) -> std::io::Result<()> {
            if self.0 == Step::WriteTemp {
                // Half of the bytes reach the temporary file, then the disk is full.
                file.write_all(&bytes[..bytes.len() / 2])?;
                return Err(std::io::Error::other("injected: disk full"));
            }
            RealFs.write_all(file, bytes)
        }
        fn set_permissions(&self, path: &Path, perm: fs::Permissions) -> std::io::Result<()> {
            if self.0 == Step::Permissions {
                return Err(std::io::Error::other("injected: permission"));
            }
            RealFs.set_permissions(path, perm)
        }
        fn sync(&self, file: &File) -> std::io::Result<()> {
            if self.0 == Step::SyncTemp {
                return Err(std::io::Error::other("injected: I/O error on fsync"));
            }
            RealFs.sync(file)
        }
        fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            if self.0 == Step::Rename {
                return Err(std::io::Error::other("injected: rename failed"));
            }
            RealFs.rename(from, to)
        }
        fn hard_link(&self, from: &Path, to: &Path) -> std::io::Result<()> {
            if self.0 == Step::Rename {
                return Err(std::io::Error::other("injected: link failed"));
            }
            RealFs.hard_link(from, to)
        }
        fn sync_dir(&self, dir: &Path) -> std::io::Result<()> {
            RealFs.sync_dir(dir)
        }
        fn same_owner(&self, ours: &Path, theirs: &Path) -> std::io::Result<bool> {
            if self.0 == Step::Resolve {
                return Ok(false);
            }
            RealFs.same_owner(ours, theirs)
        }
    }

    #[test]
    fn a_failure_at_any_step_keeps_the_original_and_removes_the_temp() {
        let d = tmpdir();
        let f = d.path().join("spec.md");
        let original = "# original\r\nação\r\n".as_bytes();
        for step in [
            Step::CreateTemp,
            Step::WriteTemp,
            Step::Permissions,
            Step::SyncTemp,
            Step::Rename,
            Step::Resolve,
        ] {
            fs::write(&f, original).unwrap();
            let r = save_with(
                &Failing(step),
                &f,
                b"replacement that must never land",
                Expect::Existing,
                &NoGuard,
            );
            match step {
                Step::Resolve => assert_eq!(r, Err(WriteError::ForeignOwner)),
                _ => assert!(
                    matches!(&r, Err(WriteError::Io { step: s, .. }) if *s == step),
                    "{step:?}: {r:?}"
                ),
            }
            assert_eq!(fs::read(&f).unwrap(), original, "{step:?}");
            assert!(leftovers(d.path()).is_empty(), "{step:?}");
        }
    }

    #[test]
    fn orphan_temps_are_listed_and_removed_only_by_name() {
        let d = tmpdir();
        fs::write(d.path().join("spec.md"), b"x").unwrap();
        fs::write(d.path().join(temp_name("spec.md", 99, 1)), b"partial").unwrap();
        fs::write(d.path().join(".other.tmp"), b"not ours").unwrap();
        assert_eq!(
            orphan_temps(d.path()).unwrap(),
            vec![temp_name("spec.md", 99, 1)]
        );
        assert!(matches!(
            remove_orphan_temp(d.path(), "spec.md"),
            Err(WriteError::Refused(_))
        ));
        assert!(matches!(
            remove_orphan_temp(d.path(), "../.spec.md.quire-1-1.tmp"),
            Err(WriteError::Refused(_))
        ));
        remove_orphan_temp(d.path(), &temp_name("spec.md", 99, 1)).unwrap();
        assert!(orphan_temps(d.path()).unwrap().is_empty());
        assert!(d.path().join("spec.md").exists() && d.path().join(".other.tmp").exists());
    }

    // ---------------------------------------------------------------- conflicts (T13)

    /// Writes as an agent would at one checkpoint, then lets `Unchanged` check.
    struct AgentAt(Checkpoint, &'static [u8], Unchanged);
    impl Guard for AgentAt {
        fn check(&self, at: Checkpoint, target: &Path) -> Result<(), WriteError> {
            if at == self.0 {
                fs::write(target, self.1).unwrap();
            }
            self.2.check(at, target)
        }
    }

    #[test]
    fn an_outside_write_before_the_save_is_a_conflict_and_survives() {
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"loaded").unwrap();
        let base = Fingerprint::of(&f).unwrap().unwrap();
        fs::write(&f, b"agent wrote this").unwrap();
        let r = save_over(&f, b"mine", &base);
        assert!(
            matches!(&r, Err(WriteError::Conflict { disk: Some(d) }) if *d == Fingerprint::of_bytes(b"agent wrote this")),
            "{r:?}"
        );
        assert_eq!(fs::read(&f).unwrap(), b"agent wrote this");
        assert!(leftovers(d.path()).is_empty());
    }

    #[test]
    fn an_outside_write_between_the_temp_and_the_rename_is_caught() {
        for at in [Checkpoint::BeforeTemp, Checkpoint::BeforeRename] {
            let d = tmpdir();
            let f = d.path().join("spec.md");
            fs::write(&f, b"loaded").unwrap();
            let base = Fingerprint::of(&f).unwrap().unwrap();
            let r = save(
                &f,
                b"mine",
                Expect::Existing,
                &AgentAt(at, b"agent", Unchanged(base)),
            );
            assert!(
                matches!(r, Err(WriteError::Conflict { .. })),
                "{at:?}: {r:?}"
            );
            assert_eq!(fs::read(&f).unwrap(), b"agent", "{at:?}");
            assert!(leftovers(d.path()).is_empty(), "{at:?}");
        }
    }

    #[test]
    fn same_mtime_with_other_content_is_a_conflict_and_the_reverse_is_not() {
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"aaaa").unwrap();
        let base = Fingerprint::of(&f).unwrap().unwrap();
        let mtime = fs::metadata(&f).unwrap().modified().unwrap();
        // Same size, same mtime, other bytes (a tool that preserves mtime).
        fs::write(&f, b"bbbb").unwrap();
        File::options()
            .write(true)
            .open(&f)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
        assert_eq!(fs::metadata(&f).unwrap().modified().unwrap(), mtime);
        assert!(matches!(
            save_over(&f, b"mine", &base),
            Err(WriteError::Conflict { .. })
        ));
        assert_eq!(fs::read(&f).unwrap(), b"bbbb");
        // Same bytes, new mtime (touch): not a conflict.
        fs::write(&f, b"aaaa").unwrap();
        let later = mtime + Duration::from_secs(3600);
        File::options()
            .write(true)
            .open(&f)
            .unwrap()
            .set_modified(later)
            .unwrap();
        let new = save_over(&f, b"mine", &base).unwrap();
        assert_eq!(fs::read(&f).unwrap(), b"mine");
        assert_eq!(new, Fingerprint::of_bytes(b"mine"));
    }

    #[test]
    fn the_returned_fingerprint_is_the_next_base() {
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"v1").unwrap();
        let b1 = Fingerprint::of(&f).unwrap().unwrap();
        let b2 = save_over(&f, b"v2", &b1).unwrap();
        let b3 = save_over(&f, b"v3", &b2).unwrap();
        assert_eq!(Fingerprint::of(&f).unwrap(), Some(b3));
        // The old base no longer matches.
        assert!(matches!(
            save_over(&f, b"v4", &b1),
            Err(WriteError::Conflict { .. })
        ));
    }

    #[test]
    fn conflict_copies_take_a_free_name_and_never_overwrite() {
        let d = tmpdir();
        fs::write(d.path().join("spec.md"), b"disk").unwrap();
        let stamp = "20261116-143005";
        let (n1, _) = save_conflict_copy(d.path(), "spec.md", b"buffer 1", stamp).unwrap();
        assert_eq!(n1, "spec.conflito-20261116-143005.md");
        let (n2, _) = save_conflict_copy(d.path(), "spec.md", b"buffer 2", stamp).unwrap();
        assert_eq!(n2, "spec.conflito-20261116-143005-2.md");
        // A file someone else created with the next name is kept.
        fs::write(
            d.path().join("spec.conflito-20261116-143005-3.md"),
            b"theirs",
        )
        .unwrap();
        let (n4, _) = save_conflict_copy(d.path(), "spec.md", b"buffer 3", stamp).unwrap();
        assert_eq!(n4, "spec.conflito-20261116-143005-4.md");
        assert_eq!(fs::read(d.path().join(&n1)).unwrap(), b"buffer 1");
        assert_eq!(
            fs::read(d.path().join("spec.conflito-20261116-143005-3.md")).unwrap(),
            b"theirs"
        );
        assert_eq!(fs::read(d.path().join("spec.md")).unwrap(), b"disk");
        assert!(leftovers(d.path()).is_empty());
        for bad in [
            "",
            "2026-11-16",
            "20261116_143005",
            "2026111a-143005",
            "../x",
        ] {
            assert!(
                matches!(
                    conflict_copy_name("spec.md", bad, 1),
                    Err(WriteError::Refused(_))
                ),
                "{bad}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn a_file_held_without_delete_sharing_is_reported_in_use_after_a_second() {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x1;
        let d = tmpdir();
        let f = d.path().join("spec.md");
        fs::write(&f, b"agent").unwrap();
        let held = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&f)
            .unwrap();
        let t = std::time::Instant::now();
        let r = save(&f, b"mine", Expect::Existing, &NoGuard);
        let waited = t.elapsed();
        drop(held);
        assert_eq!(r, Err(WriteError::InUse), "{r:?}");
        assert!(waited >= IN_USE_WAIT, "{waited:?}");
        assert!(waited < IN_USE_WAIT * 3, "{waited:?}");
        assert_eq!(fs::read(&f).unwrap(), b"agent");
        assert!(leftovers(d.path()).is_empty());
    }
}
