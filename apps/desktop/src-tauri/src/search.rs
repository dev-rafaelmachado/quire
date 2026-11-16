//! Full-text search over IPC (SRC-03..07): the sections that match, grouped by spec, with
//! identical copies shown once (SRC-04).

use std::collections::{HashMap, HashSet};

use quire_core::search::{DEFAULT_LIMIT, SearchQuery};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;

use crate::commands::{CmdError, err};
use crate::state::AppState;

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchInput {
    pub text: String,
    pub project_id: Option<u32>,
    /// File name, e.g. `spec.md`.
    pub file: Option<String>,
    /// Only the open spec (SRC-05).
    pub spec_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    /// The instance to open: the default copy when the section is in it, else the first copy
    /// that holds it (design, "Agrupamento").
    pub instance_id: u32,
    pub file: String,
    pub heading: String,
    /// First source line of the section (SRC-06).
    pub line: u32,
    /// Excerpt with matches between U+0002 and U+0003; the UI turns them into `<mark>` and
    /// never renders HTML from here.
    pub snippet: String,
    /// Branch of the copy, when the spec has several (divergent copies can match twice).
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroup {
    pub spec_id: u32,
    pub folder: String,
    pub issue_key: Option<String>,
    pub title: Option<String>,
    pub project_id: u32,
    pub project: String,
    pub hits: Vec<SearchHit>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub groups: Vec<SearchGroup>,
    /// The limit was reached: the UI asks to refine (spec, edge cases).
    pub truncated: bool,
    /// The terms as searched, for highlighting the opened file.
    pub terms: Vec<String>,
}

fn id32(v: i64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    state: State<'_, AppState>,
    input: SearchInput,
) -> Result<SearchResults, CmdError> {
    let index = state
        .index
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let rows = index
        .search(&SearchQuery {
            text: input.text.clone(),
            project_id: input.project_id.map(i64::from),
            file: input.file.clone().filter(|f| !f.is_empty()),
            spec_id: input.spec_id.map(i64::from),
            limit: Some(DEFAULT_LIMIT * 4),
        })
        .map_err(|e| err("index", e))?;
    let list = index.list().map_err(|e| err("index", e))?;
    drop(index);

    // Spec facts from the list (which copy is the default, titles, project names).
    let mut specs = HashMap::new();
    for p in &list {
        for s in &p.specs {
            let default = s.instances.get(s.default_index).map(|i| i.id);
            let title = s
                .instances
                .get(s.default_index)
                .and_then(|i| i.title.clone());
            specs.insert(s.id, (p, s, default, title));
        }
    }
    // Copies of each section of each spec, to prefer the default copy.
    let mut holders: HashMap<(i64, i64, String), Vec<i64>> = HashMap::new();
    for r in &rows {
        holders
            .entry((r.spec_id, r.section, r.file.clone()))
            .or_default()
            .push(r.instance_id);
    }
    let mut seen: HashSet<(i64, i64, String)> = HashSet::new();
    let mut groups: Vec<SearchGroup> = Vec::new();
    let mut at: HashMap<i64, usize> = HashMap::new();
    let mut hits = 0;
    for r in &rows {
        let key = (r.spec_id, r.section, r.file.clone());
        if !seen.insert(key.clone()) {
            continue; // an identical copy of a section already listed (SRC-04)
        }
        let Some((project, spec, default, title)) = specs.get(&r.spec_id) else {
            continue;
        };
        if hits >= DEFAULT_LIMIT {
            break;
        }
        hits += 1;
        let copies = holders.get(&key).map(Vec::as_slice).unwrap_or(&[]);
        let instance = default
            .filter(|d| copies.contains(d))
            .or_else(|| copies.first().copied())
            .unwrap_or(r.instance_id);
        let g = *at.entry(r.spec_id).or_insert_with(|| {
            groups.push(SearchGroup {
                spec_id: id32(r.spec_id),
                folder: spec.folder.clone(),
                issue_key: spec.issue_key.clone(),
                title: title.clone(),
                project_id: id32(project.id),
                project: project
                    .alias
                    .clone()
                    .unwrap_or_else(|| project.name.clone()),
                hits: vec![],
            });
            groups.len() - 1
        });
        if let Some(group) = groups.get_mut(g) {
            let branch = (spec.instances.len() > 1)
                .then(|| spec.instances.iter().find(|i| i.id == instance))
                .flatten()
                .map(|i| i.branch.clone().unwrap_or_else(|| "HEAD".to_owned()));
            group.hits.push(SearchHit {
                instance_id: id32(instance),
                file: r.file.clone(),
                heading: r.heading.clone(),
                line: u32::try_from(r.line).unwrap_or(0),
                snippet: r.snippet.clone(),
                branch,
            });
        }
    }
    let unique: HashSet<_> = rows
        .iter()
        .map(|r| (r.spec_id, r.section, r.file.clone()))
        .collect();
    Ok(SearchResults {
        groups,
        truncated: unique.len() > hits,
        terms: input
            .text
            .split_whitespace()
            .filter(|t| t.chars().any(char::is_alphanumeric))
            .map(str::to_owned)
            .collect(),
    })
}
