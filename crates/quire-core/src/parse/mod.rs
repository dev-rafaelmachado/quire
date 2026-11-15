//! Pure, tolerant parsers for spec files (PST-03): input outside the expected format yields
//! empty results, never an error. Line numbers are 1-based.

pub mod header;
pub mod ids;
pub mod summary;
pub mod tasks;

pub use header::{HeaderField, header};
pub use ids::{IdRef, ids};
pub use summary::{ProgressRow, summary_progress};
pub use tasks::{DetailField, Phase, PhaseState, Task, TaskDetail, TasksDoc, tasks};

/// Tracks fenced code blocks (``` or ~~~) while iterating lines.
#[derive(Debug, Default)]
pub(crate) struct Fence {
    open: Option<(char, usize)>,
}

impl Fence {
    /// Feeds one line; returns `true` when the line is a fence delimiter or inside a block.
    pub(crate) fn step(&mut self, line: &str) -> bool {
        let t = line.trim_start();
        let marker = t.chars().next().filter(|c| *c == '`' || *c == '~');
        let run = marker.map_or(0, |m| t.chars().take_while(|c| *c == m).count());
        match (self.open, marker) {
            (None, Some(m)) if run >= 3 => {
                self.open = Some((m, run));
                true
            }
            (Some((m, n)), Some(c))
                if c == m && run >= n && t[run * c.len_utf8()..].trim().is_empty() =>
            {
                self.open = None;
                true
            }
            (Some(_), _) => true,
            _ => false,
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

/// Splits `**Name:** value` or `**Name**: value` into (name, value).
pub(crate) fn bold_field(line: &str) -> Option<(&str, &str)> {
    let rest = line.trim_start().strip_prefix("**")?;
    let end = rest.find("**")?;
    let (name, after) = (&rest[..end], &rest[end + 2..]);
    if let Some(name) = name.strip_suffix(':') {
        return Some((name.trim(), after.trim()));
    }
    let value = after.strip_prefix(':')?;
    Some((name.trim(), value.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fence_tracking() {
        let mut f = Fence::default();
        let lines = [
            "a", "```rust", "x", "~~~", "```", "b", "~~~~", "```", "~~~~",
        ];
        let inside: Vec<bool> = lines.iter().map(|l| f.step(l)).collect();
        assert_eq!(
            inside,
            [false, true, true, true, true, false, true, true, true]
        );
        assert!(!f.is_open());
    }

    #[test]
    fn bold_fields() {
        assert_eq!(bold_field("**Status:** ok"), Some(("Status", "ok")));
        assert_eq!(bold_field("**What**: do it"), Some(("What", "do it")));
        assert_eq!(
            bold_field("**Pronto quando**:"),
            Some(("Pronto quando", ""))
        );
        assert_eq!(bold_field("**bold** text"), None);
        assert_eq!(bold_field("plain"), None);
    }
}
