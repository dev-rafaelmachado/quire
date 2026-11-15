//! User configuration (`config.toml` in the system config directory, A2).
//!
//! ```toml
//! scan_paths = ["~/projects"]
//! poll_interval_ms = 3000
//!
//! [projects."acme_gateway"]
//! preset = "exban"
//! alias = "gateway"
//! issue_url_base = "https://tracker.example/browse/"
//!
//! [presets.exban]              # override a built-in preset, or define a new one
//! extends = "exban"            # base preset (defaults to the same name if built in, else generic)
//! primary_files = ["spec.md", "tasks.md"]
//! ```
//!
//! Unknown keys are rejected so typos surface. Every error carries the line and column.

use std::collections::BTreeMap;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Deserialize;
use toml::Spanned;

use crate::error::ConfigError;
use crate::preset::{self, Preset, PrimaryFiles, TasksGrammar};

pub const DEFAULT_POLL_INTERVAL_MS: u64 = 3000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectConfig {
    pub preset: Option<String>,
    pub alias: Option<String>,
    pub issue_url_base: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub scan_paths: Vec<PathBuf>,
    pub poll_interval_ms: u64,
    /// Keyed by project name (the `origin` name or folder name, DUP-01).
    pub projects: BTreeMap<String, ProjectConfig>,
    presets: BTreeMap<String, Preset>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scan_paths: vec![],
            poll_interval_ms: DEFAULT_POLL_INTERVAL_MS,
            projects: BTreeMap::new(),
            presets: builtin_presets(),
        }
    }
}

fn builtin_presets() -> BTreeMap<String, Preset> {
    [Preset::generic(), Preset::exban()]
        .into_iter()
        .map(|p| (p.name.clone(), p))
        .collect()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfig {
    #[serde(default)]
    scan_paths: Vec<String>,
    poll_interval_ms: Option<u64>,
    #[serde(default)]
    projects: BTreeMap<String, RawProject>,
    #[serde(default)]
    presets: BTreeMap<String, RawPreset>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProject {
    preset: Option<Spanned<String>>,
    alias: Option<String>,
    issue_url_base: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPreset {
    extends: Option<Spanned<String>>,
    primary_files: Option<Vec<String>>,
    primary_extensions: Option<Vec<String>>,
    header_files: Option<Vec<String>>,
    tasks: Option<Spanned<String>>,
    id_pattern: Option<Spanned<String>>,
    issue_key_pattern: Option<Spanned<String>>,
}

/// 1-based line and column of a byte offset.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
    (line, column)
}

fn invalid(text: &str, span: Range<usize>, message: String) -> ConfigError {
    let (line, column) = position(text, span.start);
    ConfigError::Invalid {
        line,
        column,
        message,
    }
}

fn compile(
    text: &str,
    field: &Option<Spanned<String>>,
    name: &str,
) -> Result<Option<Option<Regex>>, ConfigError> {
    let Some(src) = field else { return Ok(None) };
    if src.get_ref().is_empty() {
        return Ok(Some(None)); // "" disables the pattern
    }
    Regex::new(src.get_ref())
        .map(|r| Some(Some(r)))
        .map_err(|e| invalid(text, src.span(), format!("invalid {name}: {e}")))
}

impl Config {
    /// Parses `config.toml` text. Paths starting with `~/` are expanded.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let raw: RawConfig = toml::from_str(text).map_err(|e| {
            let (line, column) = e.span().map_or((0, 0), |s| position(text, s.start));
            ConfigError::Parse {
                line,
                column,
                message: e.message().to_owned(),
            }
        })?;
        let mut presets = builtin_presets();
        for (name, rp) in &raw.presets {
            let base_name = match &rp.extends {
                Some(b) => {
                    if !presets.contains_key(b.get_ref()) {
                        return Err(invalid(
                            text,
                            b.span(),
                            format!("unknown base preset \"{}\"", b.get_ref()),
                        ));
                    }
                    b.get_ref().clone()
                }
                None if presets.contains_key(name) => name.clone(),
                None => preset::GENERIC.to_owned(),
            };
            let mut p = presets
                .get(&base_name)
                .cloned()
                .unwrap_or_else(Preset::generic);
            p.name = name.clone();
            if let Some(files) = &rp.primary_files {
                p.primary_files = PrimaryFiles::Names(files.clone());
            }
            if let Some(exts) = &rp.primary_extensions {
                p.primary_files = PrimaryFiles::Extensions(
                    exts.iter()
                        .map(|e| e.trim_start_matches('.').to_owned())
                        .collect(),
                );
            }
            if let Some(h) = &rp.header_files {
                p.header_files = h.clone();
            }
            if let Some(t) = &rp.tasks {
                p.tasks = match t.get_ref().as_str() {
                    "phases" => TasksGrammar::Phases,
                    "none" => TasksGrammar::None,
                    other => {
                        return Err(invalid(
                            text,
                            t.span(),
                            format!("tasks must be \"phases\" or \"none\", got \"{other}\""),
                        ));
                    }
                };
            }
            if let Some(r) = compile(text, &rp.id_pattern, "id_pattern")? {
                p.id_pattern = r;
            }
            if let Some(r) = compile(text, &rp.issue_key_pattern, "issue_key_pattern")? {
                p.issue_key_pattern = r;
            }
            presets.insert(name.clone(), p);
        }
        let mut projects = BTreeMap::new();
        for (name, rp) in raw.projects {
            if let Some(p) = rp
                .preset
                .as_ref()
                .filter(|p| !presets.contains_key(p.get_ref()))
            {
                return Err(invalid(
                    text,
                    p.span(),
                    format!("unknown preset \"{}\" for project \"{name}\"", p.get_ref()),
                ));
            }
            projects.insert(
                name,
                ProjectConfig {
                    preset: rp.preset.map(Spanned::into_inner),
                    alias: rp.alias,
                    issue_url_base: rp.issue_url_base,
                },
            );
        }
        Ok(Self {
            scan_paths: raw.scan_paths.iter().map(|p| expand_home(p)).collect(),
            poll_interval_ms: raw.poll_interval_ms.unwrap_or(DEFAULT_POLL_INTERVAL_MS),
            projects,
            presets,
        })
    }

    /// Loads the file; a missing file means the default configuration. Relative scan paths
    /// are resolved against the directory of the file.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        match fs::read_to_string(path) {
            Ok(text) => {
                let mut config = Self::parse(&text)?;
                if let Some(base) = path.parent() {
                    for p in &mut config.scan_paths {
                        if p.is_relative() {
                            *p = base.join(&*p);
                        }
                    }
                }
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ConfigError::Io {
                path: path.to_path_buf(),
                message: e.to_string(),
            }),
        }
    }

    /// `<config dir>/quire/config.toml` (A2).
    pub fn default_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("quire").join("config.toml"))
    }

    /// Preset of a project: configured one, else `generic` (PST-01).
    pub fn preset_for(&self, project_name: &str) -> &Preset {
        self.projects
            .get(project_name)
            .and_then(|p| p.preset.as_ref())
            .and_then(|name| self.presets.get(name))
            .or_else(|| self.presets.get(preset::GENERIC))
            .unwrap_or_else(|| unreachable_generic())
    }

    pub fn has_explicit_preset(&self, project_name: &str) -> bool {
        self.projects
            .get(project_name)
            .is_some_and(|p| p.preset.is_some())
    }

    pub fn alias_for(&self, project_name: &str) -> Option<&str> {
        self.projects
            .get(project_name)
            .and_then(|p| p.alias.as_deref())
    }

    pub fn preset(&self, name: &str) -> Option<&Preset> {
        self.presets.get(name)
    }

    pub fn preset_names(&self) -> impl Iterator<Item = &str> {
        self.presets.keys().map(String::as_str)
    }

    /// Scan filter: a folder is a spec candidate when any known preset sees a primary file in
    /// it. Each project then keeps only folders that match its own preset (DSC-02).
    pub fn scan_filter(&self) -> impl Fn(&[String]) -> bool + Sync + '_ {
        move |files: &[String]| {
            self.presets
                .values()
                .any(|p| files.iter().any(|f| p.is_primary(f)))
        }
    }
}

/// `generic` is always registered; this only exists to avoid an `unwrap` on that invariant.
fn unreachable_generic() -> &'static Preset {
    static GENERIC: std::sync::OnceLock<Preset> = std::sync::OnceLock::new();
    GENERIC.get_or_init(Preset::generic)
}

fn expand_home(p: &str) -> PathBuf {
    match (
        p.strip_prefix("~/").or_else(|| p.strip_prefix("~\\")),
        std::env::home_dir(),
    ) {
        (Some(rest), Some(home)) => home.join(rest),
        _ if p == "~" => std::env::home_dir().unwrap_or_else(|| PathBuf::from(p)),
        _ => PathBuf::from(p),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_config_is_the_default() {
        let c = Config::parse("").unwrap();
        assert_eq!(c, Config::default());
        assert_eq!(c.poll_interval_ms, 3000);
        assert_eq!(c.preset_for("anything").name, "generic");
    }

    #[test]
    fn project_preset_and_alias() {
        let c = Config::parse(
            r#"
scan_paths = ["~/projects", "/abs"]
poll_interval_ms = 1500

[projects."acme_gateway"]
preset = "exban"
alias = "gateway"
issue_url_base = "https://tracker.example/browse/"
"#,
        )
        .unwrap();
        assert_eq!(c.preset_for("acme_gateway").name, "exban");
        assert_eq!(c.preset_for("acme_billing").name, "generic");
        assert_eq!(c.alias_for("acme_gateway"), Some("gateway"));
        assert_eq!(c.poll_interval_ms, 1500);
        assert_eq!(c.scan_paths[1], PathBuf::from("/abs"));
        if let Some(home) = std::env::home_dir() {
            assert_eq!(c.scan_paths[0], home.join("projects"));
        }
    }

    #[test]
    fn switching_the_preset_changes_primary_files() {
        let generic = Config::parse("").unwrap();
        let exban = Config::parse("[projects.p]\npreset = \"exban\"\n").unwrap();
        assert!(generic.preset_for("p").is_primary("notes.md"));
        assert!(!exban.preset_for("p").is_primary("notes.md"));
        assert!(exban.preset_for("p").is_primary("spec.md"));
    }

    #[test]
    fn presets_can_be_overridden_and_defined() {
        let c = Config::parse(
            r#"
[presets.exban]
primary_files = ["spec.md", "tasks.md"]

[presets.team]
extends = "exban"
issue_key_pattern = '^(proj-\d+)-'
id_pattern = ""

[presets.docs]
primary_extensions = [".md", "txt"]
tasks = "none"

[projects.a]
preset = "team"
"#,
        )
        .unwrap();
        assert!(!c.preset("exban").unwrap().is_primary("design.md"));
        let team = c.preset_for("a");
        assert_eq!(team.name, "team");
        assert!(team.is_primary("spec.md") && !team.is_primary("design.md"));
        assert_eq!(team.issue_key("proj-7-x").as_deref(), Some("PROJ-7"));
        assert!(team.id_pattern.is_none());
        assert!(c.preset("docs").unwrap().is_primary("a.txt"));
        assert!(c.scan_filter()(&["a.txt".to_owned()]));
    }

    #[test]
    fn syntax_errors_carry_the_line() {
        let err = Config::parse("scan_paths = [\"a\"]\npoll_interval_ms = = 3\n").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { line: 2, .. }), "{err:?}");
    }

    #[test]
    fn unknown_keys_and_wrong_types_are_rejected_with_line() {
        let err = Config::parse("scan_paths = []\nscan_path = []\n").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { line: 2, .. }), "{err:?}");
        let err = Config::parse("poll_interval_ms = \"fast\"\n").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { line: 1, .. }), "{err:?}");
    }

    #[test]
    fn semantic_errors_carry_the_line() {
        let err = Config::parse("\n[projects.a]\npreset = \"nope\"\n").unwrap_err();
        assert!(
            matches!(
                err,
                ConfigError::Invalid {
                    line: 3,
                    column: 10,
                    ..
                }
            ),
            "{err:?}"
        );
        let err = Config::parse("[presets.x]\nid_pattern = \"(\"\n").unwrap_err();
        assert!(
            matches!(err, ConfigError::Invalid { line: 2, .. }),
            "{err:?}"
        );
        let err = Config::parse("[presets.x]\ntasks = \"tree\"\n").unwrap_err();
        assert!(
            matches!(err, ConfigError::Invalid { line: 2, .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("line 2"), "{err}");
    }

    #[test]
    fn missing_file_loads_defaults() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(
            Config::load(&tmp.path().join("none.toml")).unwrap(),
            Config::default()
        );
        let rel = tmp.path().join("rel.toml");
        fs::write(&rel, "scan_paths = [\"workspace/projects\", \"/abs\"]\n").unwrap();
        let c = Config::load(&rel).unwrap();
        assert_eq!(
            c.scan_paths,
            [tmp.path().join("workspace/projects"), PathBuf::from("/abs")]
        );
        let bad = tmp.path().join("bad.toml");
        fs::write(&bad, "x = 1\n").unwrap();
        assert!(Config::load(&bad).is_err());
    }
}
