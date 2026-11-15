#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use std::path::PathBuf;

use quire_core::Config;
use quire_core::git::GitResolver;
use quire_core::model::{InstanceKind, InstanceState, Project, Spec, build_projects};
use quire_core::scan::{ScanEvent, scan_with_worktrees};

fn projects_with(paths: &[PathBuf], config: &Config) -> Vec<Project> {
    let mut git = GitResolver::new();
    let mut found = vec![];
    scan_with_worktrees(paths, &config.scan_filter(), &mut git, |e| {
        if let ScanEvent::Found(d) = e {
            found.push(d);
        }
    });
    build_projects(&found, &mut git, config)
}

fn projects_from(paths: &[PathBuf]) -> Vec<Project> {
    projects_with(paths, &Config::default())
}

fn spec<'a>(projects: &'a [Project], project: &str, folder: &str) -> &'a Spec {
    projects
        .iter()
        .find(|p| p.name == project)
        .and_then(|p| p.specs.iter().find(|s| s.folder == folder))
        .unwrap()
}

#[test]
fn groups_by_project_and_folder() {
    let fx = common::Fixtures::new();
    let projects = projects_from(&[fx.projects()]);
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
    let fx = common::Fixtures::new();
    let projects = projects_from(&[fx.projects()]);
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
    let fx = common::Fixtures::new();
    let projects = projects_from(&[fx.projects()]);
    let s = spec(&projects, "acme_gateway", "acme-170-rate-limit");
    assert_eq!(s.versions_badge(), Some(3));
    let default = s.default_instance().unwrap();
    assert!(
        default.spec_dir.starts_with(fx.path("external")),
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
    let fx = common::Fixtures::new();
    let projects = projects_from(&[fx.projects()]);
    let a = spec(&projects, "acme_gateway", "acme-200-shared-auth");
    let b = spec(&projects, "acme_billing", "acme-200-shared-auth");
    assert_eq!(b.instance_count(), 1);
    assert_ne!(a.instances[0].content_hash, b.instances[0].content_hash);
}

#[test]
fn overlapping_scan_paths_do_not_duplicate_instances() {
    let fx = common::Fixtures::new();
    let once = projects_from(&[fx.projects()]);
    let twice = projects_from(&[
        fx.projects(),
        fx.path("projects/acme_gateway"),
        fx.root().to_path_buf(),
    ]);
    assert_eq!(once, twice);
}

#[test]
fn without_preset_projects_are_generic_and_exban_is_suggested() {
    let fx = common::Fixtures::new();
    let projects = projects_from(&[fx.projects()]);
    for p in &projects {
        assert_eq!(p.preset, "generic");
        assert_eq!(p.suggested_preset.as_deref(), Some("exban"), "{}", p.name);
    }
    assert_eq!(
        spec(&projects, "acme_gateway", "acme-142-service-tokens").issue_key,
        None
    );
}

#[test]
fn configured_preset_alias_and_issue_keys() {
    let fx = common::Fixtures::new();
    let config =
        Config::parse("[projects.acme_gateway]\npreset = \"exban\"\nalias = \"gateway\"\n")
            .unwrap();
    let projects = projects_with(&[fx.projects()], &config);
    let gw = projects.iter().find(|p| p.name == "acme_gateway").unwrap();
    assert_eq!(gw.display_name(), "gateway");
    assert_eq!(gw.preset, "exban");
    assert_eq!(gw.suggested_preset, None);
    let s = spec(&projects, "acme_gateway", "acme-142-service-tokens");
    assert_eq!(s.issue_key.as_deref(), Some("ACME-142"));
    // exban hashes only its own files: notes.md and payload.json do not count.
    let generic = projects_from(&[fx.projects()]);
    let g = spec(&generic, "acme_gateway", "acme-142-service-tokens");
    assert_ne!(s.instances[0].content_hash, g.instances[0].content_hash);
}
