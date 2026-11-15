#![allow(clippy::unwrap_used)] // test-only crate

//! Parsers against the fixture templates and against Quire's own spec (dogfooding).

use std::fs;
use std::path::Path;

use quire_core::parse::{self, PhaseState};
use quire_core::preset::Preset;

fn read(rel: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
    .unwrap()
}

const SPEC_DIR: &str = "fixtures/templates/acme_gateway/.specs/features/acme-142-service-tokens";

#[test]
fn fixture_tasks_counts_match_checkboxes() {
    let md = read(&format!("{SPEC_DIR}/tasks.md"));
    let exban = Preset::exban();
    let doc = parse::tasks(&md, exban.id_pattern.as_ref());
    let checked = md.lines().filter(|l| l.starts_with("- [x] T")).count();
    let all = md
        .lines()
        .filter(|l| l.starts_with("- [x] T") || l.starts_with("- [ ] T"))
        .count();
    assert_eq!(doc.progress(), (checked, all));
    assert_eq!(doc.progress(), (7, 8));
    let states: Vec<PhaseState> = doc.phases.iter().map(|p| p.state()).collect();
    assert_eq!(
        states,
        [
            PhaseState::Complete,
            PhaseState::InProgress,
            PhaseState::Complete
        ]
    );
    assert_eq!(
        doc.phases[1].diagram,
        vec![vec!["T4", "T5"], vec!["T4", "T6"]]
    );
    assert!(doc.phases[2].diagram.is_empty(), "phase without diagram");
    let ids: Vec<&str> = doc.phases[2].tasks.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, ["T9", "T7"], "out-of-order ids keep file order");
    let t7 = doc.task("T7").unwrap();
    assert!(t7.done && t7.hash.is_none());
    assert_eq!(doc.task("T4").unwrap().requirements, ["TOK-01", "TOK-02"]);
    let d5 = doc.detail("T5").unwrap();
    assert_eq!(d5.depends_on(), ["T4"]);
    assert_eq!(d5.field("Pronto quando").unwrap().items.len(), 1);
    assert_eq!(doc.details.len(), 8);
}

#[test]
fn fixture_header_ids_and_progress() {
    let spec = read(&format!("{SPEC_DIR}/spec.md"));
    let h = parse::header(&spec);
    assert_eq!(
        h.iter().map(|f| f.key.as_str()).collect::<Vec<_>>(),
        ["Status", "Issue", "Idioma"]
    );
    assert_eq!(
        h[1].url.as_deref(),
        Some("https://tracker.example/browse/ACME-142")
    );
    let ids: Vec<String> = parse::ids(&spec, Preset::exban().id_pattern.as_ref().unwrap())
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(ids, ["TOK-01", "TOK-02", "TOK-03", "TOK-04"]);
    let summary = read(&format!("{SPEC_DIR}/summary.md"));
    let rows = parse::summary_progress(&summary);
    let tasks = parse::tasks(&read(&format!("{SPEC_DIR}/tasks.md")), None);
    let from_table: Vec<(u32, u32)> = rows.iter().map(|r| (r.done, r.total)).collect();
    let from_tasks: Vec<(u32, u32)> = tasks
        .phases
        .iter()
        .map(|p| (p.done() as u32, p.total() as u32))
        .collect();
    assert_eq!(from_table, from_tasks);
}

#[test]
fn quire_own_tasks_file_parses() {
    let md = read(".specs/features/quire-v0-1/tasks.md");
    let doc = parse::tasks(&md, Preset::exban().id_pattern.as_ref());
    assert_eq!(doc.phases.len(), 6);
    assert_eq!(doc.progress().1, 24);
    let all = md
        .lines()
        .filter(|l| l.starts_with("- [x] T") || l.starts_with("- [ ] T"))
        .count();
    assert_eq!(doc.progress().1, all);
    assert_eq!(doc.task("T1").unwrap().requirements, ["OSS-02"]);
    assert_eq!(doc.phases[0].diagram, vec![vec!["T1", "T2", "T3", "T4"]]);
    assert_eq!(doc.details.len(), 24);
}

#[test]
fn files_out_of_format_give_empty_results() {
    let design = read(&format!("{SPEC_DIR}/design.md"));
    assert!(parse::tasks(&design, None).is_empty());
    assert!(parse::summary_progress(&design).is_empty());
    let json = read(&format!("{SPEC_DIR}/payload.json"));
    assert!(parse::header(&json).is_empty());
    assert!(parse::tasks(&json, None).is_empty());
}

#[test]
fn fixture_jsonl_with_invalid_and_truncated_lines() {
    let dir = "fixtures/templates/acme_billing/.specs/features/bill-31-invoice-export";
    let gates: parse::Jsonl<parse::GateRun> = parse::jsonl(&read(&format!("{dir}/gates.jsonl")));
    assert_eq!(gates.items.len(), 3);
    assert_eq!(gates.skipped, 2);
    assert_eq!(gates.skipped_lines, [2, 5]);
    let counts = parse::gate_counts(&gates.items);
    assert_eq!((counts.runs, counts.passes, counts.fails), (3, 2, 1));
    let fail = gates.items.iter().find(|g| g.failed()).unwrap();
    assert_eq!(fail.exit, Some(101));
    assert_eq!(fail.tail.len(), 2);
    assert_eq!(fail.cwd.as_deref(), Some("/home/dev/acme/acme_billing"));
    let lessons: parse::Jsonl<parse::Lesson> = parse::jsonl(&read(&format!("{dir}/lessons.jsonl")));
    assert_eq!((lessons.items.len(), lessons.skipped), (1, 0));
    assert_eq!(
        lessons.items[0].signal.as_deref(),
        Some("spec_precision_gap")
    );

    let ok: parse::Jsonl<parse::GateRun> = parse::jsonl(&read(&format!("{SPEC_DIR}/gates.jsonl")));
    assert_eq!((ok.items.len(), ok.skipped), (5, 0));
    let mut runs = ok.items;
    parse::newest_first(&mut runs);
    assert_eq!(runs[0].level.as_deref(), Some("final"));
}
