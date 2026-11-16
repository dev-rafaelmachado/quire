//! Discovery of spec folders (`<repo>/.specs/features/<folder>/`) under scan paths.
//!
//! The walk respects `.gitignore` and always skips `node_modules`, `target`, `dist` and `.git`
//! (DSC-01). It runs on the parallel walker of the `ignore` crate (spike T3: 6.6x faster on a
//! cold cache) and reports each discovery as soon as it is found (DSC-04). Problems with a scan
//! path are reported for that path only (DSC-05).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use ignore::{DirEntry, WalkBuilder, WalkState};

use crate::error::ScanError;

/// Directory names that are never walked, with or without a `.gitignore`.
pub const IGNORED_DIRS: [&str; 4] = ["node_modules", "target", "dist", ".git"];

/// A spec folder found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Discovery {
    /// `<repo>/.specs/features/<folder>`, canonicalized.
    pub spec_dir: PathBuf,
    /// The folder name, which is the spec identity inside a project (DUP-02).
    pub folder: String,
    /// The directory that contains `.specs` (a repository or worktree root).
    pub repo_root: PathBuf,
    /// Names of the regular files directly inside `spec_dir`, sorted.
    pub files: Vec<String>,
}

/// Progress of a scan, delivered in order to the callback of [`scan`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanEvent {
    Found(Discovery),
    PathError { path: PathBuf, error: ScanError },
}

/// Summary returned when a scan finishes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanReport {
    pub found: usize,
    pub path_errors: usize,
}

/// Decides whether a folder under `.specs/features` is a spec, from its file names (DSC-02).
pub type SpecFilter<'a> = dyn Fn(&[String]) -> bool + Sync + 'a;

/// Filter of the `generic` preset: any `.md` or `.jsonl` file (PST-01).
pub fn generic_filter(files: &[String]) -> bool {
    files
        .iter()
        .any(|f| f.ends_with(".md") || f.ends_with(".jsonl"))
}

/// Scans every path and calls `on_event` for each discovery or path error, as they happen.
///
/// Spec folders reached through more than one scan path (overlapping paths, symlinks to the
/// same place) are reported once.
pub fn scan(
    paths: &[PathBuf],
    filter: &SpecFilter<'_>,
    mut on_event: impl FnMut(ScanEvent),
) -> ScanReport {
    let mut report = ScanReport::default();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for path in paths {
        if let Err(error) = check_scan_path(path) {
            report.path_errors += 1;
            on_event(ScanEvent::PathError {
                path: path.clone(),
                error,
            });
            continue;
        }
        let (tx, rx) = mpsc::channel::<Discovery>();
        std::thread::scope(|s| {
            s.spawn(move || walk(path, filter, &tx));
            for d in rx {
                if seen.insert(d.spec_dir.clone()) {
                    report.found += 1;
                    on_event(ScanEvent::Found(d));
                }
            }
        });
    }
    report
}

/// Like [`scan`], then also scans the worktrees of every repository found that live outside
/// the scan paths (DSC-03). Worktrees that are prunable (folder gone) are skipped.
pub fn scan_with_worktrees(
    paths: &[PathBuf],
    filter: &SpecFilter<'_>,
    git: &mut crate::git::GitResolver,
    mut on_event: impl FnMut(ScanEvent),
) -> ScanReport {
    let mut repo_roots: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut report = scan(paths, filter, |e| {
        if let ScanEvent::Found(d) = &e {
            seen.insert(d.spec_dir.clone());
            if !repo_roots.contains(&d.repo_root) {
                repo_roots.push(d.repo_root.clone());
            }
        }
        on_event(e);
    });
    let roots: Vec<PathBuf> = paths.iter().filter_map(|p| p.canonicalize().ok()).collect();
    let mut extra: Vec<PathBuf> = Vec::new();
    for repo in &repo_roots {
        let Ok(list) = git.worktrees(repo) else {
            continue;
        };
        for wt in list {
            if wt.bare || wt.prunable {
                continue;
            }
            let Ok(path) = wt.path.canonicalize() else {
                continue;
            };
            let inside = roots.iter().any(|r| path.starts_with(r));
            if !inside && !extra.contains(&path) {
                extra.push(path);
            }
        }
    }
    if !extra.is_empty() {
        scan(&extra, filter, |e| {
            match &e {
                // Discoveries already reported through the scan paths are dropped.
                ScanEvent::Found(d) if !seen.insert(d.spec_dir.clone()) => return,
                ScanEvent::Found(_) => report.found += 1,
                ScanEvent::PathError { .. } => report.path_errors += 1,
            }
            on_event(e);
        });
    }
    report
}

/// Whether a scan path can be scanned: it exists, is a directory and is readable (DSC-05).
pub fn check_scan_path(path: &Path) -> Result<(), ScanError> {
    let meta = fs::metadata(path).map_err(|e| ScanError::from_io(path.to_path_buf(), &e))?;
    if !meta.is_dir() {
        return Err(ScanError::NotADirectory(path.to_path_buf()));
    }
    fs::read_dir(path).map_err(|e| ScanError::from_io(path.to_path_buf(), &e))?;
    Ok(())
}

fn keep_entry(entry: &DirEntry) -> bool {
    let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
    !(is_dir && IGNORED_DIRS.iter().any(|d| entry.file_name() == *d))
}

fn walk(root: &Path, filter: &SpecFilter<'_>, tx: &mpsc::Sender<Discovery>) {
    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(false)
        .git_global(false)
        .parents(false)
        .follow_links(false)
        .filter_entry(keep_entry)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    // Unreadable subdirectories are skipped; only scan roots are reported.
                    return WalkState::Continue;
                };
                let is_dir = entry.file_type().is_some_and(|t| t.is_dir());
                if is_dir && entry.file_name() == ".specs" {
                    for d in specs_in(entry.path(), filter) {
                        if tx.send(d).is_err() {
                            return WalkState::Quit;
                        }
                    }
                    // Nothing inside `.specs` is another repository.
                    return WalkState::Skip;
                }
                WalkState::Continue
            })
        });
}

/// Lists the spec folders of one `.specs` directory.
fn specs_in(specs_dir: &Path, filter: &SpecFilter<'_>) -> Vec<Discovery> {
    let Some(repo_root) = specs_dir.parent().and_then(|p| p.canonicalize().ok()) else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(specs_dir.join("features")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let Ok(folder) = entry.file_name().into_string() else {
            continue;
        };
        let files = list_files(&entry.path());
        if !filter(&files) {
            continue;
        }
        let Ok(spec_dir) = entry.path().canonicalize() else {
            continue;
        };
        out.push(Discovery {
            spec_dir,
            folder,
            repo_root: repo_root.clone(),
            files,
        });
    }
    out
}

/// Names of the regular files directly inside `dir`, sorted.
pub fn list_files(dir: &Path) -> Vec<String> {
    let mut files: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        // Files, and links to files (a save writes the target and keeps the link, SAV-05);
        // a broken link is not a file of the spec.
        .filter(|e| {
            e.file_type().is_ok_and(|t| t.is_file())
                || (e.file_type().is_ok_and(|t| t.is_symlink())
                    && fs::metadata(e.path()).is_ok_and(|m| m.is_file()))
        })
        .filter_map(|e| e.file_name().into_string().ok())
        // Temporary files of the write pipeline are not files of the spec (SAV-10).
        .filter(|n| !crate::write::is_quire_temp(n))
        .collect();
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "# x\n").unwrap();
    }

    fn run(paths: &[PathBuf]) -> (Vec<Discovery>, Vec<(PathBuf, ScanError)>) {
        let (mut found, mut errors) = (vec![], vec![]);
        scan(paths, &generic_filter, |e| match e {
            ScanEvent::Found(d) => found.push(d),
            ScanEvent::PathError { path, error } => errors.push((path, error)),
        });
        found.sort_by(|a, b| a.spec_dir.cmp(&b.spec_dir));
        (found, errors)
    }

    #[test]
    fn finds_spec_folders_with_primary_files_only() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        touch(&repo.join(".specs/features/a-1-one/spec.md"));
        touch(&repo.join(".specs/features/a-2-two/gates.jsonl"));
        touch(&repo.join(".specs/features/a-3-empty/image.png"));
        fs::create_dir_all(repo.join(".specs/features/a-4-nothing")).unwrap();
        touch(&repo.join(".specs/STATE.md"));
        let (found, errors) = run(&[tmp.path().to_path_buf()]);
        assert!(errors.is_empty());
        let names: Vec<&str> = found.iter().map(|d| d.folder.as_str()).collect();
        assert_eq!(names, ["a-1-one", "a-2-two"]);
        assert_eq!(found[0].repo_root, repo.canonicalize().unwrap());
        assert_eq!(found[0].files, ["spec.md"]);
    }

    #[test]
    fn skips_ignored_dirs_even_without_gitignore() {
        let tmp = tempfile::tempdir().unwrap();
        for d in IGNORED_DIRS {
            touch(
                &tmp.path()
                    .join(format!("r/{d}/x/.specs/features/decoy/spec.md")),
            );
        }
        touch(&tmp.path().join("r/.specs/features/real/spec.md"));
        let (found, _) = run(&[tmp.path().to_path_buf()]);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].folder, "real");
    }

    #[test]
    fn missing_path_is_reported_and_other_paths_continue() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("r/.specs/features/real/spec.md"));
        let missing = tmp.path().join("nope");
        let file = tmp.path().join("r/.specs/features/real/spec.md");
        let (found, errors) = run(&[missing.clone(), file.clone(), tmp.path().to_path_buf()]);
        assert_eq!(found.len(), 1);
        assert_eq!(
            errors,
            vec![
                (missing.clone(), ScanError::NotFound(missing)),
                (file.clone(), ScanError::NotADirectory(file))
            ]
        );
    }

    #[test]
    fn overlapping_paths_report_each_folder_once() {
        let tmp = tempfile::tempdir().unwrap();
        touch(&tmp.path().join("a/r/.specs/features/s/spec.md"));
        let (found, _) = run(&[
            tmp.path().to_path_buf(),
            tmp.path().join("a"),
            tmp.path().join("a/r"),
        ]);
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn handles_spaces_and_accents_in_paths() {
        let tmp = tempfile::tempdir().unwrap();
        touch(
            &tmp.path()
                .join("projetos com espaço/ação/.specs/features/acme-1-teste/spec.md"),
        );
        let (found, errors) = run(&[tmp.path().to_path_buf()]);
        assert!(errors.is_empty());
        assert_eq!(found.len(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_path_is_reported() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let locked = tmp.path().join("locked");
        fs::create_dir(&locked).unwrap();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
        // Root ignores permissions; nothing to assert in that case.
        let readable = fs::read_dir(&locked).is_ok();
        let (_, errors) = run(std::slice::from_ref(&locked));
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        if !readable {
            assert_eq!(
                errors,
                vec![(locked.clone(), ScanError::PermissionDenied(locked))]
            );
        }
    }
}
