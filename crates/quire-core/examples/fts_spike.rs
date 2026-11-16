//! T10 spike (v0.2, disposable): FTS5 in SQLite for ~300 synthetic specs.
//!
//! Builds the schema of the design (TD-15: documents deduplicated by content hash, one FTS row
//! per section, `unicode61 remove_diacritics 2`) in a temporary database and measures size,
//! full indexing, reindexing one file and query latency. Content is invented pt-BR text; no
//! real data. Run with:
//!
//! ```text
//! cargo run --release -p quire-core --example fts_spike
//! ```
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Instant;

use rusqlite::{Connection, params};

const SPECS: usize = 300;
const FILES: [(&str, usize); 7] = [
    ("spec.md", 7_000),
    ("design.md", 9_000),
    ("tasks.md", 6_000),
    ("summary.md", 4_000),
    ("validation.md", 2_500),
    ("research.md", 3_000),
    ("notes.md", 1_500),
];
const WORDS: &[&str] = &[
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
    "observador",
    "configuração",
    "exceção",
    "lição",
    "gate",
    "pasta",
    "arquivo",
    "busca",
    "conflito",
    "escrita",
    "leitura",
    "cópia",
    "instância",
    "relatório",
    "versão",
    "migração",
    "autenticação",
    "token",
    "faturamento",
    "estorno",
    "paginação",
    "limite",
    "taxa",
    "cliente",
    "serviço",
    "rotação",
    "segredo",
    "nó",
    "árvore",
    "diagrama",
    "sequência",
    "estado",
    "decisão",
    "risco",
    "mitigação",
    "ênfase",
    "pré-requisito",
    "útil",
    "possível",
    "também",
    "não",
    "então",
    "porém",
    "após",
    "até",
];

/// Deterministic pseudo-random numbers (xorshift), so every run indexes the same text.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[(self.next() % xs.len() as u64) as usize]
    }
}

fn document(rng: &mut Rng, title: &str, size: usize, rare: bool) -> String {
    let mut out = format!("# {title}\n\n");
    let mut section = 0;
    while out.len() < size {
        section += 1;
        out.push_str(&format!(
            "\n## {} {} {section}\n\n",
            rng.pick(WORDS),
            rng.pick(WORDS)
        ));
        for _ in 0..(3 + rng.next() % 4) {
            let words: Vec<&str> = (0..(12 + rng.next() % 20))
                .map(|_| rng.pick(WORDS))
                .collect();
            out.push_str(&words.join(" "));
            out.push_str(".\n\n");
        }
        if section % 3 == 0 {
            out.push_str("```rust\nfn exemplo() { let ação = 1; }\n```\n\n");
        }
    }
    if rare {
        out.push_str("\n## Anexo\n\nO zircônio aparece só aqui.\n");
    }
    out
}

/// Sections: heading line + body until the next heading, headings inside fences ignored.
fn sections(text: &str) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = vec![(String::new(), String::new(), 1)];
    let mut fence = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fence = !fence;
        }
        if !fence && line.starts_with('#') {
            out.push((
                line.trim_start_matches('#').trim().to_owned(),
                String::new(),
                i + 1,
            ));
        } else if let Some(last) = out.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    out
}

/// TD-16: every term quoted as an FTS5 string, the last one as a prefix; never raw syntax.
fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
        .collect();
    let n = terms.len();
    if n == 0 {
        return None;
    }
    Some(
        terms
            .into_iter()
            .enumerate()
            .map(|(i, t)| if i + 1 == n { format!("{t}*") } else { t })
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn schema(db: &Connection) {
    db.execute_batch(
        "PRAGMA journal_mode = WAL;
         CREATE TABLE fts_docs (id INTEGER PRIMARY KEY, content_hash TEXT UNIQUE NOT NULL);
         CREATE TABLE file_docs (instance_id INTEGER NOT NULL, name TEXT NOT NULL,
                                 doc_id INTEGER NOT NULL REFERENCES fts_docs(id),
                                 PRIMARY KEY (instance_id, name));
         CREATE VIRTUAL TABLE fts USING fts5(heading, body, doc_id UNINDEXED, line UNINDEXED,
                                             tokenize = 'unicode61 remove_diacritics 2');",
    )
    .unwrap();
}

/// Indexes one file of one instance; returns the number of FTS rows inserted (0 when the
/// same content is already indexed for another copy).
fn index_file(db: &Connection, instance: i64, name: &str, text: &str) -> usize {
    let hash = format!("{:032x}", xxhash_rust::xxh3::xxh3_128(text.as_bytes()));
    let existing: Option<i64> = db
        .query_row(
            "SELECT id FROM fts_docs WHERE content_hash = ?1",
            [&hash],
            |r| r.get(0),
        )
        .ok();
    let (doc, inserted) = match existing {
        Some(id) => (id, 0),
        None => {
            db.execute("INSERT INTO fts_docs (content_hash) VALUES (?1)", [&hash])
                .unwrap();
            let id = db.last_insert_rowid();
            let mut stmt = db
                .prepare_cached(
                    "INSERT INTO fts (heading, body, doc_id, line) VALUES (?1, ?2, ?3, ?4)",
                )
                .unwrap();
            let secs = sections(text);
            for (h, b, line) in &secs {
                stmt.execute(params![h, b, id, *line as i64]).unwrap();
            }
            (id, secs.len())
        }
    };
    db.execute(
        "INSERT OR REPLACE INTO file_docs (instance_id, name, doc_id) VALUES (?1, ?2, ?3)",
        params![instance, name, doc],
    )
    .unwrap();
    inserted
}

/// Removes documents no file points to any more, once per reconciliation (design, T21).
fn gc(db: &Connection) -> usize {
    let n = db
        .execute(
            "DELETE FROM fts WHERE doc_id IN (SELECT id FROM fts_docs WHERE id NOT IN (SELECT doc_id FROM file_docs))",
            [],
        )
        .unwrap();
    db.execute(
        "DELETE FROM fts_docs WHERE id NOT IN (SELECT doc_id FROM file_docs)",
        [],
    )
    .unwrap();
    n
}

fn search(db: &Connection, input: &str) -> Vec<(i64, String, String)> {
    let Some(q) = fts_query(input) else {
        return vec![];
    };
    let mut stmt = db
        .prepare_cached(
            "SELECT f.instance_id, f.name, snippet(fts, 1, char(2), char(3), '…', 12)
             FROM fts JOIN file_docs f ON f.doc_id = fts.doc_id
             WHERE fts MATCH ?1 ORDER BY bm25(fts, 5.0, 1.0) LIMIT 200",
        )
        .unwrap();
    stmt.query_map([q], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn percentile(mut xs: Vec<f64>, p: f64) -> f64 {
    xs.sort_by(f64::total_cmp);
    xs[((p / 100.0) * xs.len() as f64)
        .floor()
        .min(xs.len() as f64 - 1.0) as usize]
}

fn main() {
    let dir = std::env::temp_dir().join(format!("quire-fts-spike-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("index.sqlite");
    let db = Connection::open(&path).unwrap();
    schema(&db);

    let mut rng = Rng(0x5eed_1234_abcd_ef01);
    let mut corpus: Vec<(i64, &str, String)> = Vec::new();
    for spec in 0..SPECS {
        for (i, (name, size)) in FILES.iter().enumerate() {
            let rare = spec == 137 && i == 1;
            let text = document(&mut rng, &format!("ACME-{spec} {name}"), *size, rare);
            corpus.push((spec as i64, name, text));
        }
    }
    let raw: usize = corpus.iter().map(|c| c.2.len()).sum();

    let t = Instant::now();
    db.execute_batch("BEGIN").unwrap();
    let mut rows = 0;
    for (inst, name, text) in &corpus {
        rows += index_file(&db, *inst, name, text);
    }
    gc(&db);
    db.execute_batch("COMMIT").unwrap();
    let full = t.elapsed();
    db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); VACUUM;")
        .unwrap();
    let size = std::fs::metadata(&path).unwrap().len();

    // Three identical copies of spec 5 in other worktrees: no new FTS rows.
    let before: i64 = db
        .query_row("SELECT count(*) FROM fts", [], |r| r.get(0))
        .unwrap();
    let copies: Vec<_> = corpus.iter().filter(|c| c.0 == 5).cloned().collect();
    let mut dup_rows = 0;
    for k in 1..=3 {
        for (_, name, text) in &copies {
            dup_rows += index_file(&db, 10_000 + k, name, text);
        }
    }
    let after: i64 = db
        .query_row("SELECT count(*) FROM fts", [], |r| r.get(0))
        .unwrap();

    // Reindex one changed file.
    let changed = format!("{}\nNova linha com revisão.\n", corpus[42].2);
    let t = Instant::now();
    db.execute_batch("BEGIN").unwrap();
    let re_rows = index_file(&db, corpus[42].0, corpus[42].1, &changed);
    let removed = gc(&db);
    db.execute_batch("COMMIT").unwrap();
    let reindex = t.elapsed();

    println!(
        "corpus: {SPECS} specs x {} files, {:.1} MB of text",
        FILES.len(),
        raw as f64 / 1e6
    );
    println!(
        "full index: {:.0} ms, {rows} section rows, db {:.1} MB (text {:.1} MB)",
        full.as_secs_f64() * 1e3,
        size as f64 / 1e6,
        raw as f64 / 1e6
    );
    println!("3 identical copies of one spec: {dup_rows} new rows (fts {before} -> {after})");
    println!(
        "reindex one changed file: {:.1} ms, {re_rows} rows in, {removed} stale rows out",
        reindex.as_secs_f64() * 1e3
    );

    for q in [
        "zircônio",
        "zirconio",
        "acao",
        "ação",
        "tare",
        "revisão índice",
        "ACAO tarefa",
        "\"aspas",
        "a -b",
        "NEAR(x y)",
        "col:valor",
        "*",
        "AND",
        "(",
        "   ",
        "",
    ] {
        let mut times = vec![];
        let mut hits = 0;
        for _ in 0..30 {
            let t = Instant::now();
            hits = search(&db, q).len();
            times.push(t.elapsed().as_secs_f64() * 1e3);
        }
        println!(
            "query {q:?}: {hits} hits, p50 {:.2} ms, p95 {:.2} ms",
            percentile(times.clone(), 50.0),
            percentile(times, 95.0)
        );
    }
    let sample = search(&db, "zirconio");
    if let Some((inst, name, snip)) = sample.first() {
        println!(
            "snippet for zirconio: instance {inst} {name}: {}",
            snip.replace('\u{2}', "[").replace('\u{3}', "]")
        );
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
