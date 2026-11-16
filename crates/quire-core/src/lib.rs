//! Core library for Quire: scanning, parsing and indexing spec folders.
//!
//! This crate must not depend on Tauri, so it can be tested on its own.

pub mod config;
pub mod error;
pub mod fingerprint;
pub mod fs_kind;
pub mod git;
pub mod index;
pub mod model;
pub mod parse;
pub mod preset;
pub mod scan;
pub mod watch;
pub mod write;

pub use config::Config;
pub use error::{ConfigError, GitError, IndexError, ScanError, WatchError};
pub use index::Index;
