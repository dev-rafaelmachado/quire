//! File watching for spec folders (TD-05, WAT-01..04).
//!
//! Only `<repo>/.specs` (recursive) and `<repo>` (non-recursive, to see `.specs` appear or the
//! worktree disappear) are watched, never the whole scan path: `node_modules`, `target` and
//! `.git` would exhaust inotify watches (spike T3). Events are grouped per spec and debounced.
//! A created folder is reported as a list change, so the caller rescans it: files created
//! together with their folder never get their own event (spike T3: 0/20 on inotify).
//!
//! Polling is used where the OS delivers no events: network and WSL mounts on Linux
//! (`/proc/self/mountinfo`), UNC and `\\wsl$` paths on Windows, a `[watch]` override in
//! `config.toml`, or a native watch that fails to start.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{
    Config as NotifyConfig, Event, EventKind, PollWatcher, RecommendedWatcher, RecursiveMode,
    Watcher,
};

use crate::config::{Config, WatchMode};
use crate::error::WatchError;

/// What changed, after grouping raw events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchEvent {
    /// A file inside `<repo>/.specs/features/<folder>` changed (created, edited, renamed, removed).
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

/// Maps a raw event path to what it means for Quire, given the watched repository roots.
pub fn classify(path: &Path, roots: &[PathBuf]) -> Vec<WatchEvent> {
    let comps: Vec<Component<'_>> = path.components().collect();
    if let Some(i) = comps.iter().position(|c| c.as_os_str() == ".specs") {
        let root: PathBuf = comps[..i].iter().collect();
        let rest = &comps[i + 1..];
        return match rest {
            [f, folder] if f.as_os_str() == "features" => {
                // The folder itself was created, removed or renamed.
                let dir = root
                    .join(".specs")
                    .join("features")
                    .join(folder.as_os_str());
                vec![WatchEvent::ListChanged(root), WatchEvent::SpecChanged(dir)]
            }
            [f, folder, ..] if f.as_os_str() == "features" => {
                vec![WatchEvent::SpecChanged(
                    root.join(".specs")
                        .join("features")
                        .join(folder.as_os_str()),
                )]
            }
            // `.specs` or `.specs/features` themselves, or files like `.specs/STATE.md`.
            [] | [_] => vec![WatchEvent::ListChanged(root)],
            _ => vec![],
        };
    }
    // Non-recursive watch on a root: only the root itself matters (worktree removed).
    if roots.iter().any(|r| r == path) {
        return vec![WatchEvent::ListChanged(path.to_path_buf())];
    }
    vec![]
}

fn relevant(kind: &EventKind) -> bool {
    !matches!(kind, EventKind::Access(_))
}

/// Running watchers; dropping it stops watching.
pub struct SpecWatcher {
    _native: Option<RecommendedWatcher>,
    _poll: Option<PollWatcher>,
    pub status: WatchStatus,
}

impl std::fmt::Debug for SpecWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpecWatcher")
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

impl SpecWatcher {
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
            .spawn(move || debounce_loop(&rx, &roots, debounce, &on_events))
            .map_err(|e| WatchError::Start(e.to_string()))?;
        Ok(Self {
            _native: Some(native),
            _poll: poll,
            status,
        })
    }
}

fn debounce_loop(
    rx: &mpsc::Receiver<notify::Result<Event>>,
    roots: &[PathBuf],
    debounce: Duration,
    on_events: &impl Fn(Vec<WatchEvent>),
) {
    let max_wait = Duration::from_secs(1);
    loop {
        // Wait for the first event of a burst.
        let first = match rx.recv() {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut pending: BTreeSet<WatchEvent> = BTreeSet::new();
        let mut add = |e: notify::Result<Event>| {
            let Ok(ev) = e else { return };
            if relevant(&ev.kind) {
                for p in &ev.paths {
                    pending.extend(classify(p, roots));
                }
            }
        };
        add(first);
        let start = Instant::now();
        loop {
            let left = max_wait.saturating_sub(start.elapsed()).min(debounce);
            match rx.recv_timeout(left) {
                Ok(e) => add(e),
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => {
                    if !pending.is_empty() {
                        on_events(pending.into_iter().collect());
                    }
                    return;
                }
            }
            if start.elapsed() >= max_wait {
                break;
            }
        }
        if !pending.is_empty() {
            on_events(pending.into_iter().collect());
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
    fn classifies_paths() {
        let roots = [p("/r/a")];
        assert_eq!(
            classify(&p("/r/a/.specs/features/x/spec.md"), &roots),
            [WatchEvent::SpecChanged(p("/r/a/.specs/features/x"))]
        );
        assert_eq!(
            classify(&p("/r/a/.specs/features/new"), &roots),
            [
                WatchEvent::ListChanged(p("/r/a")),
                WatchEvent::SpecChanged(p("/r/a/.specs/features/new"))
            ]
        );
        assert_eq!(
            classify(&p("/r/a/.specs"), &roots),
            [WatchEvent::ListChanged(p("/r/a"))]
        );
        assert_eq!(
            classify(&p("/r/a/.specs/STATE.md"), &roots),
            [WatchEvent::ListChanged(p("/r/a"))]
        );
        assert_eq!(
            classify(&p("/r/a"), &roots),
            [WatchEvent::ListChanged(p("/r/a"))]
        );
        assert!(classify(&p("/r/a/README.md"), &roots).is_empty());
        assert!(classify(&p("/r/a/.specs/other/deep/file"), &roots).is_empty());
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
