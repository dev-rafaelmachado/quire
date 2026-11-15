#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use std::path::PathBuf;

use quire_core::git::GitResolver;
use quire_core::model::{InstanceKind, InstanceState, Project, Spec, build_projects};
use quire_core::scan::{ScanEvent, generic_filter, scan_with_worktrees};

fn projects_from(paths: &[PathBuf]) -> Vec<Project> {
    let mut git = GitResolver::new();
    let mut found = vec![];
    scan_with_worktrees(paths, &generic_filter, &mut git, |e| {
        if let ScanEvent::Found(d) = e {
            found.push(d);
        }
    });
    let is_primary = |f: &str| f.ends_with(".md") || f.ends_with(".jsonl");
    build_projects(&found, &mut git, &is_primary)
}

fn spec<'a>(projects: &'a [Project], project: &str, folder: &str) -> &'a Spec {
    projects
        .iter()
        .find(|p| p.name == project)
        .and_then(|p| p.specs.iter().find(|s| s.folder == folder))
        .unwrap()
}

fn ws(rel: &str) -> PathBuf {
    common::workspace().join(rel).canonicalize().unwrap()
}

#[test]
fn groups_by_project_and_folder() {
    let projects = projects_from(&[common::projects()]);
    let names: Vec<&str> = projects.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["acme_billing", "acme_gateway"]);
    let folders = |i: usize| {
        projects[i]
            .specs
            .iter()
            .map(|s| s.folder.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        folders(0),
        [
            "acme-200-shared-auth",
            "bill-31-invoice-export",
            "bill-40-refunds"
        ]
    );
    assert_eq!(
        folders(1),
        [
            "acme-142-service-tokens",
            "acme-170-rate-limit",
            "acme-200-shared-auth"
        ]
    );
}

#[test]
fn identical_copies_are_one_entry_without_badge() {
    let projects = projects_from(&[common::projects()]);
    let s = spec(&projects, "acme_gateway", "acme-142-service-tokens");
    assert_eq!(s.instance_count(), 3);
    assert_eq!(s.versions_badge(), None);
    // Same mtime everywhere: the alphabetically first path wins.
    let default = s.default_instance().unwrap();
    assert_eq!(
        default.spec_dir,
        s.instances
            .iter()
            .map(|i| i.spec_dir.clone())
            .min()
            .unwrap()
    );
}

#[test]
fn divergent_copy_shows_badge_and_opens_newest() {
    let projects = projects_from(&[common::projects()]);
    let s = spec(&projects, "acme_gateway", "acme-170-rate-limit");
    assert_eq!(s.versions_badge(), Some(3));
    let default = s.default_instance().unwrap();
    assert!(
        default.spec_dir.starts_with(ws("external")),
        "{:?}",
        default.spec_dir
    );
    assert_eq!(default.kind, InstanceKind::Worktree);
    assert_eq!(default.branch.as_deref(), Some("feat/acme-170-rate-limit"));
    let states: Vec<_> = (0..s.instance_count())
        .map(|i| s.state_of(i, s.default_index).unwrap())
        .collect();
    assert_eq!(
        states
            .iter()
            .filter(|st| **st == InstanceState::Diverges)
            .count(),
        2
    );
    let main = s
        .instances
        .iter()
        .find(|i| i.kind == InstanceKind::Checkout)
        .unwrap();
    assert_eq!(main.branch.as_deref(), Some("main"));
}

#[test]
fn same_folder_in_two_projects_stays_separate() {
    let projects = projects_from(&[common::projects()]);
    let a = spec(&projects, "acme_gateway", "acme-200-shared-auth");
    let b = spec(&projects, "acme_billing", "acme-200-shared-auth");
    assert_eq!(b.instance_count(), 1);
    assert_ne!(a.instances[0].content_hash, b.instances[0].content_hash);
}

#[test]
fn overlapping_scan_paths_do_not_duplicate_instances() {
    let once = projects_from(&[common::projects()]);
    let twice = projects_from(&[
        common::projects(),
        ws("projects/acme_gateway"),
        common::workspace().to_path_buf(),
    ]);
    assert_eq!(once, twice);
}
