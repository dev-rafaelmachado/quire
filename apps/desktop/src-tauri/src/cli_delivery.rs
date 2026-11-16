//! Hands command-line navigation to the UI once the index can resolve it.

use std::sync::{Mutex, MutexGuard};

use quire_core::index::Index;
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::cli::{self, CliOpen, CliRequest};
use crate::events::CliNavigate;
use crate::state::AppState;

#[derive(Default)]
pub struct CliState {
    /// Waiting for a scan: the index was empty or did not know the target yet.
    pending: Mutex<Option<CliRequest>>,
    /// Last navigation, kept until the UI takes it (the UI may not be listening yet).
    delivered: Mutex<Option<CliOpen>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Resolves and sends a request now if possible; otherwise keeps it for after the next scan.
pub fn deliver<R: Runtime>(app: &AppHandle<R>, request: CliRequest) {
    let state = app.state::<AppState>();
    let list = Index::open(&state.index_path)
        .and_then(|i| i.list())
        .unwrap_or_default();
    match cli::resolve(&list, &request) {
        None => {}
        Some(_) if list.is_empty() => *lock(&state.cli.pending) = Some(request),
        Some(open @ CliOpen::NotFound { .. }) => {
            // The spec may be new on disk: try again after the next scan, and say so now.
            *lock(&state.cli.pending) = Some(request);
            send(app, open);
        }
        Some(open) => send(app, open),
    }
}

/// Called after each scan: resolves a waiting request against the fresh index.
pub fn after_scan<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let Some(request) = lock(&state.cli.pending).take() else {
        return;
    };
    let list = Index::open(&state.index_path)
        .and_then(|i| i.list())
        .unwrap_or_default();
    if let Some(open) = cli::resolve(&list, &request) {
        send(app, open);
    }
}

fn send<R: Runtime>(app: &AppHandle<R>, open: CliOpen) {
    *lock(&app.state::<AppState>().cli.delivered) = Some(open.clone());
    let _ = CliNavigate { open }.emit(app);
}

/// The navigation not yet taken by the UI, if any.
pub fn take<R: Runtime>(app: &AppHandle<R>) -> Option<CliOpen> {
    lock(&app.state::<AppState>().cli.delivered).take()
}
