#![allow(clippy::unwrap_used)] // test-only crate

//! A real crash between the temporary file and the rename (SAV-01, SAV-10, T26): a child
//! process saves through the pipeline and aborts right before the rename; the parent checks
//! that the original is intact and that the leftover temporary file is found, not listed as a
//! file of the spec, and removed only on request.

use std::path::Path;
use std::process::Command;

use quire_core::write::{self, Checkpoint, Expect, Guard, WriteError};

const ENV: &str = "QUIRE_TEST_CRASH_TARGET";

struct AbortBeforeRename;
impl Guard for AbortBeforeRename {
    fn check(&self, at: Checkpoint, _: &Path) -> Result<(), WriteError> {
        if at == Checkpoint::BeforeRename {
            // The process dies here: no cleanup code of the pipeline runs.
            std::process::abort();
        }
        Ok(())
    }
}

/// Runs only as the child of `a_crash_before_the_rename_keeps_the_original`.
#[test]
#[ignore]
fn crash_child() {
    let Some(target) = std::env::var_os(ENV) else {
        return;
    };
    let _ = write::save(
        Path::new(&target),
        b"new content that never lands",
        Expect::Existing,
        &AbortBeforeRename,
    );
    unreachable!("the guard aborts the process");
}

#[test]
fn a_crash_before_the_rename_keeps_the_original() {
    let dir = tempfile::tempdir().unwrap();
    let spec = dir.path().join("spec.md");
    std::fs::write(&spec, "# Original\r\nação\r\n").unwrap();
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "crash_child",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(ENV, &spec)
        .status()
        .unwrap();
    assert!(!status.success(), "the child must die before the rename");
    // The original is byte for byte what it was.
    assert_eq!(
        std::fs::read(&spec).unwrap(),
        "# Original\r\nação\r\n".as_bytes()
    );
    // The temporary file stays, is recognized, is not a file of the spec, and goes only on request.
    let orphans = write::orphan_temps(dir.path()).unwrap();
    assert_eq!(orphans.len(), 1, "{orphans:?}");
    assert!(
        std::fs::read(dir.path().join(&orphans[0]))
            .unwrap()
            .starts_with(b"new content")
    );
    assert_eq!(
        quire_core::scan::list_files(dir.path()),
        vec!["spec.md".to_owned()]
    );
    write::remove_orphan_temp(dir.path(), &orphans[0]).unwrap();
    assert!(write::orphan_temps(dir.path()).unwrap().is_empty());
    assert_eq!(
        std::fs::read(&spec).unwrap(),
        "# Original\r\nação\r\n".as_bytes()
    );
}
