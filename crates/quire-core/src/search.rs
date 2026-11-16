//! Full-text search of the Markdown files of the specs (SRC-01..08, TD-15, TD-16).
//!
//! Lives in the index cache (still disposable): documents are deduplicated by content hash, so
//! identical copies in several worktrees are indexed once; each document is split in sections
//! (heading + body up to the next heading, headings inside code blocks ignored) so a result
//! leads to its section. Only `.md` files are indexed: `gates.jsonl` and `lessons.jsonl` never
//! are (P7). The user's text never reaches `MATCH` raw: every term is quoted, and only the last
//! one is a prefix.

use rusqlite::{Connection, OptionalExtension, params};

use crate::error::IndexError;
use crate::parse::Fence;

/// Files larger than this are not indexed (same order as the editing limit, A8).
pub const MAX_INDEXED_BYTES: u64 = 4 * 1024 * 1024;
/// Results are capped; the UI asks to refine (spec, edge cases).
pub const DEFAULT_LIMIT: usize = 200;

pub(crate) const SCHEMA: &str = "
-- One row per distinct content: copies in several worktrees share it (SRC-04).
CREATE TABLE fts_docs (
  id INTEGER PRIMARY KEY,
  content_hash TEXT NOT NULL UNIQUE
);
CREATE TABLE file_docs (
  instance_id INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  doc_id INTEGER NOT NULL REFERENCES fts_docs(id),
  PRIMARY KEY (instance_id, name)
);
CREATE INDEX file_docs_doc ON file_docs(doc_id);
-- One row per section, to take a result to its heading (SRC-06).
CREATE VIRTUAL TABLE fts USING fts5(
  heading, body, doc_id UNINDEXED, line UNINDEXED,
  tokenize = 'unicode61 remove_diacritics 2'
);
";

fn sql(e: rusqlite::Error) -> IndexError {
    IndexError::Sqlite(e.to_string())
}

/// Whether a file of a spec folder goes to the index (P7).
pub fn indexable(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".md") && !crate::write::is_quire_temp(name)
}

/// `(heading, body, first line)` of each section; text before the first heading is a section
/// with an empty heading.
pub fn sections(text: &str) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = vec![(String::new(), String::new(), 1)];
    let mut fence = Fence::default();
    for (i, line) in text.lines().enumerate() {
        let in_code = fence.step(line);
        let t = line.trim_start();
        let heading = !in_code && t.starts_with('#') && {
            let hashes = t.chars().take_while(|c| *c == '#').count();
            hashes <= 6 && t[hashes..].starts_with([' ', '\t'])
        };
        if heading {
            out.push((
                t.trim_start_matches('#').trim().to_owned(),
                String::new(),
                i + 1,
            ));
        } else if let Some(last) = out.last_mut() {
            last.1.push_str(line);
            last.1.push('\n');
        }
    }
    out.retain(|(h, b, _)| !h.is_empty() || !b.trim().is_empty());
    out
}

/// Links file `name` of `instance` to the document of `text`, indexing the document only when
/// no other file has the same content. Returns the number of FTS rows inserted.
pub(crate) fn index_file(
    conn: &Connection,
    instance: i64,
    name: &str,
    text: &str,
) -> Result<usize, IndexError> {
    let hash = format!("{:032x}", xxhash_rust::xxh3::xxh3_128(text.as_bytes()));
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM fts_docs WHERE content_hash = ?1",
            [&hash],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql)?;
    let (doc, inserted) = match existing {
        Some(id) => (id, 0),
        None => {
            let id: i64 = conn
                .query_row(
                    "INSERT INTO fts_docs (content_hash) VALUES (?1) RETURNING id",
                    [&hash],
                    |r| r.get(0),
                )
                .map_err(sql)?;
            let mut ins = conn
                .prepare_cached(
                    "INSERT INTO fts (heading, body, doc_id, line) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(sql)?;
            let secs = sections(text);
            for (h, b, line) in &secs {
                ins.execute(params![h, b, id, *line as i64]).map_err(sql)?;
            }
            (id, secs.len())
        }
    };
    conn.execute(
        "INSERT OR REPLACE INTO file_docs (instance_id, name, doc_id) VALUES (?1, ?2, ?3)",
        params![instance, name, doc],
    )
    .map_err(sql)?;
    Ok(inserted)
}

/// Drops documents no file points to any more; once per reconciliation (T10: per file it was
/// quadratic).
pub(crate) fn gc(conn: &Connection) -> Result<usize, IndexError> {
    let n = conn
        .execute(
            "DELETE FROM fts WHERE doc_id IN
               (SELECT id FROM fts_docs WHERE id NOT IN (SELECT doc_id FROM file_docs))",
            [],
        )
        .map_err(sql)?;
    conn.execute(
        "DELETE FROM fts_docs WHERE id NOT IN (SELECT doc_id FROM file_docs)",
        [],
    )
    .map_err(sql)?;
    Ok(n)
}

/// TD-16: the FTS5 expression for what the user typed, or `None` when there is nothing to
/// search. Every term is a quoted string (inner quotes doubled); the last one is a prefix; the
/// space between them is an implicit AND (SRC-03, SRC-07).
pub fn fts_query(input: &str) -> Option<String> {
    let terms: Vec<String> = input
        .split_whitespace()
        .filter(|t| t.chars().any(char::is_alphanumeric))
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

/// What to search and where (SRC-05).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    pub text: String,
    pub project_id: Option<i64>,
    /// File name, e.g. `spec.md`.
    pub file: Option<String>,
    /// Only this spec ("na spec aberta").
    pub spec_id: Option<i64>,
    pub limit: Option<usize>,
}

/// One section that matched, in one instance that holds it. Identical copies come back once
/// per instance; the caller groups them (SRC-04).
#[derive(Debug, Clone, PartialEq)]
pub struct SearchRow {
    /// Section row of the FTS table: equal for identical copies.
    pub section: i64,
    pub instance_id: i64,
    pub spec_id: i64,
    pub project_id: i64,
    pub file: String,
    pub heading: String,
    pub line: i64,
    /// Body excerpt with the matches between `\u{2}` and `\u{3}` (never HTML).
    pub snippet: String,
    /// bm25: lower is better.
    pub rank: f64,
}

pub(crate) fn search(conn: &Connection, q: &SearchQuery) -> Result<Vec<SearchRow>, IndexError> {
    let Some(expr) = fts_query(&q.text) else {
        return Ok(vec![]);
    };
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, 1000) as i64;
    let mut stmt = conn
        .prepare_cached(
            "SELECT fts.rowid, f.instance_id, i.spec_id, s.project_id, f.name, fts.heading, fts.line,
                    snippet(fts, 1, char(2), char(3), '…', 12), bm25(fts, 5.0, 1.0) AS rank
             FROM fts
             JOIN file_docs f ON f.doc_id = fts.doc_id
             JOIN instances i ON i.id = f.instance_id
             JOIN specs s ON s.id = i.spec_id
             WHERE fts MATCH ?1
               AND (?2 IS NULL OR s.project_id = ?2)
               AND (?3 IS NULL OR f.name = ?3)
               AND (?4 IS NULL OR s.id = ?4)
             ORDER BY rank, f.instance_id
             LIMIT ?5",
        )
        .map_err(sql)?;
    let rows = stmt
        .query_map(params![expr, q.project_id, q.file, q.spec_id, limit], |r| {
            Ok(SearchRow {
                section: r.get(0)?,
                instance_id: r.get(1)?,
                spec_id: r.get(2)?,
                project_id: r.get(3)?,
                file: r.get(4)?,
                heading: r.get(5)?,
                line: r.get(6)?,
                snippet: r.get(7)?,
                rank: r.get(8)?,
            })
        })
        .map_err(sql)?;
    rows.collect::<Result<_, _>>().map_err(sql)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_are_built_never_passed_through() {
        assert_eq!(fts_query("ação"), Some("\"ação\"*".into()));
        assert_eq!(
            fts_query("revisão  índice"),
            Some("\"revisão\" \"índice\"*".into())
        );
        assert_eq!(fts_query("a\"b"), Some("\"a\"\"b\"*".into()));
        assert_eq!(fts_query("NEAR(x y)"), Some("\"NEAR(x\" \"y)\"*".into()));
        for nothing in ["", "   ", "*", "\"", "(", "-", ":"] {
            assert_eq!(fts_query(nothing), None, "{nothing:?}");
        }
    }

    #[test]
    fn sections_follow_headings_outside_code() {
        let md = "intro\n# Título\ntexto\n```md\n# não é título\n```\n## Seção 2\nmais\n#hashtag\n";
        let s = sections(md);
        let heads: Vec<(&str, usize)> = s.iter().map(|(h, _, l)| (h.as_str(), *l)).collect();
        assert_eq!(heads, vec![("", 1), ("Título", 2), ("Seção 2", 7)]);
        assert!(s[1].1.contains("# não é título"));
        assert!(s[2].1.contains("#hashtag"));
    }

    #[test]
    fn only_markdown_is_indexed() {
        assert!(indexable("spec.md") && indexable("Notes.MD"));
        for no in [
            "gates.jsonl",
            "lessons.jsonl",
            "payload.json",
            "notes.txt",
            ".spec.md.quire-1-2.tmp",
        ] {
            assert!(!indexable(no), "{no}");
        }
    }
}
