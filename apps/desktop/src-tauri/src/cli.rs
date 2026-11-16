//! Command line (TD-10): `quire`, `quire .`, `quire open <key|path>`, `--help`, `--version`.
//!
//! The grammar lives in `tauri.conf.json` (`plugins.cli`) and is parsed by tauri-plugin-cli,
//! both for this process and for the argv a second instance forwards (CLI-04). A target is
//! resolved against the index into what the UI should do (CLI-01..03).

use std::path::{Path, PathBuf};

use quire_core::index::ProjectSummary;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use tauri_plugin_cli::Matches;

/// What was asked on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliRequest {
    Help(String),
    Version,
    /// `quire` with no target.
    Nothing,
    /// `quire <dir>`, e.g. `quire .` (CLI-01).
    Here(PathBuf),
    /// `quire open <key|path>` (CLI-02, CLI-03).
    Open {
        query: String,
        cwd: PathBuf,
    },
}

fn string_arg(m: &Matches, name: &str) -> Option<String> {
    match m.args.get(name).map(|a| &a.value) {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        _ => None,
    }
}

/// Interprets plugin-cli matches, with `cwd` the directory the command ran in.
pub fn interpret(matches: &Matches, cwd: &Path) -> CliRequest {
    if let Some(Value::String(help)) = matches.args.get("help").map(|a| &a.value) {
        return CliRequest::Help(help.clone());
    }
    if matches.args.contains_key("version") {
        return CliRequest::Version;
    }
    if let Some(sub) = &matches.subcommand {
        if let Some(Value::String(help)) = sub.matches.args.get("help").map(|a| &a.value) {
            return CliRequest::Help(help.clone());
        }
        if sub.name == "open" {
            return match string_arg(&sub.matches, "target") {
                Some(query) => CliRequest::Open {
                    query,
                    cwd: cwd.to_path_buf(),
                },
                None => CliRequest::Nothing,
            };
        }
    }
    match string_arg(matches, "path") {
        Some(p) => CliRequest::Here(cwd.join(p)),
        None => CliRequest::Nothing,
    }
}

/// What the UI should do (event `cli://open`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CliOpen {
    /// Open this spec, in this instance when given, on this file's tab when given.
    Spec {
        spec_id: u32,
        instance_id: Option<u32>,
        file: Option<String>,
    },
    /// `quire .` in a worktree with several specs: the project list, these highlighted.
    Highlight {
        query: String,
        spec_ids: Vec<u32>,
    },
    /// The key matches specs in more than one place: let the user pick (CLI-02).
    Pick {
        query: String,
        spec_ids: Vec<u32>,
    },
    NotFound {
        query: String,
    },
}

fn id(n: i64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn canon(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

/// Spec folder that contains `path`, or the specs of the deepest worktree that contains it.
fn by_path(list: &[ProjectSummary], path: &Path, query: &str) -> CliOpen {
    let path = canon(path);
    for p in list {
        for s in &p.specs {
            if let Some(i) = s.instances.iter().find(|i| path.starts_with(&i.path)) {
                // A file directly inside the spec folder opens on its tab.
                let file = path
                    .strip_prefix(&i.path)
                    .ok()
                    .filter(|rest| rest.components().count() == 1)
                    .map(|rest| rest.to_string_lossy().into_owned());
                return CliOpen::Spec {
                    spec_id: id(s.id),
                    instance_id: Some(id(i.id)),
                    file,
                };
            }
        }
    }
    let root = list
        .iter()
        .flat_map(|p| {
            p.specs
                .iter()
                .flat_map(|s| s.instances.iter().map(|i| i.repo_root.clone()))
        })
        .filter(|r| path.starts_with(r))
        .max_by_key(|r| r.components().count());
    let Some(root) = root else {
        return CliOpen::NotFound {
            query: query.to_owned(),
        };
    };
    let mut hits: Vec<(u32, u32)> = list
        .iter()
        .flat_map(|p| p.specs.iter())
        .filter_map(|s| {
            s.instances
                .iter()
                .find(|i| i.repo_root == root)
                .map(|i| (id(s.id), id(i.id)))
        })
        .collect();
    hits.sort_unstable();
    match hits.as_slice() {
        [] => CliOpen::NotFound {
            query: query.to_owned(),
        },
        [(spec_id, instance_id)] => CliOpen::Spec {
            spec_id: *spec_id,
            instance_id: Some(*instance_id),
            file: None,
        },
        many => CliOpen::Highlight {
            query: query.to_owned(),
            spec_ids: many.iter().map(|h| h.0).collect(),
        },
    }
}

/// Specs whose folder starts with the key, at a word boundary (`acme-14` does not match
/// `acme-142-…`).
fn by_key(list: &[ProjectSummary], key: &str) -> CliOpen {
    let k = key.to_lowercase();
    let mut ids: Vec<u32> = list
        .iter()
        .flat_map(|p| p.specs.iter())
        .filter(|s| {
            let f = s.folder.to_lowercase();
            f.strip_prefix(&k)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'))
        })
        .map(|s| id(s.id))
        .collect();
    ids.sort_unstable();
    match ids.as_slice() {
        [] => CliOpen::NotFound {
            query: key.to_owned(),
        },
        [only] => CliOpen::Spec {
            spec_id: *only,
            instance_id: None,
            file: None,
        },
        _ => CliOpen::Pick {
            query: key.to_owned(),
            spec_ids: ids,
        },
    }
}

fn looks_like_path(query: &str, cwd: &Path) -> bool {
    query.contains('/')
        || query.contains('\\')
        || query.starts_with('.')
        || cwd.join(query).exists()
}

/// Resolves a request against the index list. `None` for requests that open nothing.
pub fn resolve(list: &[ProjectSummary], request: &CliRequest) -> Option<CliOpen> {
    match request {
        CliRequest::Here(dir) => Some(by_path(list, dir, &dir.display().to_string())),
        CliRequest::Open { query, cwd } if looks_like_path(query, cwd) => {
            Some(by_path(list, &cwd.join(query), query))
        }
        CliRequest::Open { query, .. } => Some(by_key(list, query)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use quire_core::index::{InstanceSummary, SpecSummary};
    use quire_core::model::InstanceKind;

    use super::*;

    fn inst(id: i64, root: &str, folder: &str) -> InstanceSummary {
        InstanceSummary {
            id,
            path: PathBuf::from(format!("{root}/.specs/features/{folder}")),
            repo_root: PathBuf::from(root),
            branch: Some("main".into()),
            kind: InstanceKind::Checkout,
            mtime_ms: Some(1),
            same_as_default: true,
            content_hash: "h".into(),
            title: None,
            status: None,
            issue_url: None,
            progress: None,
        }
    }

    fn spec(id: i64, folder: &str, instances: Vec<InstanceSummary>) -> SpecSummary {
        SpecSummary {
            id,
            folder: folder.into(),
            issue_key: None,
            instances,
            default_index: 0,
            versions_badge: None,
        }
    }

    fn list() -> Vec<ProjectSummary> {
        let gw = |id, folder: &str, extra: Vec<InstanceSummary>| {
            let mut v = vec![inst(id * 10, "/ws/gw", folder)];
            v.extend(extra);
            spec(id, folder, v)
        };
        vec![
            ProjectSummary {
                id: 1,
                key: "/ws/gw/.git".into(),
                name: "acme_gateway".into(),
                alias: None,
                preset: "exban".into(),
                suggested_preset: None,
                git_warning: None,
                specs: vec![
                    gw(
                        1,
                        "acme-142-tokens",
                        vec![inst(11, "/wt/acme-170", "acme-142-tokens")],
                    ),
                    gw(
                        2,
                        "acme-170-rate",
                        vec![inst(21, "/wt/acme-170", "acme-170-rate")],
                    ),
                    gw(3, "acme-200-auth", vec![]),
                ],
            },
            ProjectSummary {
                id: 2,
                key: "/ws/bill/.git".into(),
                name: "acme_billing".into(),
                alias: None,
                preset: "exban".into(),
                suggested_preset: None,
                git_warning: None,
                specs: vec![
                    spec(
                        4,
                        "acme-200-auth",
                        vec![inst(40, "/ws/bill", "acme-200-auth")],
                    ),
                    spec(5, "bill-31-x", vec![inst(50, "/ws/bill", "bill-31-x")]),
                ],
            },
        ]
    }

    fn open(q: &str) -> CliRequest {
        CliRequest::Open {
            query: q.into(),
            cwd: PathBuf::from("/nowhere"),
        }
    }

    #[test]
    fn keys_resolve_to_one_spec_or_a_picker() {
        assert_eq!(
            resolve(&list(), &open("ACME-142")),
            Some(CliOpen::Spec {
                spec_id: 1,
                instance_id: None,
                file: None
            })
        );
        assert_eq!(
            resolve(&list(), &open("bill-31")),
            Some(CliOpen::Spec {
                spec_id: 5,
                instance_id: None,
                file: None
            })
        );
        assert_eq!(
            resolve(&list(), &open("acme-200")),
            Some(CliOpen::Pick {
                query: "acme-200".into(),
                spec_ids: vec![3, 4]
            })
        );
        assert_eq!(
            resolve(&list(), &open("acme-14")),
            Some(CliOpen::NotFound {
                query: "acme-14".into()
            })
        );
    }

    #[test]
    fn paths_open_the_spec_folder_or_the_worktree_specs() {
        let here = |p: &str| CliRequest::Here(PathBuf::from(p));
        assert_eq!(
            resolve(
                &list(),
                &here("/ws/bill/.specs/features/bill-31-x/tasks.md")
            ),
            Some(CliOpen::Spec {
                spec_id: 5,
                instance_id: Some(50),
                file: Some("tasks.md".into())
            })
        );
        assert_eq!(
            resolve(&list(), &here("/wt/acme-170/src")),
            Some(CliOpen::Highlight {
                query: "/wt/acme-170/src".into(),
                spec_ids: vec![1, 2]
            })
        );
        assert_eq!(
            resolve(&list(), &here("/elsewhere")),
            Some(CliOpen::NotFound {
                query: "/elsewhere".into()
            })
        );
        let path = CliRequest::Open {
            query: "/ws/gw/.specs/features/acme-170-rate".into(),
            cwd: PathBuf::from("/"),
        };
        assert_eq!(
            resolve(&list(), &path),
            Some(CliOpen::Spec {
                spec_id: 2,
                instance_id: Some(20),
                file: None
            })
        );
        assert_eq!(resolve(&list(), &CliRequest::Nothing), None);
    }
}
