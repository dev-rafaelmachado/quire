//! Core library for Quire: scanning, parsing and indexing spec folders.
//!
//! This crate must not depend on Tauri, so it can be tested on its own.

pub mod error;
pub mod git;
pub mod scan;

pub use error::{GitError, ScanError};
