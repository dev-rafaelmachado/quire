//! Shared state of the shell: configuration and the index cache.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};

use quire_core::{Config, Index};

pub struct AppState {
    pub config: RwLock<Config>,
    pub config_path: Option<PathBuf>,
    /// Set when `config.toml` could not be read; the app runs with defaults and shows it.
    pub config_error: Option<String>,
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
        let config_path = std::env::var_os("QUIRE_CONFIG")
            .map(PathBuf::from)
            .or_else(Config::default_path);
        let (config, config_error) = match config_path.as_deref().map(Config::load) {
            Some(Ok(c)) => (c, None),
            Some(Err(e)) => (Config::default(), Some(e.to_string())),
            None => (Config::default(), None),
        };
        let index_path = std::env::var_os("QUIRE_INDEX")
            .map(PathBuf::from)
            .or_else(Index::default_path)
            .ok_or("no data directory for the index cache")?;
        Self::new(config, config_path, config_error, index_path)
    }

    pub fn new(
        config: Config,
        config_path: Option<PathBuf>,
        config_error: Option<String>,
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
