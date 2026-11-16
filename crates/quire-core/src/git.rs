//! Project identity and worktree discovery through the system `git` binary (TD-03).
//!
//! Every call has a timeout, never takes optional locks (agents may be committing at the same
//! time) and ignores `GIT_DIR`-style variables from the environment. Results are cached per
//! repository for the lifetime of a [`GitResolver`], which is meant to live for one scan.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub use crate::error::GitError;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Where a project's display name came from (DUP-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSource {
    /// Last segment of the `origin` remote URL, without `.git`.
    Origin,
    /// Folder name of the main repository (no `origin`, or git unavailable).
    Folder,
}

/// Identity of a project: the git common dir of a repository and its worktrees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectIdentity {
    /// Stable key: the canonical git common dir, or the repository root when git is unavailable.
    pub key: PathBuf,
    pub name: String,
    pub name_source: NameSource,
    pub origin_url: Option<String>,
    /// Root of the main working tree (parent of the common dir), when known.
    pub main_root: PathBuf,
    /// Set when the identity is a fallback because git failed (A4).
    pub git_error: Option<GitError>,
}

/// Git facts about one working tree (an instance of a spec lives in one).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeFacts {
    /// Current branch, or `None` when HEAD is detached or git is unavailable.
    pub branch: Option<String>,
    /// `true` for a linked worktree, `false` for the main checkout (DUP-05).
    pub linked: bool,
}

/// One entry of `git worktree list --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeEntry {
    pub path: PathBuf,
    pub branch: Option<String>,
    pub bare: bool,
    pub prunable: bool,
}

#[derive(Debug, Clone)]
struct RepoFacts {
    common_dir: PathBuf,
    git_dir: PathBuf,
    branch: Option<String>,
}

/// Resolves and caches git facts during a scan.
#[derive(Debug)]
pub struct GitResolver {
    binary: PathBuf,
    timeout: Duration,
    repos: HashMap<PathBuf, Result<RepoFacts, GitError>>,
    projects: HashMap<PathBuf, ProjectIdentity>,
    worktrees: HashMap<PathBuf, Vec<WorktreeEntry>>,
}

impl Default for GitResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl GitResolver {
    pub fn new() -> Self {
        Self::with_binary("git")
    }

    /// Uses a specific git binary (tests use a missing one to exercise the fallback).
    pub fn with_binary(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            timeout: DEFAULT_TIMEOUT,
            repos: HashMap::new(),
            projects: HashMap::new(),
            worktrees: HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Project of the working tree rooted at `repo_root` (the folder that contains `.specs`).
    pub fn project(&mut self, repo_root: &Path) -> ProjectIdentity {
        let facts = match self.repo(repo_root) {
            Ok(f) => f,
            Err(e) => return fallback_identity(repo_root, e),
        };
        if let Some(p) = self.projects.get(&facts.common_dir) {
            return p.clone();
        }
        let main_root = main_root_of(&facts.common_dir);
        let origin = self
            .run(repo_root, &["config", "--get", "remote.origin.url"])
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        let (name, name_source) = match origin.as_deref().and_then(name_from_url) {
            Some(n) => (n, NameSource::Origin),
            None => (folder_name(&main_root), NameSource::Folder),
        };
        let identity = ProjectIdentity {
            key: facts.common_dir.clone(),
            name,
            name_source,
            origin_url: origin,
            main_root,
            git_error: None,
        };
        self.projects.insert(facts.common_dir, identity.clone());
        identity
    }

    /// Branch and kind of the working tree rooted at `repo_root`.
    pub fn worktree_facts(&mut self, repo_root: &Path) -> WorktreeFacts {
        match self.repo(repo_root) {
            Ok(f) => WorktreeFacts {
                branch: f.branch,
                linked: f.git_dir != f.common_dir,
            },
            Err(_) => WorktreeFacts {
                branch: None,
                linked: false,
            },
        }
    }

    /// All worktrees of the repository that `repo_root` belongs to (cached per repository).
    pub fn worktrees(&mut self, repo_root: &Path) -> Result<Vec<WorktreeEntry>, GitError> {
        let common = self.repo(repo_root)?.common_dir;
        if let Some(w) = self.worktrees.get(&common) {
            return Ok(w.clone());
        }
        let out = self.run(repo_root, &["worktree", "list", "--porcelain"])?;
        let list = parse_worktree_list(&out);
        self.worktrees.insert(common, list.clone());
        Ok(list)
    }

    fn repo(&mut self, repo_root: &Path) -> Result<RepoFacts, GitError> {
        if let Some(r) = self.repos.get(repo_root) {
            return r.clone();
        }
        let result = self.load_repo(repo_root);
        self.repos.insert(repo_root.to_path_buf(), result.clone());
        result
    }

    fn load_repo(&self, repo_root: &Path) -> Result<RepoFacts, GitError> {
        let out = self.run(
            repo_root,
            &[
                "rev-parse",
                "--path-format=absolute",
                "--git-dir",
                "--git-common-dir",
            ],
        )?;
        let mut lines = out.lines();
        let (Some(git_dir), Some(common_dir)) = (lines.next(), lines.next()) else {
            return Err(GitError::UnexpectedOutput("rev-parse".into()));
        };
        let canon = |p: &str| {
            PathBuf::from(p)
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from(p))
        };
        // `symbolic-ref` fails on a detached HEAD, which is not an error for us.
        let branch = self
            .run(repo_root, &["symbolic-ref", "--short", "-q", "HEAD"])
            .ok()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
        Ok(RepoFacts {
            common_dir: canon(common_dir),
            git_dir: canon(git_dir),
            branch,
        })
    }

    /// Runs `git -C dir args...` with a timeout and returns stdout.
    fn run(&self, dir: &Path, args: &[&str]) -> Result<String, GitError> {
        let mut child = Command::new(&self.binary)
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("LC_ALL", "C")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_COMMON_DIR")
            .env_remove("GIT_INDEX_FILE")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => GitError::NotInstalled,
                _ => GitError::Spawn(e.to_string()),
            })?;
        let stdout = child.stdout.take().map(|mut s| {
            std::thread::spawn(move || {
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                buf
            })
        });
        let stderr = child.stderr.take().map(|mut s| {
            std::thread::spawn(move || {
                let mut buf = String::new();
                let _ = s.read_to_string(&mut buf);
                buf
            })
        });
        let deadline = Instant::now() + self.timeout;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if Instant::now() >= deadline => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(GitError::Timeout {
                        args: args.join(" "),
                        secs: self.timeout.as_secs(),
                    });
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(2)),
                Err(e) => return Err(GitError::Spawn(e.to_string())),
            }
        };
        let out = stdout.and_then(|h| h.join().ok()).unwrap_or_default();
        let err = stderr.and_then(|h| h.join().ok()).unwrap_or_default();
        if status.success() {
            Ok(out)
        } else {
            Err(GitError::Failed {
                args: args.join(" "),
                code: status.code(),
                stderr: err.trim().to_owned(),
            })
        }
    }
}

fn fallback_identity(repo_root: &Path, error: GitError) -> ProjectIdentity {
    ProjectIdentity {
        key: repo_root.to_path_buf(),
        name: folder_name(repo_root),
        name_source: NameSource::Folder,
        origin_url: None,
        main_root: repo_root.to_path_buf(),
        git_error: Some(error),
    }
}

/// The main working tree is the parent of a `.git` common dir; a bare repo is its own root.
fn main_root_of(common_dir: &Path) -> PathBuf {
    if common_dir.file_name().is_some_and(|n| n == ".git") {
        common_dir
            .parent()
            .map_or_else(|| common_dir.to_path_buf(), Path::to_path_buf)
    } else {
        common_dir.to_path_buf()
    }
}

fn folder_name(p: &Path) -> String {
    let name = p.file_name().map_or_else(
        || p.to_string_lossy().into_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    name.strip_suffix(".git")
        .map_or(name.clone(), str::to_owned)
}

/// Project name from a remote URL: last path segment without `.git` (DUP-01).
///
/// Handles `git@host:org/repo.git`, `ssh://…/repo.git`, `https://…/repo/`, local paths and
/// Windows paths.
pub fn name_from_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches(['/', '\\']);
    let last = trimmed.rsplit(['/', '\\', ':']).next()?;
    let name = last.strip_suffix(".git").unwrap_or(last);
    (!name.is_empty()).then(|| name.to_owned())
}

/// Parses `git worktree list --porcelain`.
pub fn parse_worktree_list(out: &str) -> Vec<WorktreeEntry> {
    let mut list = Vec::new();
    let mut cur: Option<WorktreeEntry> = None;
    for line in out.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            if let Some(done) = cur.take() {
                list.push(done);
            }
            cur = Some(WorktreeEntry {
                path: PathBuf::from(path),
                branch: None,
                bare: false,
                prunable: false,
            });
        } else if let Some(entry) = cur.as_mut() {
            if let Some(b) = line.strip_prefix("branch ") {
                entry.branch = Some(b.strip_prefix("refs/heads/").unwrap_or(b).to_owned());
            } else if line == "bare" {
                entry.bare = true;
            } else if line.starts_with("prunable") {
                entry.prunable = true;
            }
        }
    }
    list.extend(cur);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_from_urls() {
        let cases = [
            (
                "git@git.example.com:acme/acme_gateway.git",
                Some("acme_gateway"),
            ),
            (
                "https://git.example.com/acme/acme_gateway.git",
                Some("acme_gateway"),
            ),
            (
                "https://git.example.com/acme/acme_gateway/",
                Some("acme_gateway"),
            ),
            (
                "ssh://git@git.example.com:2222/acme/billing",
                Some("billing"),
            ),
            ("/srv/git/tools.git", Some("tools")),
            (r"C:\repos\local-repo", Some("local-repo")),
            ("git@host:repo.git", Some("repo")),
            ("", None),
        ];
        for (url, want) in cases {
            assert_eq!(name_from_url(url).as_deref(), want, "{url}");
        }
    }

    #[test]
    fn parses_porcelain_worktree_list() {
        let out = "worktree /a/main\nHEAD 1111\nbranch refs/heads/main\n\nworktree /a/wt\nHEAD 2222\ndetached\n\nworktree /a/gone\nHEAD 3333\nbranch refs/heads/feat/x\nprunable gitdir file points to non-existent location\n";
        let list = parse_worktree_list(out);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].branch.as_deref(), Some("main"));
        assert_eq!(list[1].branch, None);
        assert_eq!(list[2].branch.as_deref(), Some("feat/x"));
        assert!(list[2].prunable);
    }

    #[test]
    fn main_root_of_common_dir() {
        assert_eq!(
            main_root_of(Path::new("/r/app/.git")),
            PathBuf::from("/r/app")
        );
        assert_eq!(
            main_root_of(Path::new("/r/bare.git")),
            PathBuf::from("/r/bare.git")
        );
        assert_eq!(folder_name(Path::new("/r/bare.git")), "bare");
    }

    #[test]
    fn missing_git_binary_falls_back_without_panicking() {
        let tmp = tempfile::tempdir().unwrap();
        let mut git = GitResolver::with_binary("/nonexistent/quire-test-git");
        let id = git.project(tmp.path());
        assert_eq!(id.git_error, Some(GitError::NotInstalled));
        assert_eq!(id.name_source, NameSource::Folder);
        assert_eq!(id.key, tmp.path());
        assert_eq!(
            git.worktree_facts(tmp.path()),
            WorktreeFacts {
                branch: None,
                linked: false
            }
        );
        assert_eq!(git.worktrees(tmp.path()), Err(GitError::NotInstalled));
    }

    #[test]
    fn folder_outside_any_repo_falls_back() {
        let tmp = tempfile::tempdir().unwrap();
        let mut git = GitResolver::new();
        let id = git.project(tmp.path());
        assert!(matches!(
            id.git_error,
            Some(GitError::Failed { .. }) | Some(GitError::NotInstalled)
        ));
        assert_eq!(id.name, folder_name(tmp.path()));
    }
}
