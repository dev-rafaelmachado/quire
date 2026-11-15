//! SQLite index: a disposable cache of what the scan found (TD-06, IDX-01..03).
//!
//! [`Index::list`] answers from the cache alone, so the sidebar can show ~300 specs at startup
//! before any disk work (IDX-03). [`Index::reconcile`] scans, compares `(name, size, mtime)` of
//! every file with the cache and only reads and parses instances whose files changed (IDX-02).
//! Deleting the database loses nothing: the next reconcile rebuilds it (IDX-01). A database
//! with another schema version, or that SQLite cannot read, is deleted and recreated.
//!
//! The database runs in WAL mode, so a second connection can [`Index::list`] while a reconcile
//! writes; writes are committed in batches so the list fills in progressively (DSC-04).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::config::Config;
use crate::error::{IndexError, ScanError};
use crate::git::GitResolver;
use crate::model::{self, Instance, InstanceKind};
use crate::parse;
use crate::preset::{self, Preset, TasksGrammar};
use crate::scan::{Discovery, ScanEvent, scan_with_worktrees};

const SCHEMA_VERSION: i64 = 1;
const BATCH: usize = 50;

const SCHEMA: &str = "
CREATE TABLE projects (
  id INTEGER PRIMARY KEY,
  git_common_dir TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  alias TEXT,
  preset TEXT NOT NULL,
  suggested_preset TEXT,
  git_warning TEXT
);
CREATE TABLE specs (
  id INTEGER PRIMARY KEY,
  project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  folder TEXT NOT NULL,
  issue_key TEXT,
  UNIQUE(project_id, folder)
);
CREATE TABLE instances (
  id INTEGER PRIMARY KEY,
  spec_id INTEGER NOT NULL REFERENCES specs(id) ON DELETE CASCADE,
  path TEXT NOT NULL UNIQUE,
  repo_root TEXT NOT NULL,
  branch TEXT,
  kind TEXT NOT NULL CHECK(kind IN ('worktree', 'checkout')),
  mtime INTEGER,
  content_hash TEXT NOT NULL
);
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  instance_id INTEGER NOT NULL REFERENCES instances(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  size INTEGER NOT NULL,
  mtime INTEGER,
  UNIQUE(instance_id, name)
);
-- Values derived from the files of each instance, so the list never opens files.
CREATE TABLE spec_meta (
  instance_id INTEGER PRIMARY KEY REFERENCES instances(id) ON DELETE CASCADE,
  title TEXT,
  status TEXT,
  issue_url TEXT,
  tasks_done INTEGER,
  tasks_total INTEGER
);
";

// ------------------------------------------------------------------ public types

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct InstanceSummary {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub id: i64,
    pub path: PathBuf,
    pub repo_root: PathBuf,
    pub branch: Option<String>,
    pub kind: InstanceKind,
    #[cfg_attr(feature = "specta", specta(type = Option<f64>))]
    pub mtime_ms: Option<i64>,
    /// Same primary-file content as the spec's default instance.
    pub same_as_default: bool,
    /// Hash of the primary files; equal hashes mean identical copies (DUP-05).
    pub content_hash: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub issue_url: Option<String>,
    pub progress: Option<(u32, u32)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SpecSummary {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub id: i64,
    pub folder: String,
    pub issue_key: Option<String>,
    /// Sorted by path.
    pub instances: Vec<InstanceSummary>,
    /// Index into `instances` (DUP-04).
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub default_index: usize,
    /// "N versões" badge (DUP-03).
    #[cfg_attr(feature = "specta", specta(type = Option<u32>))]
    pub versions_badge: Option<usize>,
}

impl SpecSummary {
    pub fn default_instance(&self) -> Option<&InstanceSummary> {
        self.instances.get(self.default_index)
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ProjectSummary {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub id: i64,
    pub key: PathBuf,
    pub name: String,
    pub alias: Option<String>,
    pub preset: String,
    pub suggested_preset: Option<String>,
    /// Set when git failed for this project and the name is a fallback (A4).
    pub git_warning: Option<String>,
    /// Sorted by folder.
    pub specs: Vec<SpecSummary>,
}

impl ProjectSummary {
    pub fn display_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
}

/// Progress of [`Index::reconcile`], for the UI (DSC-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEvent {
    /// The scan found one more spec folder.
    Found {
        count: usize,
    },
    PathError {
        path: PathBuf,
        error: ScanError,
    },
    /// Folders processed so far; emitted after each committed batch (the list can be reloaded).
    Progress {
        processed: usize,
        total: usize,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ReconcileReport {
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub found: usize,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub path_errors: usize,
    /// Instances read and parsed again because a file changed (or was new).
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub reprocessed: usize,
    /// Instances skipped because every file kept its size and mtime.
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub unchanged: usize,
    #[cfg_attr(feature = "specta", specta(type = u32))]
    pub removed: usize,
}

// ------------------------------------------------------------------ index

pub struct Index {
    conn: Connection,
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index").finish_non_exhaustive()
    }
}

fn sql(e: rusqlite::Error) -> IndexError {
    IndexError::Sqlite(e.to_string())
}

impl Index {
    /// `<data dir>/quire/index.sqlite` (A2).
    pub fn default_path() -> Option<PathBuf> {
        dirs::data_dir().map(|d| d.join("quire").join("index.sqlite"))
    }

    /// Opens or creates the cache. An unreadable database or another schema version is
    /// deleted and rebuilt.
    pub fn open(path: &Path) -> Result<Self, IndexError> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| IndexError::Io {
                path: dir.to_path_buf(),
                message: e.to_string(),
            })?;
        }
        match Self::try_open(path) {
            Ok(index) => Ok(index),
            Err(_) => {
                for suffix in ["", "-wal", "-shm"] {
                    let mut p = path.as_os_str().to_owned();
                    p.push(suffix);
                    let _ = fs::remove_file(PathBuf::from(p));
                }
                Self::try_open(path)
            }
        }
    }

    pub fn open_in_memory() -> Result<Self, IndexError> {
        let conn = Connection::open_in_memory().map_err(sql)?;
        Self::init(conn)
    }

    fn try_open(path: &Path) -> Result<Self, IndexError> {
        let conn = Connection::open(path).map_err(sql)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(sql)?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, IndexError> {
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(sql)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(sql)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(sql)?;
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(sql)?;
        if version != SCHEMA_VERSION {
            let tables: i64 = conn
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type = 'table'",
                    [],
                    |r| r.get(0),
                )
                .map_err(sql)?;
            if tables > 0 {
                return Err(IndexError::SchemaMismatch {
                    found: version,
                    expected: SCHEMA_VERSION,
                });
            }
            conn.execute_batch(SCHEMA).map_err(sql)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(sql)?;
        }
        Ok(Self { conn })
    }

    /// Projects, specs and instances from the cache only (IDX-03).
    pub fn list(&self) -> Result<Vec<ProjectSummary>, IndexError> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT p.id, p.git_common_dir, p.name, p.alias, p.preset, p.suggested_preset, p.git_warning,
                        s.id, s.folder, s.issue_key,
                        i.id, i.path, i.repo_root, i.branch, i.kind, i.mtime, i.content_hash,
                        m.title, m.status, m.issue_url, m.tasks_done, m.tasks_total
                 FROM projects p
                 JOIN specs s ON s.project_id = p.id
                 JOIN instances i ON i.spec_id = s.id
                 LEFT JOIN spec_meta m ON m.instance_id = i.id
                 ORDER BY p.id, s.folder, i.path",
            )
            .map_err(sql)?;
        let mut projects: Vec<ProjectSummary> = Vec::new();
        let mut hashes: HashMap<i64, Vec<String>> = HashMap::new();
        let mut rows = stmt.query([]).map_err(sql)?;
        while let Some(r) = rows.next().map_err(sql)? {
            let project_id: i64 = r.get(0).map_err(sql)?;
            if projects.last().is_none_or(|p| p.id != project_id) {
                projects.push(ProjectSummary {
                    id: project_id,
                    key: PathBuf::from(r.get::<_, String>(1).map_err(sql)?),
                    name: r.get(2).map_err(sql)?,
                    alias: r.get(3).map_err(sql)?,
                    preset: r.get(4).map_err(sql)?,
                    suggested_preset: r.get(5).map_err(sql)?,
                    git_warning: r.get(6).map_err(sql)?,
                    specs: vec![],
                });
            }
            let Some(project) = projects.last_mut() else {
                continue;
            };
            let spec_id: i64 = r.get(7).map_err(sql)?;
            if project.specs.last().is_none_or(|s| s.id != spec_id) {
                project.specs.push(SpecSummary {
                    id: spec_id,
                    folder: r.get(8).map_err(sql)?,
                    issue_key: r.get(9).map_err(sql)?,
                    instances: vec![],
                    default_index: 0,
                    versions_badge: None,
                });
            }
            let Some(spec) = project.specs.last_mut() else {
                continue;
            };
            let kind: String = r.get(14).map_err(sql)?;
            let done: Option<u32> = r.get(20).map_err(sql)?;
            let total: Option<u32> = r.get(21).map_err(sql)?;
            spec.instances.push(InstanceSummary {
                id: r.get(10).map_err(sql)?,
                path: PathBuf::from(r.get::<_, String>(11).map_err(sql)?),
                repo_root: PathBuf::from(r.get::<_, String>(12).map_err(sql)?),
                branch: r.get(13).map_err(sql)?,
                kind: if kind == "worktree" {
                    InstanceKind::Worktree
                } else {
                    InstanceKind::Checkout
                },
                mtime_ms: r.get(15).map_err(sql)?,
                same_as_default: true,
                content_hash: r.get(16).map_err(sql)?,
                title: r.get(17).map_err(sql)?,
                status: r.get(18).map_err(sql)?,
                issue_url: r.get(19).map_err(sql)?,
                progress: done.zip(total),
            });
            hashes
                .entry(spec_id)
                .or_default()
                .push(r.get(16).map_err(sql)?);
        }
        for spec in projects.iter_mut().flat_map(|p| p.specs.iter_mut()) {
            let hs = hashes.remove(&spec.id).unwrap_or_default();
            let as_model: Vec<Instance> = spec
                .instances
                .iter()
                .zip(&hs)
                .map(|(i, h)| Instance {
                    spec_dir: i.path.clone(),
                    repo_root: i.repo_root.clone(),
                    branch: None,
                    kind: i.kind,
                    mtime_ms: i.mtime_ms,
                    content_hash: h.clone(),
                    files: vec![],
                })
                .collect();
            spec.default_index = model::default_index(&as_model);
            let default_hash = hs.get(spec.default_index).cloned().unwrap_or_default();
            for (inst, h) in spec.instances.iter_mut().zip(&hs) {
                inst.same_as_default = *h == default_hash;
            }
            let diverges = hs.windows(2).any(|w| w[0] != w[1]);
            spec.versions_badge = diverges.then_some(spec.instances.len());
        }
        projects.sort_by(|a, b| {
            a.display_name()
                .cmp(b.display_name())
                .then_with(|| a.key.cmp(&b.key))
        });
        Ok(projects)
    }

    /// Where an instance lives and how its project reads it.
    pub fn instance_info(&self, instance_id: i64) -> Result<Option<InstanceInfo>, IndexError> {
        let row: Option<(i64, i64, String, String, String, String)> = self
            .conn
            .query_row(
                "SELECT i.spec_id, p.id, i.path, i.repo_root, p.name, p.preset
                 FROM instances i JOIN specs s ON s.id = i.spec_id JOIN projects p ON p.id = s.project_id
                 WHERE i.id = ?1",
                [instance_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .optional()
            .map_err(sql)?;
        let Some((spec_id, project_id, path, repo_root, project_name, preset)) = row else {
            return Ok(None);
        };
        Ok(Some(InstanceInfo {
            id: instance_id,
            spec_id,
            project_id,
            path: PathBuf::from(path),
            repo_root: PathBuf::from(repo_root),
            project_name,
            preset,
            files: self.files(instance_id)?,
        }))
    }

    /// Files recorded for an instance: `(name, size, mtime_ms)`, sorted by name.
    pub fn files(&self, instance_id: i64) -> Result<Vec<(String, u64, Option<i64>)>, IndexError> {
        let mut stmt = self
            .conn
            .prepare_cached(
                "SELECT name, size, mtime FROM files WHERE instance_id = ?1 ORDER BY name",
            )
            .map_err(sql)?;
        let rows = stmt
            .query_map([instance_id], |r| {
                Ok((r.get(0)?, r.get::<_, i64>(1)? as u64, r.get(2)?))
            })
            .map_err(sql)?;
        rows.collect::<Result<_, _>>().map_err(sql)
    }

    /// Scans `paths` (and worktrees outside them) and brings the cache in line with the disk.
    pub fn reconcile(
        &mut self,
        paths: &[PathBuf],
        config: &Config,
        git: &mut GitResolver,
        mut on_event: impl FnMut(IndexEvent),
    ) -> Result<ReconcileReport, IndexError> {
        let mut report = ReconcileReport::default();
        let mut discoveries: Vec<Discovery> = Vec::new();
        let filter = config.scan_filter();
        scan_with_worktrees(paths, &filter, git, |e| match e {
            ScanEvent::Found(d) => {
                discoveries.push(d);
                on_event(IndexEvent::Found {
                    count: discoveries.len(),
                });
            }
            ScanEvent::PathError { path, error } => {
                report.path_errors += 1;
                on_event(IndexEvent::PathError { path, error });
            }
        });
        report.found = discoveries.len();

        let mut projects: BTreeMap<PathBuf, ProjectState> = BTreeMap::new();
        let mut seen: HashSet<i64> = HashSet::new();
        self.conn.execute_batch("BEGIN").map_err(sql)?;
        for (n, d) in discoveries.iter().enumerate() {
            match self.process(d, config, git, &mut projects) {
                Ok(Some((id, reprocessed))) => {
                    seen.insert(id);
                    if reprocessed {
                        report.reprocessed += 1;
                    } else {
                        report.unchanged += 1;
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = self.conn.execute_batch("ROLLBACK");
                    return Err(e);
                }
            }
            if (n + 1) % BATCH == 0 {
                self.conn.execute_batch("COMMIT; BEGIN").map_err(sql)?;
                on_event(IndexEvent::Progress {
                    processed: n + 1,
                    total: discoveries.len(),
                });
            }
        }
        report.removed = self.remove_unseen(&seen)?;
        self.finish_projects(&projects)?;
        self.conn.execute_batch("COMMIT").map_err(sql)?;
        on_event(IndexEvent::Progress {
            processed: discoveries.len(),
            total: discoveries.len(),
        });
        Ok(report)
    }

    /// Upserts one folder. Returns the instance id and whether it was re-read, or `None` when
    /// the folder has no primary file for its project's preset.
    fn process(
        &self,
        d: &Discovery,
        config: &Config,
        git: &mut GitResolver,
        projects: &mut BTreeMap<PathBuf, ProjectState>,
    ) -> Result<Option<(i64, bool)>, IndexError> {
        let identity = git.project(&d.repo_root);
        let preset = config.preset_for(&identity.name);
        if !d.files.iter().any(|f| preset.is_primary(f)) {
            return Ok(None);
        }
        let key = identity.key.to_string_lossy().into_owned();
        let state = match projects.get(&identity.key) {
            Some(s) => s.clone(),
            None => {
                let previous: Option<(i64, String)> = self
                    .conn
                    .query_row(
                        "SELECT id, preset FROM projects WHERE git_common_dir = ?1",
                        [&key],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                    .optional()
                    .map_err(sql)?;
                let id = match previous {
                    Some((id, _)) => id,
                    None => self
                        .conn
                        .query_row(
                            "INSERT INTO projects (git_common_dir, name, preset) VALUES (?1, ?2, ?3) RETURNING id",
                            params![key, identity.name, preset.name],
                            |r| r.get(0),
                        )
                        .map_err(sql)?,
                };
                let s = ProjectState {
                    id,
                    name: identity.name.clone(),
                    alias: config.alias_for(&identity.name).map(str::to_owned),
                    preset: preset.name.clone(),
                    explicit_preset: config.has_explicit_preset(&identity.name),
                    // A different preset changes primary files and parsing: re-read everything.
                    force: previous.is_some_and(|(_, p)| p != preset.name),
                    git_warning: identity.git_error.as_ref().map(ToString::to_string),
                    folders: vec![],
                };
                projects.insert(identity.key.clone(), s.clone());
                s
            }
        };
        if let Some(s) = projects.get_mut(&identity.key) {
            s.folders.push(d.files.clone());
        }

        let spec_id: i64 = self
            .conn
            .query_row(
                "INSERT INTO specs (project_id, folder, issue_key) VALUES (?1, ?2, ?3)
                 ON CONFLICT(project_id, folder) DO UPDATE SET issue_key = excluded.issue_key
                 RETURNING id",
                params![state.id, d.folder, preset.issue_key(&d.folder)],
                |r| r.get(0),
            )
            .map_err(sql)?;

        let facts = git.worktree_facts(&d.repo_root);
        let kind = if facts.linked { "worktree" } else { "checkout" };
        let path = d.spec_dir.to_string_lossy().into_owned();
        let stats = stat_files(&d.spec_dir, &d.files);

        let existing: Option<(i64, i64)> = self
            .conn
            .query_row(
                "SELECT id, spec_id FROM instances WHERE path = ?1",
                [&path],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(sql)?;
        let unchanged = match existing {
            Some((id, old_spec)) if old_spec == spec_id && !state.force => {
                self.files_match(id, &stats)?.then_some(id)
            }
            _ => None,
        };
        if let Some(id) = unchanged {
            // Branch can change without any file changing (checkout in a worktree).
            self.conn
                .execute(
                    "UPDATE instances SET branch = ?2, kind = ?3, repo_root = ?4 WHERE id = ?1",
                    params![id, facts.branch, kind, d.repo_root.to_string_lossy()],
                )
                .map_err(sql)?;
            return Ok(Some((id, false)));
        }

        let (content_hash, mtime) =
            model::fingerprint(&d.spec_dir, &d.files, &|f| preset.is_primary(f));
        let meta = derive_meta(&d.spec_dir, &d.files, preset);
        let id: i64 = self
            .conn
            .query_row(
                "INSERT INTO instances (spec_id, path, repo_root, branch, kind, mtime, content_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(path) DO UPDATE SET spec_id = excluded.spec_id, repo_root = excluded.repo_root,
                   branch = excluded.branch, kind = excluded.kind, mtime = excluded.mtime,
                   content_hash = excluded.content_hash
                 RETURNING id",
                params![spec_id, path, d.repo_root.to_string_lossy(), facts.branch, kind, mtime, content_hash],
                |r| r.get(0),
            )
            .map_err(sql)?;
        self.conn
            .execute("DELETE FROM files WHERE instance_id = ?1", [id])
            .map_err(sql)?;
        {
            let mut ins = self
                .conn
                .prepare_cached(
                    "INSERT INTO files (instance_id, name, size, mtime) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(sql)?;
            for (name, size, mtime) in &stats {
                ins.execute(params![id, name, *size as i64, mtime])
                    .map_err(sql)?;
            }
        }
        self.conn
            .execute(
                "INSERT OR REPLACE INTO spec_meta (instance_id, title, status, issue_url, tasks_done, tasks_total)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    id,
                    meta.title,
                    meta.status,
                    meta.issue_url,
                    meta.progress.map(|p| p.0),
                    meta.progress.map(|p| p.1)
                ],
            )
            .map_err(sql)?;
        Ok(Some((id, true)))
    }

    fn files_match(
        &self,
        instance_id: i64,
        stats: &[(String, u64, Option<i64>)],
    ) -> Result<bool, IndexError> {
        Ok(self.files(instance_id)? == stats)
    }

    fn remove_unseen(&self, seen: &HashSet<i64>) -> Result<usize, IndexError> {
        let ids: Vec<i64> = {
            let mut stmt = self.conn.prepare("SELECT id FROM instances").map_err(sql)?;
            let rows = stmt.query_map([], |r| r.get(0)).map_err(sql)?;
            rows.collect::<Result<_, _>>().map_err(sql)?
        };
        let mut removed = 0;
        for id in ids.into_iter().filter(|id| !seen.contains(id)) {
            removed += self
                .conn
                .execute("DELETE FROM instances WHERE id = ?1", [id])
                .map_err(sql)?;
        }
        self.conn
            .execute_batch(
                "DELETE FROM specs WHERE id NOT IN (SELECT spec_id FROM instances);
                 DELETE FROM projects WHERE id NOT IN (SELECT project_id FROM specs);",
            )
            .map_err(sql)?;
        Ok(removed)
    }

    fn finish_projects(
        &self,
        projects: &BTreeMap<PathBuf, ProjectState>,
    ) -> Result<(), IndexError> {
        for p in projects.values() {
            let folders: Vec<&[String]> = p.folders.iter().map(Vec::as_slice).collect();
            let suggested = if p.explicit_preset {
                None
            } else {
                preset::suggest(&p.preset, &folders)
            };
            self.conn
                .execute(
                    "UPDATE projects SET name = ?2, alias = ?3, preset = ?4, suggested_preset = ?5, git_warning = ?6
                     WHERE id = ?1",
                    params![p.id, p.name, p.alias, p.preset, suggested, p.git_warning],
                )
                .map_err(sql)?;
        }
        Ok(())
    }
}

/// An instance with what is needed to read its files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceInfo {
    pub id: i64,
    pub spec_id: i64,
    pub project_id: i64,
    pub path: PathBuf,
    pub repo_root: PathBuf,
    pub project_name: String,
    pub preset: String,
    /// `(name, size, mtime_ms)`, sorted by name.
    pub files: Vec<(String, u64, Option<i64>)>,
}

#[derive(Debug, Clone)]
struct ProjectState {
    id: i64,
    name: String,
    alias: Option<String>,
    preset: String,
    explicit_preset: bool,
    force: bool,
    git_warning: Option<String>,
    folders: Vec<Vec<String>>,
}

fn stat_files(dir: &Path, files: &[String]) -> Vec<(String, u64, Option<i64>)> {
    files
        .iter()
        .filter_map(|name| {
            let meta = fs::metadata(dir.join(name)).ok()?;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .and_then(|d| i64::try_from(d.as_millis()).ok());
            Some((name.clone(), meta.len(), mtime))
        })
        .collect()
}

/// Values shown in the list without opening files.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstanceMeta {
    pub title: Option<String>,
    pub status: Option<String>,
    pub issue_url: Option<String>,
    pub progress: Option<(u32, u32)>,
}

/// Reads the files that feed the list: title (first `# ` of the first tab), header `Status`
/// and `Issue` link, and task progress when the preset has a tasks grammar.
pub fn derive_meta(dir: &Path, files: &[String], preset: &Preset) -> InstanceMeta {
    let read = |name: &str| fs::read_to_string(dir.join(name)).ok();
    let mut meta = InstanceMeta::default();
    let tabs = preset.tabs(files);
    if let Some(text) = tabs
        .iter()
        .filter(|t| t.ends_with(".md"))
        .find_map(|t| read(t))
    {
        meta.title = text
            .lines()
            .find_map(|l| l.trim().strip_prefix("# ").map(|t| t.trim().to_owned()))
            .filter(|t| !t.is_empty());
    }
    for name in preset.header_files.iter().filter(|h| files.contains(h)) {
        let Some(text) = read(name) else { continue };
        for field in parse::header(&text) {
            let key = field.key.to_lowercase();
            if key == "status" && meta.status.is_none() {
                meta.status = Some(field.text.clone());
            }
            if key == "issue" && meta.issue_url.is_none() {
                meta.issue_url = field.url.clone();
            }
        }
    }
    let has_tasks = preset.tasks == TasksGrammar::Phases && files.iter().any(|f| f == "tasks.md");
    if let Some(text) = has_tasks.then(|| read("tasks.md")).flatten() {
        let doc = parse::tasks(&text, None);
        if !doc.is_empty() {
            let (d, t) = doc.progress();
            meta.progress = Some((d as u32, t as u32));
        }
    }
    meta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_from_exban_files() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("spec.md"), "# ACME-1 — Título\n\n**Status:** Em andamento\n**Issue:** [ACME-1](https://t.example/ACME-1)\n").unwrap();
        fs::write(
            tmp.path().join("tasks.md"),
            "### Fase 1: A\n- [x] T1: a\n- [ ] T2: b\n",
        )
        .unwrap();
        let files = vec!["spec.md".to_owned(), "tasks.md".to_owned()];
        let meta = derive_meta(tmp.path(), &files, &Preset::exban());
        assert_eq!(meta.title.as_deref(), Some("ACME-1 — Título"));
        assert_eq!(meta.status.as_deref(), Some("Em andamento"));
        assert_eq!(meta.issue_url.as_deref(), Some("https://t.example/ACME-1"));
        assert_eq!(meta.progress, Some((1, 2)));
        let generic = derive_meta(tmp.path(), &files, &Preset::generic());
        assert_eq!(generic.status, None);
        assert_eq!(generic.progress, None);
        assert_eq!(generic.title.as_deref(), Some("ACME-1 — Título"));
    }

    #[test]
    fn foreign_or_old_database_is_rebuilt() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("index.sqlite");
        fs::write(&path, b"this is not a sqlite database at all, just bytes").unwrap();
        let index = Index::open(&path).unwrap();
        assert!(index.list().unwrap().is_empty());
        drop(index);
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
        drop(conn);
        let index = Index::open(&path).unwrap();
        assert!(index.list().unwrap().is_empty());
    }
}
