//! Checking and unchecking a task of `tasks.md` (CHK-02, CHK-03, TD-18).
//!
//! Works on the raw bytes: only the character between the brackets of that one line changes,
//! so line endings, BOM, final newline and every other byte stay identical. The line must still
//! be the task the UI showed (same line number, id and state, recognized by the same grammar as
//! `parse::tasks`, which skips code blocks); the write goes through the pipeline of
//! `write` with the fingerprint of the `tasks.md` the view was built from. Nothing else is
//! written: no hash, date or note (P4); the commit hash stays the agent's job.

use std::path::Path;

use crate::fingerprint::Fingerprint;
use crate::write::{self, WriteError};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ToggleError {
    /// The line is no longer that task in that state; reload the view (CHK-03).
    #[error("task {0} is no longer on that line in that state")]
    Moved(String),
    #[error("tasks.md is not valid UTF-8")]
    NotUtf8,
    #[error(transparent)]
    Write(#[from] WriteError),
}

impl ToggleError {
    /// Stable code for the UI (DEB-02).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Moved(_) => "toggle_moved",
            Self::NotUtf8 => "toggle_not_utf8",
            Self::Write(e) => e.code(),
        }
    }
}

/// The bytes of `tasks.md` with task `id` on 1-based line `line` switched from `done` to the
/// other state. Checking writes `x`; unchecking turns `x` or `X` into a space.
pub fn toggle(bytes: &[u8], line: usize, id: &str, done: bool) -> Result<Vec<u8>, ToggleError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ToggleError::NotUtf8)?;
    let moved = || ToggleError::Moved(id.to_owned());
    let doc = crate::parse::tasks::tasks(text, None);
    let found = doc
        .phases
        .iter()
        .flat_map(|p| &p.tasks)
        .any(|t| t.id == id && t.line == line && t.done == done);
    if !found {
        return Err(moved());
    }
    // Byte range of that line, as `str::lines` sees it (LF separated, a trailing CR dropped).
    let start = if line <= 1 {
        0
    } else {
        bytes
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'\n')
            .nth(line - 2)
            .map(|(i, _)| i + 1)
            .ok_or_else(moved)?
    };
    let end = bytes[start..]
        .iter()
        .position(|b| *b == b'\n')
        .map_or(bytes.len(), |p| start + p);
    let line_text = std::str::from_utf8(&bytes[start..end])
        .map_err(|_| ToggleError::NotUtf8)?
        .trim_end_matches('\r');
    let mark = crate::parse::tasks::checkbox_offset(line_text).ok_or_else(moved)?;
    let at = start + mark;
    let new = match (bytes.get(at), done) {
        (Some(b' '), false) => b'x',
        (Some(b'x' | b'X'), true) => b' ',
        _ => return Err(moved()),
    };
    let mut out = bytes.to_vec();
    out[at] = new;
    Ok(out)
}

/// Toggles the task in the file at `path`, which must still be the `tasks.md` the view was
/// built from (`base`). Returns the new bytes and the fingerprint written.
pub fn toggle_file(
    path: &Path,
    line: usize,
    id: &str,
    done: bool,
    base: &Fingerprint,
) -> Result<(Vec<u8>, Fingerprint), ToggleError> {
    let bytes = std::fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            WriteError::Conflict { disk: None }
        } else {
            WriteError::Io {
                step: write::Step::Resolve,
                message: e.to_string(),
            }
        }
    })?;
    let disk = Fingerprint::of_bytes(&bytes);
    if disk != *base {
        return Err(WriteError::Conflict { disk: Some(disk) }.into());
    }
    let new = toggle(&bytes, line, id, done)?;
    let fp = write::save_over(path, &new, base)?;
    Ok((new, fp))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Tarefas\n\n### Fase 1: Base\n\n- [ ] T1: primeira (abc1234)\n- [x] T2: segunda\n- [X] T3: terceira\n\n```md\n- [ ] T9: dentro de bloco\n```\n\n  * [ ] T4 (nota): quarta\n";

    fn diff_positions(a: &[u8], b: &[u8]) -> Vec<usize> {
        assert_eq!(a.len(), b.len());
        (0..a.len()).filter(|i| a[*i] != b[*i]).collect()
    }

    #[test]
    fn only_the_checkbox_byte_changes_in_every_layout() {
        for crlf in [false, true] {
            for bom in [false, true] {
                for final_nl in [false, true] {
                    let mut text = DOC.to_owned();
                    if !final_nl {
                        text.pop();
                    }
                    if crlf {
                        text = text.replace('\n', "\r\n");
                    }
                    let mut bytes = if bom {
                        b"\xEF\xBB\xBF".to_vec()
                    } else {
                        vec![]
                    };
                    bytes.extend_from_slice(text.as_bytes());
                    let checked = toggle(&bytes, 5, "T1", false).unwrap();
                    let d = diff_positions(&bytes, &checked);
                    assert_eq!(d.len(), 1, "crlf={crlf} bom={bom} nl={final_nl}");
                    assert_eq!((bytes[d[0]], checked[d[0]]), (b' ', b'x'));
                    // And back: the original bytes, exactly.
                    assert_eq!(toggle(&checked, 5, "T1", true).unwrap(), bytes);
                    // The indented `*` task and the last line without newline.
                    let t4 = toggle(&bytes, 13, "T4", false).unwrap();
                    assert_eq!(diff_positions(&bytes, &t4).len(), 1);
                }
            }
        }
    }

    #[test]
    fn uppercase_x_unchecks_and_checking_writes_lowercase() {
        let bytes = DOC.as_bytes();
        let out = toggle(bytes, 7, "T3", true).unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("- [ ] T3: terceira")
        );
        let out = toggle(bytes, 6, "T2", true).unwrap();
        let back = toggle(&out, 6, "T2", false).unwrap();
        assert!(
            String::from_utf8(back)
                .unwrap()
                .contains("- [x] T2: segunda")
        );
        // Nothing is appended: no hash, no note (P4).
        let checked = String::from_utf8(toggle(bytes, 5, "T1", false).unwrap()).unwrap();
        assert!(checked.contains("- [x] T1: primeira (abc1234)\n"));
        assert_eq!(checked.len(), DOC.len());
    }

    #[test]
    fn a_line_that_is_no_longer_that_task_is_refused() {
        let bytes = DOC.as_bytes();
        let moved = |r: Result<Vec<u8>, ToggleError>| matches!(r, Err(ToggleError::Moved(_)));
        assert!(moved(toggle(bytes, 6, "T1", false)), "other line");
        assert!(moved(toggle(bytes, 5, "T2", false)), "other id");
        assert!(moved(toggle(bytes, 5, "T1", true)), "other state");
        assert!(moved(toggle(bytes, 10, "T9", false)), "inside a code block");
        assert!(moved(toggle(bytes, 99, "T1", false)), "past the end");
        assert!(moved(toggle(bytes, 0, "T1", false)), "line zero");
        assert_eq!(
            toggle(b"\xff\xfe", 1, "T1", false),
            Err(ToggleError::NotUtf8)
        );
    }

    #[test]
    fn the_file_must_still_be_the_one_the_view_was_built_from() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("tasks.md");
        std::fs::write(&f, DOC).unwrap();
        let base = Fingerprint::of(&f).unwrap().unwrap();
        // The agent edits tasks.md after the view loaded.
        let agent = DOC.replace("segunda", "segunda (e5f6a7b)");
        std::fs::write(&f, &agent).unwrap();
        let r = toggle_file(&f, 5, "T1", false, &base);
        assert!(
            matches!(r, Err(ToggleError::Write(WriteError::Conflict { .. }))),
            "{r:?}"
        );
        assert_eq!(std::fs::read_to_string(&f).unwrap(), agent);
        // With the right base, the write happens and returns the next base.
        let base = Fingerprint::of(&f).unwrap().unwrap();
        let (bytes, fp) = toggle_file(&f, 5, "T1", false, &base).unwrap();
        assert_eq!(std::fs::read(&f).unwrap(), bytes);
        assert_eq!(Fingerprint::of(&f).unwrap(), Some(fp));
        assert!(
            !std::fs::read_dir(d.path()).unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
    }

    #[test]
    fn the_v0_1_tasks_md_of_this_repository_in_a_temp_copy() {
        // Real case, read-only: the repository file is read, the copy in a temp dir is edited.
        let real =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.specs/features/quire-v0-1/tasks.md");
        let original = std::fs::read(&real).unwrap();
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("tasks.md");
        std::fs::write(&f, &original).unwrap();
        let doc = crate::parse::tasks::tasks(std::str::from_utf8(&original).unwrap(), None);
        let task = doc
            .phases
            .iter()
            .flat_map(|p| &p.tasks)
            .find(|t| !t.done)
            .unwrap()
            .clone();
        let base = Fingerprint::of(&f).unwrap().unwrap();
        let (bytes, _) = toggle_file(&f, task.line, &task.id, false, &base).unwrap();
        assert_eq!(diff_positions(&original, &bytes).len(), 1, "{}", task.id);
        let after = crate::parse::tasks::tasks(std::str::from_utf8(&bytes).unwrap(), None);
        let t = after.task(&task.id).unwrap();
        assert!(t.done);
        assert_eq!(
            (t.title.as_str(), &t.commits, &t.note),
            (task.title.as_str(), &task.commits, &task.note)
        );
        assert_eq!(
            std::fs::read(&real).unwrap(),
            original,
            "the repository file is untouched"
        );
    }
}
