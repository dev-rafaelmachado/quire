//! Shared helpers for integration tests.
//!
//! Every test builds its own fixture workspace with `scripts/make-fixtures.sh` in a fresh
//! temporary directory (~45 ms). Nothing is shared between tests or test processes, so tests
//! can run in parallel threads or processes and leftovers of earlier runs do not matter. The
//! script is located from `CARGO_MANIFEST_DIR`, so the working directory is irrelevant.

#![allow(dead_code, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn bash() -> PathBuf {
    if let Some(b) = std::env::var_os("QUIRE_BASH") {
        return b.into();
    }
    // On Windows, `bash` on PATH may be the WSL launcher; prefer Git Bash.
    let git_bash = Path::new(r"C:\Program Files\Git\bin\bash.exe");
    if cfg!(windows) && git_bash.exists() {
        return git_bash.to_path_buf();
    }
    PathBuf::from("bash")
}

/// A generated fixture workspace (`<out>` in fixtures/README.md), deleted on drop.
pub struct Fixtures {
    root: PathBuf,
    _dir: tempfile::TempDir,
}

impl Fixtures {
    pub fn new() -> Self {
        let dir = tempfile::Builder::new()
            .prefix("quire-fixtures-")
            .tempdir()
            .expect("cannot create a temporary directory");
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/make-fixtures.sh");
        // The script accepts an empty directory and rebuilds it.
        let status = Command::new(bash())
            .arg(&script)
            .arg(dir.path())
            .status()
            .expect("failed to run scripts/make-fixtures.sh");
        assert!(status.success(), "make-fixtures.sh failed");
        let root = dir
            .path()
            .canonicalize()
            .expect("fixture workspace missing");
        Self { root, _dir: dir }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The scan path of the fixtures.
    pub fn projects(&self) -> PathBuf {
        self.root.join("projects")
    }

    /// Canonical path of something inside the workspace.
    pub fn path(&self, rel: &str) -> PathBuf {
        self.root
            .join(rel)
            .canonicalize()
            .expect("path not in the fixtures")
    }
}
