#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use std::path::PathBuf;

use quire_core::scan::{ScanEvent, generic_filter, scan};

fn rel(p: &std::path::Path) -> String {
    p.strip_prefix(common::workspace())
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn finds_every_expected_spec_in_the_fixtures() {
    let mut found = vec![];
    let report = scan(&[common::projects()], &generic_filter, |e| {
        if let ScanEvent::Found(d) = e {
            found.push(rel(&d.spec_dir));
        }
    });
    found.sort();
    let expected = [
        "projects/.worktrees/acme-142/acme_gateway/.specs/features/acme-142-service-tokens",
        "projects/.worktrees/acme-142/acme_gateway/.specs/features/acme-170-rate-limit",
        "projects/.worktrees/acme-142/acme_gateway/.specs/features/acme-200-shared-auth",
        "projects/acme_billing/.specs/features/acme-200-shared-auth",
        "projects/acme_billing/.specs/features/bill-31-invoice-export",
        "projects/acme_billing/.specs/features/bill-40-refunds",
        "projects/acme_gateway/.specs/features/acme-142-service-tokens",
        "projects/acme_gateway/.specs/features/acme-170-rate-limit",
        "projects/acme_gateway/.specs/features/acme-200-shared-auth",
    ];
    assert_eq!(found, expected);
    assert_eq!(report.found, expected.len());
    assert_eq!(report.path_errors, 0);
}

#[test]
fn decoys_in_ignored_folders_never_appear() {
    let mut found = vec![];
    scan(&[common::workspace().to_path_buf()], &generic_filter, |e| {
        if let ScanEvent::Found(d) = e {
            found.push(d.folder);
        }
    });
    assert!(!found.is_empty());
    assert!(found.iter().all(|f| !f.starts_with("decoy")), "{found:?}");
}

#[test]
fn invalid_path_does_not_stop_the_scan() {
    let mut errors = 0;
    let mut found = 0;
    scan(
        &[PathBuf::from("/definitely/not/here"), common::projects()],
        &generic_filter,
        |e| match e {
            ScanEvent::Found(_) => found += 1,
            ScanEvent::PathError { .. } => errors += 1,
        },
    );
    assert_eq!(errors, 1);
    assert_eq!(found, 9);
}
