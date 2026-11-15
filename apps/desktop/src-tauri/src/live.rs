//! Keeps the index and the UI in line with the disk: scans requested by the UI or by the
//! file watcher run one at a time, and the watcher follows the repositories in the index.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use quire_core::index::Index;
use quire_core::watch::{self, SpecWatcher, WatchEvent, WatchTarget};
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::commands::run_scan;
use crate::events::{ScanPathError, SpecChanged, WatchStatus};
use crate::state::AppState;

const DEBOUNCE: Duration = Duration::from_millis(150);

#[derive(Default)]
struct Queue {
    running: bool,
    rerun: bool,
    /// Spec folders reported changed since the last scan started.
    dirs: BTreeSet<PathBuf>,
}

#[derive(Default)]
pub struct Live {
    queue: Mutex<Queue>,
    watcher: Mutex<Option<(Vec<WatchTarget>, SpecWatcher)>>,
    pub status: Mutex<WatchStatus>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Marks the queue idle when the scan thread ends, even by panic.
struct Running<'a, R: Runtime>(&'a AppHandle<R>);

impl<R: Runtime> Drop for Running<'_, R> {
    fn drop(&mut self) {
        let state = self.0.state::<AppState>();
        lock(&state.live.queue).running = false;
        state.scanning.store(false, Ordering::SeqCst);
    }
}

/// Requests a reconciliation. While one runs, requests are merged into a single rerun.
pub fn request_scan<R: Runtime>(app: &AppHandle<R>, changed: Vec<PathBuf>) -> std::io::Result<()> {
    let state = app.state::<AppState>();
    {
        let mut q = lock(&state.live.queue);
        q.dirs.extend(changed);
        if q.running {
            q.rerun = true;
            return Ok(());
        }
        q.running = true;
        state.scanning.store(true, Ordering::SeqCst);
    }
    let app = app.clone();
    let spawned = std::thread::Builder::new()
        .name("quire-scan".into())
        .spawn(move || {
            let _running = Running(&app);
            loop {
                let state = app.state::<AppState>();
                let dirs = {
                    let mut q = lock(&state.live.queue);
                    q.rerun = false;
                    std::mem::take(&mut q.dirs)
                };
                run_scan(&app);
                if !dirs.is_empty() {
                    let spec_dirs = dirs.iter().map(|d| d.display().to_string()).collect();
                    let _ = SpecChanged { spec_dirs }.emit(&app);
                }
                refresh_watcher(&app);
                crate::cli_delivery::after_scan(&app);
                let rerun = lock(&state.live.queue).rerun;
                if !rerun {
                    break;
                }
            }
        });
    if let Err(e) = spawned {
        lock(&state.live.queue).running = false;
        state.scanning.store(false, Ordering::SeqCst);
        return Err(e);
    }
    Ok(())
}

/// Watches the repositories currently in the index; restarts only when the targets change.
fn refresh_watcher<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let config = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let roots: Vec<PathBuf> = match Index::open(&state.index_path).and_then(|i| i.list()) {
        Ok(list) => list
            .iter()
            .flat_map(|p| {
                p.specs
                    .iter()
                    .flat_map(|s| s.instances.iter().map(|i| i.repo_root.clone()))
            })
            .collect(),
        Err(_) => return,
    };
    let targets = watch::plan(&roots, &config);
    let mut slot = lock(&state.live.watcher);
    if slot.as_ref().is_some_and(|(t, _)| *t == targets) {
        return;
    }
    *slot = None; // stop the old watcher before starting the new one
    let handle = app.clone();
    let interval = Duration::from_millis(config.poll_interval_ms);
    let started = SpecWatcher::start(&targets, interval, DEBOUNCE, move |events| {
        let dirs = events
            .into_iter()
            .filter_map(|e| match e {
                WatchEvent::SpecChanged(d) => Some(d),
                WatchEvent::ListChanged(_) => None,
            })
            .collect();
        let _ = request_scan(&handle, dirs);
    });
    let status = match &started {
        Ok(w) => WatchStatus {
            repos: u32::try_from(w.status.repos).unwrap_or(u32::MAX),
            native: u32::try_from(w.status.native).unwrap_or(u32::MAX),
            polled: u32::try_from(w.status.polled).unwrap_or(u32::MAX),
            poll_interval_ms: u32::try_from(config.poll_interval_ms).unwrap_or(u32::MAX),
            fallbacks: w
                .status
                .fallbacks
                .iter()
                .map(|(p, m)| ScanPathError {
                    path: p.display().to_string(),
                    message: m.clone(),
                })
                .collect(),
        },
        Err(e) => WatchStatus {
            fallbacks: vec![ScanPathError {
                path: String::new(),
                message: e.to_string(),
            }],
            ..WatchStatus::default()
        },
    };
    if let Ok(w) = started {
        *slot = Some((targets, w));
    }
    drop(slot);
    *lock(&state.live.status) = status.clone();
    let _ = status.emit(app);
}
