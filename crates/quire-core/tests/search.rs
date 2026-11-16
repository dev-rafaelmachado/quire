#![allow(clippy::unwrap_used)] // test-only crate

mod common;

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use quire_core::Config;
use quire_core::git::GitResolver;
use quire_core::index::{Index, ReconcileReport};
use quire_core::search::{SearchQuery, sections};

fn reconcile(index: &mut Index, paths: &[PathBuf], config: &Config) -> ReconcileReport {
    index
        .reconcile(paths, config, &mut GitResolver::new(), |_| {})
        .unwrap()
}

fn q(text: &str) -> SearchQuery {
    SearchQuery {
        text: text.into(),
        ..SearchQuery::default()
    }
}

fn config() -> Config {
    Config::parse("[projects.acme_gateway]\npreset = \"exban\"\n[projects.acme_billing]\npreset = \"exban\"\n").unwrap()
}

#[test]
fn a_version_1_cache_is_rebuilt_as_version_2() {
    let tmp = tempfile::tempdir().unwrap();
    let db = tmp.path().join("index.sqlite");
    {
        let conn = rusqlite::Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE projects (id INTEGER PRIMARY KEY); PRAGMA user_version = 1;",
        )
        .unwrap();
    }
    let fx = common::Fixtures::new();
    let mut index = Index::open(&db).unwrap();
    let report = reconcile(&mut index, &[fx.projects()], &config());
    assert!(report.found > 0 && report.fts_sections > 0, "{report:?}");
    assert!(!index.search(&q("tokens")).unwrap().is_empty());
    let version: i64 = rusqlite::Connection::open(&db)
        .unwrap()
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);
}

#[test]
fn changing_one_file_reindexes_only_that_file() {
    let fx = common::Fixtures::new();
    let mut index = Index::open_in_memory().unwrap();
    let paths = [fx.projects()];
    let first = reconcile(&mut index, &paths, &config());
    assert!(first.fts_sections > 0);
    // Nothing changed: nothing indexed.
    assert_eq!(reconcile(&mut index, &paths, &config()).fts_sections, 0);

    let file = fx
        .projects()
        .join("acme_billing/.specs/features/bill-31-invoice-export/design.md");
    let text = format!(
        "{}\n## Anexo novo\n\nPalavra zircônio só aqui.\n",
        fs::read_to_string(&file).unwrap()
    );
    fs::write(&file, &text).unwrap();
    let again = reconcile(&mut index, &paths, &config());
    // Only the changed design.md is split again; its siblings keep their documents.
    assert_eq!(again.fts_sections, sections(&text).len(), "{again:?}");
    let hits = index.search(&q("zirconio")).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(
        (hits[0].file.as_str(), hits[0].heading.as_str()),
        ("design.md", "Anexo novo")
    );
    assert!(
        hits[0].snippet.contains("\u{2}zircônio\u{3}"),
        "{:?}",
        hits[0].snippet
    );

    // Removing the text removes the hit: the stale document is collected.
    fs::write(&file, "# Design\n\nCurto.\n").unwrap();
    reconcile(&mut index, &paths, &config());
    assert!(index.search(&q("zircônio")).unwrap().is_empty());
}

#[test]
fn accents_case_prefix_and_all_terms() {
    let fx = common::Fixtures::new();
    let mut index = Index::open_in_memory().unwrap();
    reconcile(&mut index, &[fx.projects()], &config());
    let ac = index.search(&q("integracoes")).unwrap();
    let acc = index.search(&q("Integrações")).unwrap();
    assert!(!ac.is_empty());
    assert_eq!(ac.len(), acc.len());
    // Prefix on the last term only.
    let prefix = index.search(&q("integra")).unwrap();
    assert!(prefix.len() >= ac.len());
    assert!(index.search(&q("integra tokens")).unwrap().len() <= prefix.len());
    // All terms must match (AND).
    assert!(index.search(&q("tokens zzzznada")).unwrap().is_empty());
}

#[test]
fn fts_syntax_typed_by_the_user_never_errors() {
    let fx = common::Fixtures::new();
    let mut index = Index::open_in_memory().unwrap();
    reconcile(&mut index, &[fx.projects()], &config());
    for text in [
        "\"",
        "\"tokens",
        "tokens\"",
        "*",
        "-tokens",
        "a -b",
        "spec:md",
        "(",
        ")",
        "AND",
        "OR",
        "NOT tokens",
        "NEAR(a b)",
        "^tokens",
        "",
        "   ",
        "{x}",
    ] {
        assert!(index.search(&q(text)).is_ok(), "{text:?}");
    }
    assert!(!index.search(&q("\"tokens")).unwrap().is_empty());
}

#[test]
fn jsonl_and_other_files_are_never_indexed() {
    let fx = common::Fixtures::new();
    let mut index = Index::open_in_memory().unwrap();
    let dir = fx
        .projects()
        .join("acme_billing/.specs/features/bill-31-invoice-export");
    let mut lessons = fs::read_to_string(dir.join("lessons.jsonl")).unwrap();
    lessons.push_str("{\"kind\":\"x\",\"text\":\"palavrasolanojsonl\"}\n");
    fs::write(dir.join("lessons.jsonl"), lessons).unwrap();
    fs::write(dir.join("notes.txt"), "palavrasolanotxt\n").unwrap();
    reconcile(&mut index, &[fx.projects()], &config());
    assert!(index.search(&q("palavrasolanojsonl")).unwrap().is_empty());
    assert!(index.search(&q("palavrasolanotxt")).unwrap().is_empty());
}

#[test]
fn identical_copies_share_one_section_row() {
    let fx = common::Fixtures::new();
    let mut index = Index::open_in_memory().unwrap();
    reconcile(&mut index, &[fx.projects()], &config());
    let hits = index
        .search(&q("Tokens de serviço para integrações"))
        .unwrap();
    // ACME-142 lives in three identical copies: same section row, three instances.
    let spec = hits[0].spec_id;
    let same: Vec<_> = hits
        .iter()
        .filter(|h| h.spec_id == spec && h.file == "spec.md" && h.line == 1)
        .collect();
    assert!(same.len() >= 2, "{hits:?}");
    assert!(same.iter().all(|h| h.section == same[0].section));
    // Filters (SRC-05).
    let only_spec = index
        .search(&SearchQuery {
            text: "tokens".into(),
            spec_id: Some(spec),
            ..SearchQuery::default()
        })
        .unwrap();
    assert!(only_spec.iter().all(|h| h.spec_id == spec));
    let only_tasks = index
        .search(&SearchQuery {
            text: "tokens".into(),
            file: Some("tasks.md".into()),
            ..SearchQuery::default()
        })
        .unwrap();
    assert!(only_tasks.iter().all(|h| h.file == "tasks.md"));
}

/// SRC-08 with the generator of the T10 spike's sizes: ~300 specs. Ignored by default (slow);
/// run with `cargo test --release -p quire-core --test search -- --ignored --nocapture`.
#[test]
#[ignore]
fn three_hundred_specs_answer_within_100_ms() {
    let tmp = tempfile::tempdir().unwrap();
    let words = [
        "ação",
        "tarefa",
        "critério",
        "validação",
        "índice",
        "sessão",
        "revisão",
        "worktree",
        "agente",
        "commit",
        "cache",
        "fase",
        "requisito",
        "preset",
        "conflito",
        "escrita",
        "leitura",
        "cópia",
        "instância",
        "migração",
        "token",
        "paginação",
        "limite",
        "cliente",
        "serviço",
        "decisão",
        "risco",
    ];
    let mut seed: u64 = 0x5eed;
    let mut next = || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut text_bytes = 0;
    for p in 0..10 {
        let repo = tmp.path().join(format!("repo{p}"));
        for s in 0..30 {
            let dir = repo.join(format!(".specs/features/acme-{p}{s:02}-x"));
            fs::create_dir_all(&dir).unwrap();
            for (name, size) in [
                ("spec.md", 7000),
                ("design.md", 9000),
                ("tasks.md", 6000),
                ("summary.md", 4000),
                ("validation.md", 2500),
            ] {
                let mut t = format!("# ACME-{p}{s:02} {name}\n");
                while t.len() < size {
                    t.push_str(&format!(
                        "\n## {} {}\n\n",
                        words[(next() % 27) as usize],
                        words[(next() % 27) as usize]
                    ));
                    for _ in 0..4 {
                        let line: Vec<&str> =
                            (0..16).map(|_| words[(next() % 27) as usize]).collect();
                        t.push_str(&line.join(" "));
                        t.push_str(".\n");
                    }
                }
                text_bytes += t.len();
                fs::write(dir.join(name), t).unwrap();
            }
        }
    }
    let db = tmp.path().join("index.sqlite");
    let mut index = Index::open(&db).unwrap();
    let t = Instant::now();
    let report = reconcile(&mut index, &[tmp.path().to_path_buf()], &Config::default());
    let indexing = t.elapsed();
    drop(index);
    let size = fs::metadata(&db).unwrap().len()
        + fs::metadata(tmp.path().join("index.sqlite-wal"))
            .map(|m| m.len())
            .unwrap_or(0);
    let index = Index::open(&db).unwrap();
    let mut worst = 0.0f64;
    for text in [
        "acao",
        "revisão índice",
        "tare",
        "zircônio",
        "cliente serviço",
    ] {
        let mut times: Vec<f64> = (0..30)
            .map(|_| {
                let t = Instant::now();
                index.search(&q(text)).unwrap();
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        times.sort_by(f64::total_cmp);
        let p95 = times[28];
        worst = worst.max(p95);
        println!("{text:?}: p50 {:.2} ms, p95 {p95:.2} ms", times[15]);
    }
    println!(
        "specs {} indexing {:.0} ms, db+wal {:.1} MB, text {:.1} MB",
        report.found,
        indexing.as_secs_f64() * 1e3,
        size as f64 / 1e6,
        text_bytes as f64 / 1e6
    );
    assert_eq!(report.found, 300);
    assert!(worst <= 100.0, "p95 {worst} ms");
}
