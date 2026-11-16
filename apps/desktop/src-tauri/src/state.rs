//! Shared state of the shell: configuration and the index cache.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

use quire_core::{Config, Index};

use crate::commands::{CmdError, err};

pub struct AppState {
    pub config: RwLock<Config>,
    pub config_path: Option<PathBuf>,
    /// Set when `config.toml` could not be read; the app runs with defaults and shows it.
    pub config_error: Option<CmdError>,
    pub index_path: PathBuf,
    /// Connection used by queries; reconciliation opens its own (WAL allows both).
    pub index: Mutex<Index>,
    pub scanning: AtomicBool,
    /// Scan queue and file watcher (T21).
    pub live: crate::live::Live,
    /// Command-line navigation waiting for the index (T22).
    pub cli: crate::cli_delivery::CliState,
}

impl AppState {
    /// Loads config and opens the cache. `QUIRE_CONFIG` and `QUIRE_INDEX` override the default
    /// locations (A2), which is how the fixtures and tests run.
    #[cfg_attr(test, allow(dead_code))]
    pub fn load() -> Result<Self, String> {
        Self::from_env(
            std::env::var_os("QUIRE_CONFIG"),
            std::env::var_os("QUIRE_INDEX"),
        )
    }

    /// `load` with the variables given (TD-21): a relative `QUIRE_CONFIG` leaves the config
    /// unreadable and not editable, so nothing is ever written next to the working directory
    /// (which varies: `tauri dev` runs in `src-tauri`); a relative `QUIRE_INDEX`, or one inside
    /// a `.specs` folder the watcher follows, stops the start before any file is created.
    pub fn from_env(
        config_var: Option<std::ffi::OsString>,
        index_var: Option<std::ffi::OsString>,
    ) -> Result<Self, String> {
        let (config_path, refused) = match config_var.map(PathBuf::from) {
            Some(p) if !p.is_absolute() => (
                None,
                Some(err(
                    "config_env_relative",
                    format!("QUIRE_CONFIG must be an absolute path, got {}", p.display()),
                )),
            ),
            Some(p) => (Some(p), None),
            None => (Config::default_path(), None),
        };
        let (config, config_error) = match (refused, config_path.as_deref().map(Config::load)) {
            (Some(e), _) => (Config::default(), Some(e)),
            (None, Some(Ok(c))) => (c, None),
            (None, Some(Err(e))) => (Config::default(), Some(err(e.code(), &e))),
            (None, None) => (Config::default(), None),
        };
        let index_path = match index_var.map(PathBuf::from) {
            Some(p) => check_index_path(p)?,
            None => Index::default_path().ok_or("no data directory for the index cache")?,
        };
        Self::new(config, config_path, config_error, index_path)
    }

    pub fn new(
        config: Config,
        config_path: Option<PathBuf>,
        config_error: Option<CmdError>,
        index_path: PathBuf,
    ) -> Result<Self, String> {
        let index = Index::open(&index_path).map_err(|e| e.to_string())?;
        Ok(Self {
            config: RwLock::new(config),
            config_path,
            config_error,
            index_path,
            index: Mutex::new(index),
            scanning: AtomicBool::new(false),
            live: crate::live::Live::default(),
            cli: crate::cli_delivery::CliState::default(),
        })
    }
}

/// Refuses an index path that is relative or inside a `.specs` folder (TD-21): writing the
/// cache there would make the watcher report the app's own writes.
pub fn check_index_path(path: PathBuf) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "QUIRE_INDEX must be an absolute path, got {}",
            path.display()
        ));
    }
    if path.components().any(|c| c.as_os_str() == ".specs") {
        return Err(format!(
            "QUIRE_INDEX must not be inside a .specs folder: {}",
            path.display()
        ));
    }
    Ok(path)
}
