#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use quire_core::Config;
use quire_core::git::GitResolver;
use quire_core::index::{Index, ProjectSummary, ReconcileReport};

fn reconcile(index: &mut Index, paths: &[PathBuf], config: &Config) -> ReconcileReport {
    index
        .reconcile(paths, config, &mut GitResolver::new(), |_| {})
        .unwrap()
}

/// The list without database ids, to compare caches built separately.
fn shape(list: &[ProjectSummary]) -> Vec<String> {
    let mut out = vec![];
    for p in list {
        out.push(format!(
            "P {} {} {:?} {:?}",
            p.name, p.preset, p.suggested_preset, p.git_warning
        ));
        for s in &p.specs {
            out.push(format!(
                "  S {} {:?} badge={:?} default={}",
                s.folder, s.issue_key, s.versions_badge, s.default_index
            ));
            for i in &s.instances {
                out.push(format!(
                    "    I {} {:?} {:?} same={} {:?} {:?} {:?} {:?}",
                    i.path.display(),
                    i.branch,
                    i.kind,
                    i.same_as_default,
                    i.title,
                    i.status,
                    i.issue_url,
                    i.progress
                ));
            }
        }
    }
    out
}

#[test]
fn deleting_the_cache_rebuilds_the_same_result() {
    let tmp = tempfile::tempdir().unwrap();
    let config = Config::parse("[projects.acme_gateway]\npreset = \"exban\"\n").unwrap();
    let paths = [common::projects()];
    let db = tmp.path().join("index.sqlite");
    let mut a = Index::open(&db).unwrap();
    let report = reconcile(&mut a, &paths, &config);
    assert_eq!(
        (report.found, report.reprocessed, report.unchanged),
        (12, 12, 0)
    );
    let first = shape(&a.list().unwrap());
    drop(a);
    for suffix in ["", "-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{suffix}", db.display()));
    }
    let mut b = Index::open(&db).unwrap();
    assert!(b.list().unwrap().is_empty());
    reconcile(&mut b, &paths, &config);
    assert_eq!(shape(&b.list().unwrap()), first);
}

#[test]
fn cache_holds_what_the_list_needs() {
    let mut index = Index::open_in_memory().unwrap();
    let config =
        Config::parse("[projects.acme_gateway]\npreset = \"exban\"\nalias = \"gateway\"\n")
            .unwrap();
    reconcile(&mut index, &[common::projects()], &config);
    let list = index.list().unwrap();
    let names: Vec<&str> = list.iter().map(ProjectSummary::display_name).collect();
    assert_eq!(names, ["acme_billing", "gateway"]);
    let gw = &list[1];
    assert_eq!(
        (gw.preset.as_str(), gw.suggested_preset.as_deref()),
        ("exban", None)
    );
    assert_eq!(list[0].suggested_preset.as_deref(), Some("exban"));
    let s142 = gw
        .specs
        .iter()
        .find(|s| s.folder == "acme-142-service-tokens")
        .unwrap();
    assert_eq!(s142.issue_key.as_deref(), Some("ACME-142"));
    assert_eq!((s142.instance_count(), s142.versions_badge), (3, None));
    let d = s142.default_instance().unwrap();
    assert_eq!(d.progress, Some((7, 8)));
    assert_eq!(
        d.issue_url.as_deref(),
        Some("https://tracker.example/browse/ACME-142")
    );
    assert!(d.status.as_deref().unwrap().starts_with("Concluído"));
    let s170 = gw
        .specs
        .iter()
        .find(|s| s.folder == "acme-170-rate-limit")
        .unwrap();
    assert_eq!(s170.versions_badge, Some(3));
    let d170 = s170.default_instance().unwrap();
    assert_eq!(d170.branch.as_deref(), Some("feat/acme-170-rate-limit"));
    assert_eq!(d170.progress, Some((2, 3)));
    assert_eq!(
        s170.instances.iter().filter(|i| !i.same_as_default).count(),
        2
    );
}

#[test]
fn second_scan_without_changes_rereads_nothing_and_changes_are_picked_up() {
    let ws = common::fresh_workspace("index-changes");
    let paths = [ws.join("projects")];
    let config = Config::parse("[projects.acme_gateway]\npreset = \"exban\"\n").unwrap();
    let mut index = Index::open_in_memory().unwrap();
    let first = reconcile(&mut index, &paths, &config);
    assert_eq!(first.reprocessed, 12);
    let second = reconcile(&mut index, &paths, &config);
    assert_eq!(
        (second.reprocessed, second.unchanged, second.removed),
        (0, 12, 0)
    );

    // Edit one file: only that instance is re-read, and the list shows the new progress.
    let tasks = ws.join("projects/acme_gateway/.specs/features/acme-142-service-tokens/tasks.md");
    let text = fs::read_to_string(&tasks)
        .unwrap()
        .replace("- [ ] T6:", "- [x] T6:");
    fs::write(&tasks, text).unwrap();
    let third = reconcile(&mut index, &paths, &config);
    assert_eq!((third.reprocessed, third.unchanged), (1, 11));
    let list = index.list().unwrap();
    let s = list
        .iter()
        .flat_map(|p| &p.specs)
        .find(|s| s.folder == "acme-142-service-tokens")
        .unwrap();
    assert_eq!(s.versions_badge, Some(3), "now one copy diverges");
    assert_eq!(
        s.default_instance().unwrap().progress,
        Some((8, 8)),
        "the edited copy is the newest"
    );

    // Remove a spec folder: it disappears from the index.
    fs::remove_dir_all(ws.join("projects/acme_billing/.specs/features/bill-40-refunds")).unwrap();
    let fourth = reconcile(&mut index, &paths, &config);
    assert_eq!(fourth.removed, 1);
    let folders: Vec<String> = index
        .list()
        .unwrap()
        .iter()
        .flat_map(|p| p.specs.iter().map(|s| s.folder.clone()))
        .collect();
    assert!(!folders.contains(&"bill-40-refunds".to_owned()));

    // Changing a project's preset re-reads its instances.
    let generic = Config::default();
    let fifth = reconcile(&mut index, &paths, &generic);
    assert_eq!(fifth.reprocessed, 9, "the 9 acme_gateway instances");
}

fn synthetic_workspace(root: &Path, repos: usize, specs_per_repo: usize) {
    for r in 0..repos {
        for s in 0..specs_per_repo {
            let dir = root.join(format!(
                "repo-{r:02}/.specs/features/syn-{r}{s:02}-spec-{s}"
            ));
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("spec.md"),
                format!("# SYN-{r}{s:02} — Spec {s}\n\n**Status:** Em andamento\n"),
            )
            .unwrap();
            fs::write(
                dir.join("tasks.md"),
                "### Fase 1: A\n- [x] T1: a\n- [ ] T2: b\n",
            )
            .unwrap();
            fs::write(dir.join("summary.md"), "# Resumo\n").unwrap();
            fs::write(dir.join("gates.jsonl"), "{\"status\":\"pass\"}\n").unwrap();
        }
    }
}

#[test]
fn three_hundred_specs_list_from_cache_under_one_second() {
    let tmp = tempfile::tempdir().unwrap();
    let ws = tmp.path().join("ws");
    synthetic_workspace(&ws, 10, 30);
    let db = tmp.path().join("index.sqlite");
    let config = Config::default();
    let mut index = Index::open(&db).unwrap();
    let t0 = Instant::now();
    let cold = reconcile(&mut index, std::slice::from_ref(&ws), &config);
    let t_cold = t0.elapsed();
    assert_eq!(cold.found, 300);
    let t1 = Instant::now();
    let warm = reconcile(&mut index, std::slice::from_ref(&ws), &config);
    let t_warm = t1.elapsed();
    assert_eq!(warm.unchanged, 300);
    drop(index);

    // Startup: open the cache with a new connection and list.
    let t2 = Instant::now();
    let index = Index::open(&db).unwrap();
    let list = index.list().unwrap();
    let t_list = t2.elapsed();
    assert_eq!(list.iter().map(|p| p.specs.len()).sum::<usize>(), 300);
    eprintln!(
        "IDX-03: 300 specs; first reconcile {:.0} ms, second (no changes) {:.0} ms, open + list from cache {:.1} ms",
        t_cold.as_secs_f64() * 1000.0,
        t_warm.as_secs_f64() * 1000.0,
        t_list.as_secs_f64() * 1000.0
    );
    assert!(t_list.as_secs_f64() < 1.0, "list took {t_list:?}");
}
