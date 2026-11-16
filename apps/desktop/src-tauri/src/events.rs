//! Typed events sent to the webview (design: `scan://progress`, `specs://changed`).
//! `spec://changed` arrives with the watcher (T21) and `cli://open` with the CLI (T22).

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScanPathError {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ScanPhase {
    Scanning,
    Indexing,
    Done,
    Failed,
}

/// Progress of a scan (DSC-04): spec folders found, folders indexed, and per-path errors (DSC-05).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
#[tauri_specta(event_name = "scan://progress")]
pub struct ScanProgress {
    pub phase: ScanPhase,
    pub found: u32,
    pub processed: u32,
    pub total: u32,
    pub errors: Vec<ScanPathError>,
    /// Set when `phase` is `failed`.
    pub message: Option<String>,
}

/// The list in the index changed; the sidebar should call `list_projects` again.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "specs://changed")]
pub struct SpecsChanged;

/// Files of these spec folders changed on disk (WAT-01); the index is already updated.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
#[tauri_specta(event_name = "spec://changed")]
pub struct SpecChanged {
    pub spec_dirs: Vec<String>,
}

/// What the file watcher is doing (status bar; WAT-03).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
#[tauri_specta(event_name = "watch://status")]
pub struct WatchStatus {
    /// Repository roots watched.
    pub repos: u32,
    pub native: u32,
    pub polled: u32,
    pub poll_interval_ms: u32,
    /// Paths whose native watch failed and fell back to polling.
    pub fallbacks: Vec<ScanPathError>,
}

/// Navigation asked on the command line (CLI-01..04).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
#[tauri_specta(event_name = "cli://open")]
pub struct CliNavigate {
    pub open: crate::cli::CliOpen,
}
