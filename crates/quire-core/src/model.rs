//! Projects, specs and instances, with deduplication across worktrees.
//!
//! A spec is identified by `(project, folder name)` (DUP-02). Each copy on disk is an
//! [`Instance`]. Instances whose primary files have the same content are identical; when any
//! differs, the spec shows the "N versões" badge with N = number of instances (DUP-03, as in
//! wireframe 4). The default instance is the most recently modified one, ties broken by path
//! (DUP-04, A3).

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use xxhash_rust::xxh3::Xxh3;

use crate::config::Config;
use crate::git::{GitResolver, NameSource, ProjectIdentity};
use crate::preset;
use crate::scan::Discovery;

/// Main checkout or linked worktree (DUP-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceKind {
    Checkout,
    Worktree,
}

/// One copy of a spec folder on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instance {
    pub spec_dir: PathBuf,
    pub repo_root: PathBuf,
    pub branch: Option<String>,
    pub kind: InstanceKind,
    /// Largest mtime among the primary files, in ms since the Unix epoch (A3).
    pub mtime_ms: Option<i64>,
    /// Hash of the primary files (names and LF-normalized content).
    pub content_hash: String,
    /// All regular files of the folder, sorted.
    pub files: Vec<String>,
}

/// State of an instance compared with a reference instance (normally the open one).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceState {
    Identical,
    Diverges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub folder: String,
    /// Issue key from the folder name, per the project's preset (`ACME-142`).
    pub issue_key: Option<String>,
    /// Sorted by path.
    pub instances: Vec<Instance>,
    /// Index into `instances` of the default instance (DUP-04).
    pub default_index: usize,
}

impl Spec {
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// `true` when the primary files of at least two instances differ.
    pub fn diverges(&self) -> bool {
        self.instances
            .windows(2)
            .any(|w| w[0].content_hash != w[1].content_hash)
    }

    /// N for the "N versões" badge, or `None` when all copies are identical (DUP-03).
    pub fn versions_badge(&self) -> Option<usize> {
        self.diverges().then(|| self.instance_count())
    }

    pub fn default_instance(&self) -> Option<&Instance> {
        self.instances.get(self.default_index)
    }

    /// State of instance `index` relative to instance `reference` (DUP-05).
    pub fn state_of(&self, index: usize, reference: usize) -> Option<InstanceState> {
        let a = self.instances.get(index)?;
        let b = self.instances.get(reference)?;
        Some(if a.content_hash == b.content_hash {
            InstanceState::Identical
        } else {
            InstanceState::Diverges
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub key: PathBuf,
    pub name: String,
    pub name_source: NameSource,
    pub origin_url: Option<String>,
    pub git_error: Option<crate::GitError>,
    /// Configured alias (DUP-01); shown instead of `name` when set.
    pub alias: Option<String>,
    /// Name of the preset in use (PST-02).
    pub preset: String,
    /// Preset suggested from the folder structure when none is configured (PST-01).
    pub suggested_preset: Option<String>,
    /// Sorted by folder name.
    pub specs: Vec<Spec>,
}

impl Project {
    pub fn display_name(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.name)
    }
}

/// Index of the default instance: newest mtime, then alphabetical path (deterministic).
pub fn default_index(instances: &[Instance]) -> usize {
    instances
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            b.mtime_ms
                .cmp(&a.mtime_ms)
                .then_with(|| a.spec_dir.cmp(&b.spec_dir))
        })
        .map_or(0, |(i, _)| i)
}

/// Content hash and newest mtime of the primary files of a folder.
pub fn fingerprint(
    spec_dir: &Path,
    files: &[String],
    is_primary: &dyn Fn(&str) -> bool,
) -> (String, Option<i64>) {
    let mut hasher = Xxh3::new();
    let mut newest: Option<i64> = None;
    for name in files.iter().filter(|f| is_primary(f)) {
        let path = spec_dir.join(name);
        hasher.update(name.as_bytes());
        hasher.update(&[0]);
        match fs::read(&path) {
            Ok(bytes) => {
                let normalized = normalize_newlines(&bytes);
                hasher.update(&(normalized.len() as u64).to_le_bytes());
                hasher.update(&normalized);
            }
            // Unreadable files still count as "present but different from readable ones".
            Err(_) => hasher.update(b"\xffunreadable"),
        }
        let mtime = fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .and_then(|d| i64::try_from(d.as_millis()).ok());
        newest = newest.max(mtime);
    }
    (format!("{:032x}", hasher.digest128()), newest)
}

fn normalize_newlines(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut it = bytes.iter().peekable();
    while let Some(&b) = it.next() {
        if b == b'\r' && it.peek() == Some(&&b'\n') {
            continue;
        }
        out.push(b);
    }
    out
}

/// Groups discoveries into projects and specs, resolving git facts, presets and fingerprints.
///
/// Folders without any primary file of their project's preset are dropped (DSC-02).
pub fn build_projects(
    discoveries: &[Discovery],
    git: &mut GitResolver,
    config: &Config,
) -> Vec<Project> {
    let mut identities: HashMap<PathBuf, ProjectIdentity> = HashMap::new();
    let mut groups: BTreeMap<PathBuf, BTreeMap<String, Vec<Instance>>> = BTreeMap::new();
    for d in discoveries {
        let identity = git.project(&d.repo_root);
        let preset = config.preset_for(&identity.name);
        if !d.files.iter().any(|f| preset.is_primary(f)) {
            continue;
        }
        let facts = git.worktree_facts(&d.repo_root);
        let (content_hash, mtime_ms) =
            fingerprint(&d.spec_dir, &d.files, &|f| preset.is_primary(f));
        let instance = Instance {
            spec_dir: d.spec_dir.clone(),
            repo_root: d.repo_root.clone(),
            branch: facts.branch,
            kind: if facts.linked {
                InstanceKind::Worktree
            } else {
                InstanceKind::Checkout
            },
            mtime_ms,
            content_hash,
            files: d.files.clone(),
        };
        let list = groups
            .entry(identity.key.clone())
            .or_default()
            .entry(d.folder.clone())
            .or_default();
        if !list.iter().any(|i| i.spec_dir == instance.spec_dir) {
            list.push(instance);
        }
        identities.entry(identity.key.clone()).or_insert(identity);
    }
    let mut projects: Vec<Project> = groups
        .into_iter()
        .filter_map(|(key, specs)| {
            let id = identities.remove(&key)?;
            let preset = config.preset_for(&id.name);
            let specs: Vec<Spec> = specs
                .into_iter()
                .map(|(folder, mut instances)| {
                    instances.sort_by(|a, b| a.spec_dir.cmp(&b.spec_dir));
                    let default_index = default_index(&instances);
                    Spec {
                        issue_key: preset.issue_key(&folder),
                        folder,
                        instances,
                        default_index,
                    }
                })
                .collect();
            let folders: Vec<&[String]> = specs
                .iter()
                .flat_map(|s| s.instances.iter().map(|i| i.files.as_slice()))
                .collect();
            let suggested_preset = if config.has_explicit_preset(&id.name) {
                None
            } else {
                preset::suggest(&preset.name, &folders).map(str::to_owned)
            };
            Some(Project {
                key,
                alias: config.alias_for(&id.name).map(str::to_owned),
                preset: preset.name.clone(),
                suggested_preset,
                name: id.name,
                name_source: id.name_source,
                origin_url: id.origin_url,
                git_error: id.git_error,
                specs,
            })
        })
        .collect();
    projects.sort_by(|a, b| {
        a.display_name()
            .cmp(b.display_name())
            .then_with(|| a.key.cmp(&b.key))
    });
    projects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inst(path: &str, mtime: Option<i64>, hash: &str) -> Instance {
        Instance {
            spec_dir: PathBuf::from(path),
            repo_root: PathBuf::from("/r"),
            branch: None,
            kind: InstanceKind::Checkout,
            mtime_ms: mtime,
            content_hash: hash.into(),
            files: vec![],
        }
    }

    fn spec(instances: Vec<Instance>) -> Spec {
        let default_index = default_index(&instances);
        Spec {
            folder: "s".into(),
            issue_key: None,
            instances,
            default_index,
        }
    }

    #[test]
    fn identical_copies_have_no_badge() {
        let s = spec(vec![inst("/a", Some(1), "h"), inst("/b", Some(2), "h")]);
        assert!(!s.diverges());
        assert_eq!(s.versions_badge(), None);
        assert_eq!(s.instance_count(), 2);
    }

    #[test]
    fn divergent_copies_show_instance_count_and_state() {
        let s = spec(vec![
            inst("/a", Some(1), "h1"),
            inst("/b", Some(2), "h2"),
            inst("/c", Some(3), "h1"),
        ]);
        assert_eq!(s.versions_badge(), Some(3));
        assert_eq!(s.state_of(0, 2), Some(InstanceState::Identical));
        assert_eq!(s.state_of(1, 2), Some(InstanceState::Diverges));
        assert_eq!(s.state_of(9, 0), None);
    }

    #[test]
    fn default_is_newest_then_alphabetical() {
        assert_eq!(
            default_index(&[inst("/a", Some(1), "h"), inst("/b", Some(5), "h")]),
            1
        );
        assert_eq!(
            default_index(&[inst("/b", Some(5), "h"), inst("/a", Some(5), "h")]),
            1
        );
        assert_eq!(
            default_index(&[inst("/a", None, "h"), inst("/b", Some(0), "h")]),
            1
        );
        assert_eq!(default_index(&[]), 0);
    }

    #[test]
    fn fingerprint_ignores_line_endings_and_non_primary_files() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b) = (tmp.path().join("a"), tmp.path().join("b"));
        fs::create_dir_all(&a).unwrap();
        fs::create_dir_all(&b).unwrap();
        fs::write(a.join("spec.md"), "x\ny\n").unwrap();
        fs::write(b.join("spec.md"), "x\r\ny\r\n").unwrap();
        fs::write(b.join("image.png"), "binary").unwrap();
        let md = |f: &str| f.ends_with(".md");
        let files_a = vec!["spec.md".to_owned()];
        let files_b = vec!["image.png".to_owned(), "spec.md".to_owned()];
        let (ha, ma) = fingerprint(&a, &files_a, &md);
        let (hb, _) = fingerprint(&b, &files_b, &md);
        assert_eq!(ha, hb);
        assert!(ma.is_some());
        fs::write(b.join("spec.md"), "x\ny\nz\n").unwrap();
        assert_ne!(ha, fingerprint(&b, &files_b, &md).0);
    }

    #[test]
    fn a_missing_primary_file_is_a_divergence() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("spec.md"), "x").unwrap();
        fs::write(tmp.path().join("tasks.md"), "y").unwrap();
        let md = |f: &str| f.ends_with(".md");
        let one = fingerprint(tmp.path(), &["spec.md".to_owned()], &md).0;
        let two = fingerprint(
            tmp.path(),
            &["spec.md".to_owned(), "tasks.md".to_owned()],
            &md,
        )
        .0;
        assert_ne!(one, two);
    }
}
