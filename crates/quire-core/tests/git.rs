#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use quire_core::git::{GitResolver, NameSource};
use quire_core::scan::{ScanEvent, generic_filter, scan_with_worktrees};

#[test]
fn worktrees_of_the_same_repo_resolve_to_the_same_project() {
    let fx = common::Fixtures::new();
    let mut git = GitResolver::new();
    let main = git.project(&fx.path("projects/acme_gateway"));
    let inside = git.project(&fx.path("projects/.worktrees/acme-142/acme_gateway"));
    let outside = git.project(&fx.path("external/acme-170/acme_gateway"));
    assert_eq!(main.key, inside.key);
    assert_eq!(main.key, outside.key);
    assert_eq!(main.name, "acme_gateway");
    assert_eq!(main.name_source, NameSource::Origin);
    assert_eq!(main.main_root, fx.path("projects/acme_gateway"));
    assert!(main.git_error.is_none());
}

#[test]
fn repo_without_origin_uses_the_main_folder_name() {
    let fx = common::Fixtures::new();
    let mut git = GitResolver::new();
    let billing = git.project(&fx.path("projects/acme_billing"));
    assert_eq!(billing.name, "acme_billing");
    assert_eq!(billing.name_source, NameSource::Folder);
    assert_eq!(billing.origin_url, None);
    let gateway = git.project(&fx.path("projects/acme_gateway"));
    assert_ne!(billing.key, gateway.key);
}

#[test]
fn branch_and_kind_per_working_tree() {
    let fx = common::Fixtures::new();
    let mut git = GitResolver::new();
    let main = git.worktree_facts(&fx.path("projects/acme_gateway"));
    assert_eq!(main.branch.as_deref(), Some("main"));
    assert!(!main.linked);
    let wt = git.worktree_facts(&fx.path("projects/.worktrees/acme-142/acme_gateway"));
    assert_eq!(wt.branch.as_deref(), Some("feat/acme-142-service-tokens"));
    assert!(wt.linked);
    let ext = git.worktree_facts(&fx.path("external/acme-170/acme_gateway"));
    assert_eq!(ext.branch.as_deref(), Some("feat/acme-170-rate-limit"));
    assert!(ext.linked);
}

#[test]
fn worktree_outside_scan_paths_is_scanned_too() {
    let fx = common::Fixtures::new();
    let mut git = GitResolver::new();
    let mut found = vec![];
    let report = scan_with_worktrees(&[fx.projects()], &generic_filter, &mut git, |e| {
        if let ScanEvent::Found(d) = e {
            found.push(d.spec_dir);
        }
    });
    let external = fx.path("external");
    let ext: Vec<_> = found.iter().filter(|p| p.starts_with(&external)).collect();
    assert_eq!(ext.len(), 3, "{ext:?}");
    assert_eq!(report.found, 12);
    let mut unique = found.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), found.len());
}

#[test]
fn without_git_the_scan_still_works() {
    let fx = common::Fixtures::new();
    let mut git = GitResolver::with_binary("/nonexistent/quire-test-git");
    let mut found = 0;
    scan_with_worktrees(&[fx.projects()], &generic_filter, &mut git, |e| {
        if let ScanEvent::Found(_) = e {
            found += 1;
        }
    });
    assert_eq!(found, 9);
}
