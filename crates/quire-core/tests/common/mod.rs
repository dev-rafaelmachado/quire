//! Shared helpers for integration tests: builds the synthetic fixture workspace once per
//! test binary with `scripts/make-fixtures.sh`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

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

/// Root of the generated workspace (`<out>` in fixtures/README.md).
pub fn workspace() -> &'static Path {
    static WS: OnceLock<PathBuf> = OnceLock::new();
    WS.get_or_init(|| {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("fixtures-{}", env!("CARGO_CRATE_NAME")));
        let status = Command::new(bash())
            .arg(repo.join("scripts/make-fixtures.sh"))
            .arg(&out)
            .status()
            .expect("failed to run scripts/make-fixtures.sh");
        assert!(status.success(), "make-fixtures.sh failed");
        out.canonicalize().expect("fixture workspace missing")
    })
}

/// The scan path of the fixtures.
pub fn projects() -> PathBuf {
    workspace().join("projects")
}
