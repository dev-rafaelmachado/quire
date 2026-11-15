//! Typed errors of the core library.

use std::path::PathBuf;

/// Why a scan path could not be scanned (DSC-05). Reported per path; other paths go on.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ScanError {
    #[error("path does not exist: {0}")]
    NotFound(PathBuf),
    #[error("path is not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    #[error("cannot read {path}: {message}")]
    Io { path: PathBuf, message: String },
}

impl ScanError {
    pub(crate) fn from_io(path: PathBuf, err: &std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound(path),
            std::io::ErrorKind::PermissionDenied => Self::PermissionDenied(path),
            std::io::ErrorKind::NotADirectory => Self::NotADirectory(path),
            _ => Self::Io {
                path,
                message: err.to_string(),
            },
        }
    }
}

/// Failure of a call to the system `git` binary (TD-03). Callers fall back to folder names.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum GitError {
    #[error("git is not installed or not on PATH")]
    NotInstalled,
    #[error("could not run git: {0}")]
    Spawn(String),
    #[error("git {args} timed out after {secs} s")]
    Timeout { args: String, secs: u64 },
    #[error("git {args} failed ({code:?}): {stderr}")]
    Failed {
        args: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("unexpected output from git {0}")]
    UnexpectedOutput(String),
}
