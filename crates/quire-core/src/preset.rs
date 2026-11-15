//! Presets: how a project's spec folders are read (PST-01, PST-02).
//!
//! The core only knows the `generic` rules (any `.md` and `.jsonl`). Team conventions live in
//! presets; the built-in `exban` preset describes the format in `docs/spec-format.md`.
//! `config.toml` can override built-in presets or define new ones (see [`crate::config`]).

use regex::Regex;

pub const GENERIC: &str = "generic";
pub const EXBAN: &str = "exban";

/// Which files of a spec folder are primary (get their own tab and enter the content hash).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryFiles {
    /// Any file with one of these extensions (without the dot), tabs in alphabetical order.
    Extensions(Vec<String>),
    /// Exactly these file names, tabs in this order.
    Names(Vec<String>),
}

/// Grammar used to read `tasks.md` (TSK-01..04).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TasksGrammar {
    /// No structure: `tasks.md` is plain markdown.
    None,
    /// `### Fase N: título`, checkbox lines `- [x] T1: título (hash)`, detail sections.
    Phases,
}

#[derive(Debug, Clone)]
pub struct Preset {
    pub name: String,
    pub primary_files: PrimaryFiles,
    /// Files whose `**Chave:** valor` header lines become metadata chips (VIW-07).
    pub header_files: Vec<String>,
    pub tasks: TasksGrammar,
    /// Requirement ids, e.g. `<!-- TOK-01 -->`. `None` disables id extraction.
    pub id_pattern: Option<Regex>,
    /// Issue key at the start of the folder name; group 1 is the key (uppercased for display).
    pub issue_key_pattern: Option<Regex>,
}

impl PartialEq for Preset {
    fn eq(&self, other: &Self) -> bool {
        let re = |r: &Option<Regex>| r.as_ref().map(|r| r.as_str().to_owned());
        self.name == other.name
            && self.primary_files == other.primary_files
            && self.header_files == other.header_files
            && self.tasks == other.tasks
            && re(&self.id_pattern) == re(&other.id_pattern)
            && re(&self.issue_key_pattern) == re(&other.issue_key_pattern)
    }
}

/// Files of the `exban` preset, in tab order (VIW-01).
pub const EXBAN_FILES: [&str; 7] = [
    "spec.md",
    "design.md",
    "tasks.md",
    "summary.md",
    "validation.md",
    "gates.jsonl",
    "lessons.jsonl",
];

fn regex(src: &str) -> Option<Regex> {
    Regex::new(src).ok()
}

impl Preset {
    /// Any `.md` and `.jsonl`; no header, tasks grammar or ids.
    pub fn generic() -> Self {
        Self {
            name: GENERIC.into(),
            primary_files: PrimaryFiles::Extensions(vec!["md".into(), "jsonl".into()]),
            header_files: vec![],
            tasks: TasksGrammar::None,
            id_pattern: None,
            issue_key_pattern: None,
        }
    }

    /// The convention described in `docs/spec-format.md`.
    pub fn exban() -> Self {
        Self {
            name: EXBAN.into(),
            primary_files: PrimaryFiles::Names(
                EXBAN_FILES.iter().map(|s| (*s).to_owned()).collect(),
            ),
            header_files: vec!["spec.md".into(), "summary.md".into()],
            tasks: TasksGrammar::Phases,
            id_pattern: regex(r"[A-Z]{2,5}-\d{2}"),
            issue_key_pattern: regex(r"^([a-z]+-\d+)-"),
        }
    }

    pub fn is_primary(&self, file: &str) -> bool {
        match &self.primary_files {
            PrimaryFiles::Extensions(exts) => file
                .rsplit_once('.')
                .is_some_and(|(stem, ext)| !stem.is_empty() && exts.iter().any(|e| e == ext)),
            PrimaryFiles::Names(names) => names.iter().any(|n| n == file),
        }
    }

    /// Primary files present in `files`, in tab order (VIW-01).
    pub fn tabs(&self, files: &[String]) -> Vec<String> {
        match &self.primary_files {
            PrimaryFiles::Names(names) => names
                .iter()
                .filter(|n| files.contains(n))
                .cloned()
                .collect(),
            PrimaryFiles::Extensions(_) => {
                let mut tabs: Vec<String> = files
                    .iter()
                    .filter(|f| self.is_primary(f))
                    .cloned()
                    .collect();
                tabs.sort();
                tabs
            }
        }
    }

    /// Issue key from the folder name, uppercased (`acme-142-x` -> `ACME-142`).
    pub fn issue_key(&self, folder: &str) -> Option<String> {
        let caps = self.issue_key_pattern.as_ref()?.captures(folder)?;
        Some(caps.get(1)?.as_str().to_uppercase())
    }
}

/// Suggests the `exban` preset for a project read as `generic` whose folders look like it:
/// some spec folder holds at least `spec.md`, `tasks.md` and one of `summary.md` or
/// `gates.jsonl` (PST-01).
pub fn suggest(current: &str, folders: &[&[String]]) -> Option<&'static str> {
    if current != GENERIC {
        return None;
    }
    let has = |files: &[String], name: &str| files.iter().any(|f| f == name);
    folders
        .iter()
        .any(|f| {
            has(f, "spec.md")
                && has(f, "tasks.md")
                && (has(f, "summary.md") || has(f, "gates.jsonl"))
        })
        .then_some(EXBAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn generic_accepts_any_md_and_jsonl() {
        let p = Preset::generic();
        assert!(p.is_primary("README.md"));
        assert!(p.is_primary("log.jsonl"));
        assert!(!p.is_primary("payload.json"));
        assert!(!p.is_primary(".md"));
        assert_eq!(
            p.tabs(&files(&["z.md", "a.jsonl", "x.png", "b.md"])),
            ["a.jsonl", "b.md", "z.md"]
        );
        assert_eq!(p.issue_key("acme-142-x"), None);
    }

    #[test]
    fn exban_uses_names_in_tab_order() {
        let p = Preset::exban();
        assert!(p.is_primary("tasks.md"));
        assert!(!p.is_primary("notes.md"));
        let present = files(&["lessons.jsonl", "notes.md", "spec.md", "tasks.md"]);
        assert_eq!(p.tabs(&present), ["spec.md", "tasks.md", "lessons.jsonl"]);
        assert_eq!(
            p.issue_key("acme-142-service-tokens").as_deref(),
            Some("ACME-142")
        );
        assert_eq!(p.issue_key("no-key-here"), None);
        assert!(
            p.id_pattern
                .as_ref()
                .is_some_and(|r| r.is_match("<!-- TOK-01 -->"))
        );
    }

    #[test]
    fn suggests_exban_only_for_generic_projects_with_its_structure() {
        let full = files(&["spec.md", "tasks.md", "summary.md"]);
        let loose = files(&["README.md", "notes.md"]);
        assert_eq!(suggest(GENERIC, &[&loose, &full]), Some(EXBAN));
        assert_eq!(suggest(GENERIC, &[&loose]), None);
        assert_eq!(suggest(EXBAN, &[&full]), None);
    }
}
