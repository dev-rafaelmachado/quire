//! File watching for spec folders (TD-05, WAT-01..04).
//!
//! Only `<repo>/.specs` (recursive) and `<repo>` (non-recursive, to see `.specs` appear or the
//! worktree disappear) are watched, never the whole scan path: `node_modules`, `target` and
//! `.git` would exhaust inotify watches (spike T3).
//!
//! Raw events only say *where* to look; what is reported comes from the disk. After the
//! debounce, each repository root touched by the burst is compared with its previous
//! snapshot (the spec folders under `.specs/features` and the length and modification time
//! of their files): folders that appeared or disappeared give a [`WatchEvent::ListChanged`],
//! files that changed give a [`WatchEvent::SpecChanged`], and a burst that changed nothing
//! gives nothing. This makes the result the same on every OS, whatever the event kinds: a
//! save through a temporary file and a rename is one `SpecChanged` both on Linux (inotify
//! `rename`) and on Windows (remove and create in the directory, plus directory "modified"
//! events), and files created together with their folder are seen even when the OS never
//! reports them (spike T3: 0/20 on inotify).
//!
//! Polling is used where the OS delivers no events: network and WSL mounts on Linux
//! (`/proc/self/mountinfo`), UNC and `\\wsl$` paths on Windows, a `[watch]` override in
//! `config.toml`, or a native watch that fails to start.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use notify::{
    Config as NotifyConfig, Event, EventKind, PollWatcher, RecommendedWatcher, RecursiveMode,
    Watcher,
};

use crate::config::{Config, WatchMode};
use crate::error::WatchError;

/// What changed, after grouping raw events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchEvent {
    /// The files of `<repo>/.specs/features/<folder>` changed (content, time, added, removed),
    /// or the folder itself appeared or disappeared.
    SpecChanged(PathBuf),
    /// Spec folders may have appeared or disappeared under this repository root.
    ListChanged(PathBuf),
}

/// One path to watch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WatchTarget {
    pub path: PathBuf,
    pub recursive: bool,
    pub poll: bool,
    /// Why polling was chosen, for the status bar.
    pub reason: Option<String>,
}

/// What the watcher ended up doing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WatchStatus {
    pub repos: usize,
    pub native: usize,
    pub polled: usize,
    /// Native watches that failed and fell back to polling: (path, error).
    pub fallbacks: Vec<(PathBuf, String)>,
}

/// Filesystem types that do not deliver change events to a local watcher.
pub fn fs_needs_polling(fstype: &str) -> bool {
    matches!(
        fstype,
        "9p" | "v9fs" | "drvfs" | "cifs" | "smb3" | "smbfs" | "nfs" | "nfs4" | "afs"
    ) || fstype.starts_with("fuse")
}

/// Windows paths without events: `\\wsl$\…`, `\\wsl.localhost\…` and any UNC share.
pub fn windows_path_needs_polling(path: &str) -> bool {
    let p = path.replace('/', "\\").to_ascii_lowercase();
    if let Some(rest) = p.strip_prefix(r"\\?\") {
        return rest.starts_with(r"unc\");
    }
    p.starts_with(r"\\")
}

/// Filesystem type of the mount holding `path` (Linux), from `/proc/self/mountinfo`.
pub fn mount_fstype(path: &Path) -> Option<String> {
    let info = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    let mut best: Option<(usize, String)> = None;
    for line in info.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let (Some(mount), Some(sep)) = (fields.get(4), fields.iter().position(|f| *f == "-"))
        else {
            continue;
        };
        let mount = mount.replace("\\040", " ");
        let Some(fstype) = fields.get(sep + 1) else {
            continue;
        };
        if path.starts_with(&mount) && best.as_ref().is_none_or(|(len, _)| mount.len() > *len) {
            best = Some((mount.len(), (*fstype).to_owned()));
        }
    }
    best.map(|(_, t)| t)
}

/// Resolves the watch mode of `path`: config override, else detection. Returns the reason
/// when polling.
pub fn decide(path: &Path, config: &Config) -> (bool, Option<String>) {
    match config.watch_mode_for(path) {
        WatchMode::Native => (false, None),
        WatchMode::Poll => (true, Some("config".into())),
        WatchMode::Auto => {
            let text = path.to_string_lossy();
            if cfg!(windows) && windows_path_needs_polling(&text) {
                return (true, Some("unc".into()));
            }
            match mount_fstype(path) {
                Some(t) if fs_needs_polling(&t) => (true, Some(t)),
                _ => (false, None),
            }
        }
    }
}

/// Targets for a set of repository roots (those that hold `.specs`).
pub fn plan(repo_roots: &[PathBuf], config: &Config) -> Vec<WatchTarget> {
    let mut roots: Vec<&PathBuf> = repo_roots.iter().collect();
    roots.sort();
    roots.dedup();
    let mut out = Vec::new();
    for root in roots {
        let (poll, reason) = decide(root, config);
        out.push(WatchTarget {
            path: root.clone(),
            recursive: false,
            poll,
            reason: reason.clone(),
        });
        let specs = root.join(".specs");
        if specs.is_dir() {
            out.push(WatchTarget {
                path: specs,
                recursive: true,
                poll,
                reason,
            });
        }
    }
    out
}

/// Form of a path used to match event paths with watched roots: without the Windows verbatim
/// prefix (`\\?\`, `\\?\UNC\`), with `/` separators, without a trailing separator, and
/// case-insensitive on Windows. Notify reports paths in the form of the watched path on some
/// platforms and in the plain form on others; this makes both match.
pub fn comparable(path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{rest}")
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        rest.to_owned()
    } else {
        text.into_owned()
    };
    let mut out = text.replace('\\', "/");
    while out.len() > 1 && out.ends_with('/') {
        out.pop();
    }
    if cfg!(windows) {
        out = out.to_lowercase();
    }
    out
}

/// Whether `path` is `root` or inside it (both from [`comparable`]).
fn within(path: &str, root: &str) -> bool {
    path == root
        || path
            .strip_prefix(root)
            .is_some_and(|rest| rest.starts_with('/') || root.ends_with('/'))
}

/// Editor and tool leftovers that never make a folder a spec nor a file part of it: hidden
/// files (`.spec.md.tmp`, `.spec.md.swp`) and backups (`spec.md~`).
fn transient(name: &str) -> bool {
    name.starts_with('.')
        || name.ends_with('~')
        || name.ends_with(".tmp")
        || name.ends_with(".swp")
        || name.ends_with(".swx")
}

/// Length and modification time of the files directly in one spec folder.
pub type FolderState = BTreeMap<String, (u64, Option<SystemTime>)>;

/// What a repository root holds under `.specs/features`: spec folders (those with at least
/// one non-transient file) and the state of their files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RootState {
    pub specs: BTreeMap<String, FolderState>,
}

/// Reads the state of a repository root from disk; a missing root or `.specs` is empty.
pub fn snapshot(root: &Path) -> RootState {
    let mut specs = BTreeMap::new();
    let Ok(entries) = std::fs::read_dir(root.join(".specs").join("features")) else {
        return RootState::default();
    };
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let Ok(folder) = entry.file_name().into_string() else {
            continue;
        };
        let mut files = FolderState::new();
        for file in std::fs::read_dir(entry.path())
            .into_iter()
            .flatten()
            .flatten()
        {
            let Ok(name) = file.file_name().into_string() else {
                continue;
            };
            if transient(&name) {
                continue;
            }
            // `fs::metadata` on the path, not the directory entry: on Windows the entry can
            // carry a stale time for a file that was just written.
            let Ok(meta) = std::fs::metadata(file.path()) else {
                continue;
            };
            if meta.is_file() {
                files.insert(name, (meta.len(), meta.modified().ok()));
            }
        }
        if !files.is_empty() {
            specs.insert(folder, files);
        }
    }
    RootState { specs }
}

/// What changed in `root` between two states.
pub fn diff(root: &Path, old: &RootState, new: &RootState) -> Vec<WatchEvent> {
    let dir = |name: &str| root.join(".specs").join("features").join(name);
    let mut out = Vec::new();
    let before: BTreeSet<&String> = old.specs.keys().collect();
    let after: BTreeSet<&String> = new.specs.keys().collect();
    if before != after {
        out.push(WatchEvent::ListChanged(root.to_path_buf()));
        out.extend(
            before
                .symmetric_difference(&after)
                .map(|name| WatchEvent::SpecChanged(dir(name))),
        );
    }
    for (name, files) in &new.specs {
        if old.specs.get(name).is_some_and(|prev| prev != files) {
            out.push(WatchEvent::SpecChanged(dir(name)));
        }
    }
    out.sort();
    out
}

fn relevant(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

/// How many raw events [`SpecWatcher::raw_events`] keeps.
const RAW_LOG: usize = 200;

/// Recent raw events and the batches made of them, oldest first, for diagnostics.
type RawLog = Arc<Mutex<VecDeque<String>>>;

fn log_raw(log: &RawLog, line: String) {
    let mut log = log
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if log.len() == RAW_LOG {
        log.pop_front();
    }
    log.push_back(line);
}

/// Running watchers; dropping it stops watching.
pub struct SpecWatcher {
    _native: Option<RecommendedWatcher>,
    _poll: Option<PollWatcher>,
    pub status: WatchStatus,
    raw: RawLog,
}

impl std::fmt::Debug for SpecWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpecWatcher")
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl SpecWatcher {
    /// The last raw events received (kind and paths) and the batches reported from them,
    /// oldest first. Meant for diagnostics: tests print it when they fail.
    pub fn raw_events(&self) -> Vec<String> {
        self.raw
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .cloned()
            .collect()
    }

    /// Starts watching. `on_events` runs on a background thread with debounced, deduplicated
    /// events, at most `debounce` after the last raw event of a burst (capped at 1 s).
    pub fn start(
        targets: &[WatchTarget],
        poll_interval: Duration,
        debounce: Duration,
        on_events: impl Fn(Vec<WatchEvent>) + Send + 'static,
    ) -> Result<Self, WatchError> {
        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let roots: Vec<PathBuf> = targets
            .iter()
            .filter(|t| !t.recursive)
            .map(|t| t.path.clone())
            .collect();
        let mut status = WatchStatus {
            repos: roots.len(),
            ..WatchStatus::default()
        };
        // The state to compare against is read before watching starts, so a change made
        // right after `start` returns is never part of it.
        let states: Vec<Root> = roots
            .iter()
            .map(|r| Root {
                path: r.clone(),
                key: comparable(r),
                state: snapshot(r),
            })
            .collect();
        let raw: RawLog = Arc::default();

        let mut native = notify::recommended_watcher(tx.clone())
            .map_err(|e| WatchError::Start(e.to_string()))?;
        let mut to_poll: Vec<&WatchTarget> = Vec::new();
        for t in targets {
            if t.poll {
                to_poll.push(t);
                continue;
            }
            let mode = if t.recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            match native.watch(&t.path, mode) {
                Ok(()) => status.native += 1,
                Err(e) => {
                    status.fallbacks.push((t.path.clone(), e.to_string()));
                    to_poll.push(t);
                }
            }
        }
        let poll = if to_poll.is_empty() {
            None
        } else {
            let cfg = NotifyConfig::default().with_poll_interval(poll_interval);
            let mut p =
                PollWatcher::new(tx.clone(), cfg).map_err(|e| WatchError::Start(e.to_string()))?;
            for t in to_poll {
                let mode = if t.recursive {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };
                if p.watch(&t.path, mode).is_ok() {
                    status.polled += 1;
                }
            }
            Some(p)
        };
        drop(tx);

        std::thread::Builder::new()
            .name("quire-watch".into())
            .spawn({
                let raw = raw.clone();
                move || debounce_loop(&rx, states, debounce, &raw, &on_events)
            })
            .map_err(|e| WatchError::Start(e.to_string()))?;
        Ok(Self {
            _native: Some(native),
            _poll: poll,
            status,
            raw,
        })
    }
}

/// A watched repository root and its last known state.
struct Root {
    path: PathBuf,
    key: String,
    state: RootState,
}

/// Roots touched by one raw event: those that contain one of its paths, or are inside one
/// (a parent removed). An error, a rescan request or an event without paths touches all.
fn touched(event: &notify::Result<Event>, roots: &[Root], out: &mut BTreeSet<usize>) {
    let paths = match event {
        Ok(ev) if !relevant(&ev.kind) => return,
        Ok(ev) if !ev.paths.is_empty() && !ev.need_rescan() => &ev.paths,
        _ => {
            out.extend(0..roots.len());
            return;
        }
    };
    for p in paths {
        let p = comparable(p);
        for (i, r) in roots.iter().enumerate() {
            if within(&p, &r.key) || within(&r.key, &p) {
                out.insert(i);
            }
        }
    }
}

fn describe(event: &notify::Result<Event>) -> String {
    match event {
        Ok(ev) => {
            let paths: Vec<String> = ev.paths.iter().map(|p| p.display().to_string()).collect();
            format!("{:?} {}", ev.kind, paths.join(" | "))
        }
        Err(e) => format!("error: {e}"),
    }
}

fn debounce_loop(
    rx: &mpsc::Receiver<notify::Result<Event>>,
    mut roots: Vec<Root>,
    debounce: Duration,
    raw: &RawLog,
    on_events: &impl Fn(Vec<WatchEvent>),
) {
    let max_wait = Duration::from_secs(1);
    loop {
        // Wait for the first event of a burst. Access events (including the reads made by
        // `snapshot` itself) never start one and are not logged.
        let first = loop {
            match rx.recv() {
                Ok(Ok(ev)) if !relevant(&ev.kind) => {}
                Ok(e) => break e,
                Err(_) => return,
            }
        };
        let mut hit: BTreeSet<usize> = BTreeSet::new();
        let mut add = |e: notify::Result<Event>| {
            if e.as_ref().is_ok_and(|ev| !relevant(&ev.kind)) {
                return;
            }
            log_raw(raw, describe(&e));
            touched(&e, &roots, &mut hit);
        };
        add(first);
        let start = Instant::now();
        let mut open = true;
        loop {
            let left = max_wait.saturating_sub(start.elapsed()).min(debounce);
            match rx.recv_timeout(left) {
                Ok(e) => add(e),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    open = false;
                    break;
                }
            }
            if start.elapsed() >= max_wait {
                break;
            }
        }
        // Compare the touched roots with their last state; report only real changes.
        let mut events: BTreeSet<WatchEvent> = BTreeSet::new();
        for i in hit {
            if let Some(root) = roots.get_mut(i) {
                let now = snapshot(&root.path);
                events.extend(diff(&root.path, &root.state, &now));
                root.state = now;
            }
        }
        let events: Vec<WatchEvent> = events.into_iter().collect();
        log_raw(raw, format!("=> {events:?}"));
        if !events.is_empty() {
            on_events(events);
        }
        if !open {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn paths_compare_without_verbatim_prefix_or_separator_style() {
        assert_eq!(comparable(&p(r"\\?\C:\r\a\")), comparable(&p("C:/r/a")));
        assert_eq!(comparable(&p(r"\\?\UNC\srv\share\r")), "//srv/share/r");
        assert_eq!(comparable(&p("/r/a/")), "/r/a");
        assert!(within("/r/a/.specs/x", "/r/a"));
        assert!(within("/r/a", "/r/a"));
        assert!(!within("/r/ab", "/r/a"));
        assert!(within("/r/a", "/"));
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn state_changes_decide_the_events() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("r");
        let features = root.join(".specs/features");
        let x = features.join("x");
        write(&x.join("spec.md"), "# x\n");
        let mut state = snapshot(&root);
        let mut step = |f: &dyn Fn()| {
            f();
            let now = snapshot(&root);
            let events = diff(&root, &state, &now);
            state = now;
            events
        };

        // Nothing changed for real: no events (a file outside, a hidden or backup file).
        assert!(step(&|| write(&root.join("README.md"), "noise\n")).is_empty());
        assert!(step(&|| write(&root.join(".specs/STATE.md"), "s\n")).is_empty());
        assert!(step(&|| write(&x.join(".spec.md.swp"), "swap\n")).is_empty());
        // Atomic save: temporary file renamed over the original is one content change.
        let events = step(&|| {
            write(&x.join(".spec.md.tmp"), "# x\n\nsaved\n");
            std::fs::rename(x.join(".spec.md.tmp"), x.join("spec.md")).unwrap();
        });
        assert_eq!(events, [WatchEvent::SpecChanged(x.clone())]);
        // A new file in a spec is a change of that spec, not of the list.
        let events = step(&|| write(&x.join("tasks.md"), "- [ ] T1: a\n"));
        assert_eq!(events, [WatchEvent::SpecChanged(x.clone())]);
        // A folder with a file is a new spec; an empty folder is not one yet.
        assert!(step(&|| std::fs::create_dir(features.join("empty")).unwrap()).is_empty());
        let y = features.join("y");
        let events = step(&|| write(&y.join("spec.md"), "# y\n"));
        assert_eq!(
            events,
            [
                WatchEvent::SpecChanged(y.clone()),
                WatchEvent::ListChanged(root.clone())
            ]
        );
        // Removing a spec folder, then the whole root (worktree removed).
        let events = step(&|| std::fs::remove_dir_all(&y).unwrap());
        assert_eq!(
            events,
            [
                WatchEvent::SpecChanged(y.clone()),
                WatchEvent::ListChanged(root.clone())
            ]
        );
        let events = step(&|| std::fs::remove_dir_all(&root).unwrap());
        assert_eq!(
            events,
            [
                WatchEvent::SpecChanged(x.clone()),
                WatchEvent::ListChanged(root.clone())
            ]
        );
    }

    #[test]
    fn polling_detection_rules() {
        for t in ["9p", "drvfs", "cifs", "nfs4", "fuse.sshfs"] {
            assert!(fs_needs_polling(t), "{t}");
        }
        for t in ["ext4", "btrfs", "xfs", "tmpfs", "ntfs3"] {
            assert!(!fs_needs_polling(t), "{t}");
        }
        assert!(windows_path_needs_polling(r"\\wsl$\Ubuntu\home\dev"));
        assert!(windows_path_needs_polling(r"\\wsl.localhost\Ubuntu\home"));
        assert!(windows_path_needs_polling(r"\\server\share\repo"));
        assert!(windows_path_needs_polling(r"\\?\UNC\server\share"));
        assert!(!windows_path_needs_polling(r"\\?\C:\repos"));
        assert!(!windows_path_needs_polling(r"C:\repos"));
    }

    #[test]
    fn plan_watches_roots_and_specs_with_config_overrides() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        std::fs::create_dir_all(a.join(".specs")).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let text = format!("[watch.paths]\n'{}' = 'poll'\n", b.display());
        let config = Config::parse(&text).unwrap();
        let targets = plan(&[b.clone(), a.clone(), a.clone()], &config);
        let summary: Vec<(PathBuf, bool, bool)> = targets
            .iter()
            .map(|t| (t.path.clone(), t.recursive, t.poll))
            .collect();
        assert_eq!(
            summary,
            [
                (a.clone(), false, false),
                (a.join(".specs"), true, false),
                (b.clone(), false, true)
            ]
        );
        assert_eq!(targets[2].reason.as_deref(), Some("config"));
    }
}
