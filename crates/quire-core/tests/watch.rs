#![allow(clippy::unwrap_used)] // test-only crate

//! The watcher on a real filesystem (native backend), plus the polling fallback.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use quire_core::watch::{SpecWatcher, WatchEvent, WatchTarget};

fn repo(tmp: &Path) -> PathBuf {
    fs::create_dir_all(tmp.join("repo/.specs/features/acme-1-x")).unwrap();
    let root = tmp.join("repo").canonicalize().unwrap();
    fs::write(root.join(".specs/features/acme-1-x/spec.md"), "# x\n").unwrap();
    root
}

fn targets(root: &Path, poll: bool) -> Vec<WatchTarget> {
    vec![
        WatchTarget {
            path: root.to_path_buf(),
            recursive: false,
            poll,
            reason: None,
        },
        WatchTarget {
            path: root.join(".specs"),
            recursive: true,
            poll,
            reason: None,
        },
    ]
}

fn start(root: &Path, poll: bool) -> (SpecWatcher, mpsc::Receiver<Vec<WatchEvent>>) {
    let (tx, rx) = mpsc::channel();
    let w = SpecWatcher::start(
        &targets(root, poll),
        Duration::from_millis(200),
        Duration::from_millis(150),
        move |e| {
            let _ = tx.send(e);
        },
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    (w, rx)
}

/// Collects batches until `pred` matches or `timeout` passes; returns everything seen.
fn collect_until(
    rx: &mpsc::Receiver<Vec<WatchEvent>>,
    timeout: Duration,
    pred: impl Fn(&[WatchEvent]) -> bool,
) -> (Vec<WatchEvent>, Duration) {
    let start = Instant::now();
    let mut seen = Vec::new();
    while start.elapsed() < timeout {
        if let Ok(batch) = rx.recv_timeout(Duration::from_millis(50)) {
            seen.extend(batch);
            if pred(&seen) {
                break;
            }
        }
    }
    (seen, start.elapsed())
}

#[test]
fn editing_a_file_reports_its_spec_within_two_seconds() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path());
    let (_w, rx) = start(&root, false);
    let spec = root.join(".specs/features/acme-1-x");
    fs::write(spec.join("spec.md"), "# x\n\nedited\n").unwrap();
    let (seen, took) = collect_until(&rx, Duration::from_secs(2), |s| {
        s.contains(&WatchEvent::SpecChanged(spec.clone()))
    });
    assert!(seen.contains(&WatchEvent::SpecChanged(spec)), "{seen:?}");
    assert!(took < Duration::from_secs(2), "{took:?}");
}

#[test]
fn a_folder_created_with_its_files_is_a_list_change() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path());
    let (_w, rx) = start(&root, false);
    let new = root.join(".specs/features/acme-2-new");
    fs::create_dir(&new).unwrap();
    fs::write(new.join("spec.md"), "# new\n").unwrap();
    let (seen, _) = collect_until(&rx, Duration::from_secs(2), |s| {
        s.contains(&WatchEvent::ListChanged(root.clone()))
    });
    assert!(
        seen.contains(&WatchEvent::ListChanged(root.clone())),
        "{seen:?}"
    );
}

#[test]
fn atomic_save_is_a_change_not_a_removal() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path());
    let (_w, rx) = start(&root, false);
    let spec = root.join(".specs/features/acme-1-x");
    fs::write(spec.join(".spec.md.tmp"), "# x\n\nsaved\n").unwrap();
    fs::rename(spec.join(".spec.md.tmp"), spec.join("spec.md")).unwrap();
    let (seen, _) = collect_until(&rx, Duration::from_millis(800), |_| false);
    assert!(seen.contains(&WatchEvent::SpecChanged(spec)), "{seen:?}");
    assert!(
        !seen.contains(&WatchEvent::ListChanged(root)),
        "a save must not look like a folder change: {seen:?}"
    );
}

#[test]
fn files_outside_specs_are_ignored_and_removals_are_list_changes() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path());
    let (_w, rx) = start(&root, false);
    fs::write(root.join("README.md"), "noise\n").unwrap();
    let (seen, _) = collect_until(&rx, Duration::from_millis(600), |_| false);
    assert!(seen.is_empty(), "{seen:?}");
    fs::remove_dir_all(root.join(".specs/features/acme-1-x")).unwrap();
    let (seen, _) = collect_until(&rx, Duration::from_secs(2), |s| {
        s.contains(&WatchEvent::ListChanged(root.clone()))
    });
    assert!(seen.contains(&WatchEvent::ListChanged(root)), "{seen:?}");
}

#[test]
fn polling_picks_up_changes_within_the_interval() {
    let tmp = tempfile::tempdir().unwrap();
    let root = repo(tmp.path());
    let (w, rx) = start(&root, true);
    assert_eq!((w.status.native, w.status.polled), (0, 2));
    let spec = root.join(".specs/features/acme-1-x");
    std::thread::sleep(Duration::from_millis(50));
    fs::write(spec.join("tasks.md"), "- [ ] T1: a\n").unwrap();
    let (seen, took) = collect_until(&rx, Duration::from_secs(3), |s| {
        s.contains(&WatchEvent::SpecChanged(spec.clone()))
    });
    assert!(seen.contains(&WatchEvent::SpecChanged(spec)), "{seen:?}");
    assert!(took < Duration::from_secs(2), "{took:?}");
}

#[test]
fn a_target_that_cannot_be_watched_falls_back_to_polling() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing");
    let t = vec![WatchTarget {
        path: missing.clone(),
        recursive: false,
        poll: false,
        reason: None,
    }];
    let w = SpecWatcher::start(
        &t,
        Duration::from_millis(200),
        Duration::from_millis(150),
        |_| {},
    )
    .unwrap();
    assert_eq!(w.status.native, 0);
    assert_eq!(w.status.fallbacks.len(), 1);
    assert_eq!(w.status.fallbacks[0].0, missing);
}
