//! Progress table of `summary.md`, between `<!-- progress:start -->` and `<!-- progress:end -->`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressRow {
    /// First column, e.g. `1: Fundações`.
    pub phase: String,
    /// Middle column(s) joined, e.g. the repository; empty for two-column tables.
    pub repo: String,
    pub done: u32,
    pub total: u32,
    pub line: usize,
}

/// Rows of the progress table. Header, separator and rows without `x/y` are skipped.
pub fn summary_progress(md: &str) -> Vec<ProgressRow> {
    let mut out = Vec::new();
    let mut inside = false;
    for (i, line) in md.lines().enumerate() {
        let t = line.trim();
        if t.contains("<!-- progress:start -->") {
            inside = true;
            continue;
        }
        if t.contains("<!-- progress:end -->") {
            break;
        }
        if !inside || !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        let (Some(first), Some(last)) = (cells.first(), cells.last()) else {
            continue;
        };
        let Some((done, total)) = fraction(last) else {
            continue;
        };
        let repo = if cells.len() > 2 {
            cells[1..cells.len() - 1].join(" | ")
        } else {
            String::new()
        };
        out.push(ProgressRow {
            phase: (*first).to_owned(),
            repo,
            done,
            total,
            line: i + 1,
        });
    }
    out
}

/// First `x/y` in a cell (`3/3 ✓` -> (3, 3)).
fn fraction(cell: &str) -> Option<(u32, u32)> {
    cell.split_whitespace().find_map(|w| {
        let (a, b) = w.split_once('/')?;
        Some((a.parse().ok()?, b.parse().ok()?))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rows_between_markers() {
        let md = "x\n<!-- progress:start -->\n| Fase | Repositório | Tarefas |\n|---|---|---|\n| 1: Fundações | repo | 5/5 ✓ |\n| 2: Resto | repo | 0/3 |\n<!-- progress:end -->\n| 3: fora | r | 1/1 |";
        let rows = summary_progress(md);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            (
                rows[0].phase.as_str(),
                rows[0].repo.as_str(),
                rows[0].done,
                rows[0].total
            ),
            ("1: Fundações", "repo", 5, 5)
        );
        assert_eq!(rows[1].line, 6);
    }

    #[test]
    fn out_of_format_is_empty() {
        assert!(summary_progress("").is_empty());
        assert!(summary_progress("| a | 1/2 |").is_empty());
        assert!(summary_progress("<!-- progress:start -->\n| a | b |\n| c | x/y |").is_empty());
    }
}
