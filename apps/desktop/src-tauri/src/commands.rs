//! Typed commands exposed to the webview (design: "Shell Tauri"). Types are exported to
//! `apps/desktop/src/lib/ipc/bindings.ts` by tauri-specta (TD-07).
//!
//! Ids cross the boundary as plain numbers (`u32` in arguments): SQLite row ids of the cache
//! stay far below that range.

use std::fs;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use quire_core::git::GitResolver;
use quire_core::index::{Index, IndexEvent, InstanceInfo, ProjectSummary, SpecSummary};
use quire_core::parse::{
    self, GateCounts, GateRun, HeaderField, Lesson, Phase, PhaseState, TaskDetail,
};
use quire_core::preset::TasksGrammar;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Manager, Runtime, State};
use tauri_specta::Event;

use crate::events::{ScanPathError, ScanPhase, ScanProgress, SpecsChanged, WatchStatus};
use crate::state::AppState;

/// Files above this size open with a warning and without syntax highlighting.
pub const LARGE_FILE_BYTES: u64 = 2 * 1024 * 1024;

/// Error of a command, or a problem shown in a view: `code` is stable and translated by the
/// UI (DEB-02); `message` is the backend detail, in English, shown only as detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CmdError {
    pub code: String,
    pub message: String,
}

pub(crate) fn err(code: &str, message: impl ToString) -> CmdError {
    CmdError {
        code: code.to_owned(),
        message: message.to_string(),
    }
}

fn lock(m: &Mutex<Index>) -> MutexGuard<'_, Index> {
    // A panic while holding the lock cannot leave the cache inconsistent (it is a cache).
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// ------------------------------------------------------------------ config

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigView {
    pub name: String,
    pub preset: Option<String>,
    pub alias: Option<String>,
    pub issue_url_base: Option<String>,
    /// Checking tasks from the app (CHK-01); absent in input means off.
    pub tasks_checkbox: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
    pub path: Option<String>,
    /// Why `config.toml` was ignored, if it was.
    pub error: Option<CmdError>,
    pub scan_paths: Vec<String>,
    pub poll_interval_ms: u32,
    pub projects: Vec<ProjectConfigView>,
    pub presets: Vec<String>,
    /// `dark`, `light` or none (dark).
    pub theme: Option<String>,
    /// UI language as written in `config.toml`; none = the system's (DEB-10).
    pub language: Option<String>,
    /// One entry per scan path, with the reason when it cannot be scanned (DSC-05).
    pub scan_path_status: Vec<ScanPathStatus>,
    /// Saving is possible: there is a config location and the current file is valid.
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ScanPathStatus {
    pub path: String,
    pub error: Option<CmdError>,
}

/// What the preferences screen saves (T23).
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInput {
    pub scan_paths: Vec<String>,
    pub poll_interval_ms: u32,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub projects: Vec<ProjectConfigView>,
}

fn config_view(state: &AppState) -> ConfigView {
    let config = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    ConfigView {
        path: state.config_path.as_ref().map(|p| p.display().to_string()),
        error: state.config_error.clone(),
        scan_paths: config
            .scan_paths
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        poll_interval_ms: u32::try_from(config.poll_interval_ms).unwrap_or(u32::MAX),
        projects: config
            .projects
            .iter()
            .map(|(name, p)| ProjectConfigView {
                name: name.clone(),
                preset: p.preset.clone(),
                alias: p.alias.clone(),
                issue_url_base: p.issue_url_base.clone(),
                tasks_checkbox: Some(p.tasks_checkbox),
            })
            .collect(),
        presets: config.preset_names().map(str::to_owned).collect(),
        theme: config.theme.map(|t| t.as_str().to_owned()),
        language: config.language.clone(),
        scan_path_status: config
            .scan_paths
            .iter()
            .map(|p| ScanPathStatus {
                path: p.display().to_string(),
                error: quire_core::scan::check_scan_path(p)
                    .err()
                    .map(|e| err(e.code(), e)),
            })
            .collect(),
        editable: state.config_path.is_some() && state.config_error.is_none(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_config(state: State<'_, AppState>) -> Result<ConfigView, CmdError> {
    Ok(config_view(&state))
}

/// Saves the preferences into `config.toml` (an explicit user action) and applies them
/// without a restart: new scan paths, presets and watch interval trigger a rescan (T23).
#[tauri::command]
#[specta::specta]
pub async fn set_config<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    settings: SettingsInput,
) -> Result<ConfigView, CmdError> {
    use quire_core::config::{ProjectConfig, Settings, Theme, save_settings};
    if let Some(e) = &state.config_error {
        // Same code as the reason it is not editable, so the UI says what to fix.
        return Err(err(
            &e.code,
            format!("fix this first, by hand: {}", e.message),
        ));
    }
    let path = state
        .config_path
        .clone()
        .ok_or_else(|| err("no_config_dir", "no configuration directory"))?;
    if !(500..=60_000).contains(&settings.poll_interval_ms) {
        return Err(err(
            "poll_interval_range",
            "the polling interval must be between 0.5 and 60 seconds",
        ));
    }
    let theme = match settings.theme.as_deref() {
        None => None,
        Some("dark") => Some(Theme::Dark),
        Some("light") => Some(Theme::Light),
        Some(other) => return Err(err("unknown_theme", format!("unknown theme {other}"))),
    };
    let clean = |v: Option<String>| v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    let current = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let next = Settings {
        scan_paths: settings
            .scan_paths
            .iter()
            .map(|p| p.trim().to_owned())
            .filter(|p| !p.is_empty())
            .collect(),
        poll_interval_ms: u64::from(settings.poll_interval_ms),
        theme,
        language: clean(settings.language),
        projects: settings
            .projects
            .into_iter()
            .map(|p| {
                let config = ProjectConfig {
                    preset: clean(p.preset),
                    alias: clean(p.alias),
                    issue_url_base: clean(p.issue_url_base),
                    // Not sent: keep what config.toml has (an older UI must not turn it off).
                    tasks_checkbox: p.tasks_checkbox.unwrap_or_else(|| {
                        current
                            .projects
                            .get(&p.name)
                            .is_some_and(|c| c.tasks_checkbox)
                    }),
                };
                (p.name, config)
            })
            .collect(),
    };
    let config = save_settings(&path, &next).map_err(|e| err(e.code(), &e))?;
    *state
        .config
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = config;
    crate::live::request_scan(&app, Vec::new()).map_err(|e| err("scan", e))?;
    Ok(config_view(&state))
}

// ------------------------------------------------------------------ scan

/// Starts a background scan and index reconciliation; progress arrives as `scan://progress`
/// events and the list as `specs://changed` (DSC-04). A call during a scan queues one rerun.
#[tauri::command]
#[specta::specta]
pub async fn start_scan<R: Runtime>(app: AppHandle<R>) -> Result<(), CmdError> {
    crate::live::request_scan(&app, Vec::new()).map_err(|e| err("scan", e))
}

/// Navigation asked on the command line that the UI has not handled yet (also sent as
/// `cli://open`); taking it clears it.
#[tauri::command]
#[specta::specta]
pub async fn take_cli_open<R: Runtime>(
    app: AppHandle<R>,
) -> Result<Option<crate::cli::CliOpen>, CmdError> {
    Ok(crate::cli_delivery::take(&app))
}

/// What the file watcher is doing right now (also sent as `watch://status`).
#[tauri::command]
#[specta::specta]
pub async fn get_watch_status(state: State<'_, AppState>) -> Result<WatchStatus, CmdError> {
    Ok(state
        .live
        .status
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone())
}

/// One reconciliation; scheduling and the watcher live in `crate::live`.
pub(crate) fn run_scan<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let config = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let mut progress = ScanProgress {
        phase: ScanPhase::Scanning,
        found: 0,
        processed: 0,
        total: 0,
        errors: vec![],
        message: None,
    };
    let _ = progress.emit(app);
    let result = Index::open(&state.index_path).and_then(|mut index| {
        index.reconcile(
            &config.scan_paths,
            &config,
            &mut GitResolver::new(),
            |event| match event {
                IndexEvent::Found { count } => {
                    progress.found = u32::try_from(count).unwrap_or(u32::MAX);
                    if count % 25 == 0 {
                        let _ = progress.emit(app);
                    }
                }
                IndexEvent::PathError { path, error } => {
                    progress.errors.push(ScanPathError {
                        path: path.display().to_string(),
                        code: error.code().to_owned(),
                        message: error.to_string(),
                    });
                    let _ = progress.emit(app);
                }
                IndexEvent::Progress { processed, total } => {
                    progress.phase = ScanPhase::Indexing;
                    progress.processed = u32::try_from(processed).unwrap_or(u32::MAX);
                    progress.total = u32::try_from(total).unwrap_or(u32::MAX);
                    let _ = progress.emit(app);
                    let _ = SpecsChanged.emit(app);
                }
            },
        )
    });
    match result {
        Ok(_) => progress.phase = ScanPhase::Done,
        Err(e) => {
            progress.phase = ScanPhase::Failed;
            progress.message = Some(e.to_string());
        }
    }
    let _ = progress.emit(app);
    let _ = SpecsChanged.emit(app);
}

// ------------------------------------------------------------------ list and spec

#[derive(Debug, Clone, Default, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ListFilter {
    /// Case-insensitive match on folder, issue key or title.
    pub text: Option<String>,
}

fn matches(spec: &SpecSummary, needle: &str) -> bool {
    let hit = |s: &str| s.to_lowercase().contains(needle);
    hit(&spec.folder)
        || spec.issue_key.as_deref().is_some_and(hit)
        || spec
            .default_instance()
            .and_then(|i| i.title.as_deref())
            .is_some_and(hit)
}

/// Projects and specs from the index cache (fast, IDX-03).
#[tauri::command]
#[specta::specta]
pub async fn list_projects(
    state: State<'_, AppState>,
    filter: Option<ListFilter>,
) -> Result<Vec<ProjectSummary>, CmdError> {
    let mut list = lock(&state.index).list().map_err(|e| err("index", e))?;
    if let Some(needle) = filter
        .and_then(|f| f.text)
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
    {
        for p in &mut list {
            p.specs.retain(|s| matches(s, &needle));
        }
        list.retain(|p| !p.specs.is_empty());
    }
    Ok(list)
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRef {
    pub id: u32,
    pub name: String,
    pub display_name: String,
    pub preset: String,
    pub issue_url_base: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub size: u32,
    pub mtime_ms: Option<f64>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SpecDetail {
    pub project: ProjectRef,
    pub spec: SpecSummary,
    /// Index into `spec.instances` of the instance shown.
    pub instance_index: u32,
    pub files: Vec<FileEntry>,
    /// Primary files present, in tab order (VIW-01).
    pub tabs: Vec<String>,
    /// Every other file of the folder, for the "outros" tab.
    pub others: Vec<String>,
    /// `**Chave:** valor` lines of the preset's header files (VIW-07).
    pub header: Vec<HeaderField>,
    /// Files whose header block is shown as metadata instead of body text.
    pub header_files: Vec<String>,
}

fn id(n: u32) -> i64 {
    i64::from(n)
}

fn to_u32(n: i64) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// One spec with the requested instance, or the default one (DUP-04).
#[tauri::command]
#[specta::specta]
pub async fn get_spec(
    state: State<'_, AppState>,
    spec_id: u32,
    instance_id: Option<u32>,
) -> Result<SpecDetail, CmdError> {
    let (project, spec, info) = {
        let index = lock(&state.index);
        let list = index.list().map_err(|e| err("index", e))?;
        let (project, spec) = list
            .into_iter()
            .find_map(|p| {
                let s = p.specs.iter().find(|s| s.id == id(spec_id)).cloned()?;
                Some((p, s))
            })
            .ok_or_else(|| err("not_found", format!("spec {spec_id} not found")))?;
        let instance_index = match instance_id {
            Some(i) => spec
                .instances
                .iter()
                .position(|x| x.id == id(i))
                .ok_or_else(|| {
                    err(
                        "not_found",
                        format!("instance {i} is not part of spec {spec_id}"),
                    )
                })?,
            None => spec.default_index,
        };
        let inst_id = spec
            .instances
            .get(instance_index)
            .map(|i| i.id)
            .ok_or_else(|| err("not_found", "no instance"))?;
        let info = index
            .instance_info(inst_id)
            .map_err(|e| err("index", e))?
            .ok_or_else(|| err("not_found", "instance not found"))?;
        (project, spec, (instance_index, info))
    };
    let (instance_index, info) = info;
    let config = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let preset = config.preset_for(&project.name);
    let names: Vec<String> = info.files.iter().map(|f| f.0.clone()).collect();
    let tabs = preset.tabs(&names);
    let others = names
        .iter()
        .filter(|n| !tabs.contains(n))
        .cloned()
        .collect();
    let mut header = Vec::new();
    for h in preset.header_files.iter().filter(|h| names.contains(h)) {
        if let Ok(text) = fs::read_to_string(info.path.join(h)) {
            for field in parse::header(&text) {
                if !header
                    .iter()
                    .any(|f: &HeaderField| f.key.eq_ignore_ascii_case(&field.key))
                {
                    header.push(field);
                }
            }
        }
    }
    Ok(SpecDetail {
        project: ProjectRef {
            id: to_u32(project.id),
            display_name: project.display_name().to_owned(),
            issue_url_base: config
                .projects
                .get(&project.name)
                .and_then(|p| p.issue_url_base.clone()),
            name: project.name.clone(),
            preset: project.preset.clone(),
        },
        files: info
            .files
            .iter()
            .map(|(name, size, mtime)| FileEntry {
                name: name.clone(),
                size: u32::try_from(*size).unwrap_or(u32::MAX),
                mtime_ms: mtime.map(|m| m as f64),
                primary: preset.is_primary(name),
            })
            .collect(),
        spec,
        instance_index: u32::try_from(instance_index).unwrap_or(0),
        tabs,
        others,
        header,
        header_files: preset.header_files.clone(),
    })
}

pub(crate) fn instance(state: &AppState, instance_id: u32) -> Result<InstanceInfo, CmdError> {
    lock(&state.index)
        .instance_info(id(instance_id))
        .map_err(|e| err("index", e))?
        .ok_or_else(|| err("not_found", format!("instance {instance_id} not found")))
}

/// Reads a file of an instance. Only names recorded for that instance are accepted, so no
/// path can escape the spec folder.
pub(crate) fn read_known(info: &InstanceInfo, name: &str) -> Result<Option<Vec<u8>>, CmdError> {
    if !info.files.iter().any(|f| f.0 == name) || Path::new(name).components().count() != 1 {
        return Err(err(
            "not_found",
            format!("{name} is not a file of this spec"),
        ));
    }
    match fs::read(info.path.join(name)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err("io", e)),
    }
}

// ------------------------------------------------------------------ files

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FileKind {
    Markdown,
    Json,
    Jsonl,
    Text,
    Binary,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    pub name: String,
    pub kind: FileKind,
    pub size: u32,
    /// `None` for binary or non-UTF-8 files (VIW-08).
    pub text: Option<String>,
    /// Above 2 MB: shown with a warning and without highlighting.
    pub large: bool,
}

fn kind_of(name: &str) -> FileKind {
    match name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("md" | "markdown") => FileKind::Markdown,
        Some("json") => FileKind::Json,
        Some("jsonl") => FileKind::Jsonl,
        _ => FileKind::Text,
    }
}

#[tauri::command]
#[specta::specta]
pub async fn read_file(
    state: State<'_, AppState>,
    instance_id: u32,
    name: String,
) -> Result<FileContent, CmdError> {
    let info = instance(&state, instance_id)?;
    let bytes =
        read_known(&info, &name)?.ok_or_else(|| err("gone", format!("{name} was removed")))?;
    let size = bytes.len() as u64;
    let looks_binary = bytes.iter().take(8192).any(|b| *b == 0);
    let text = if looks_binary {
        None
    } else {
        String::from_utf8(bytes).ok()
    };
    Ok(FileContent {
        kind: if text.is_some() {
            kind_of(&name)
        } else {
            FileKind::Binary
        },
        name,
        size: u32::try_from(size).unwrap_or(u32::MAX),
        text,
        large: size > LARGE_FILE_BYTES,
    })
}

// ------------------------------------------------------------------ tasks, gates, lessons

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PhaseView {
    #[serde(flatten)]
    pub phase: Phase,
    pub done: u32,
    pub total: u32,
    pub state: PhaseState,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TasksView {
    /// `false` when the project's preset has no tasks grammar or there is no `tasks.md`.
    pub structured: bool,
    pub phases: Vec<PhaseView>,
    pub details: Vec<TaskDetail>,
    pub done: u32,
    pub total: u32,
    /// Fingerprint token of the `tasks.md` this view was built from: the base of a checkbox
    /// write (CHK-03).
    pub fingerprint: Option<String>,
    /// The project opted in to checking tasks from the app (CHK-01).
    pub checkbox: bool,
    /// Why the checkboxes are disabled even though the project opted in (SAV-11).
    pub locked: Option<CmdError>,
}

/// `tasks.md` by phase (TSK-01..04).
#[tauri::command]
#[specta::specta]
pub async fn get_tasks(
    state: State<'_, AppState>,
    instance_id: u32,
) -> Result<TasksView, CmdError> {
    let info = instance(&state, instance_id)?;
    tasks_view(&state, &info)
}

/// The tasks view of an instance, read from disk now.
pub(crate) fn tasks_view(state: &AppState, info: &InstanceInfo) -> Result<TasksView, CmdError> {
    let bytes = match info.files.iter().any(|f| f.0 == "tasks.md") {
        true => read_known(info, "tasks.md")?,
        false => None,
    };
    Ok(tasks_view_of(state, info, bytes.as_deref()))
}

/// The tasks view of `bytes` as the `tasks.md` of `info` (also used right after a write, so
/// progress updates without waiting for the watcher, PRG-01).
pub(crate) fn tasks_view_of(
    state: &AppState,
    info: &InstanceInfo,
    bytes: Option<&[u8]>,
) -> TasksView {
    let config = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let preset = config.preset_for(&info.project_name);
    let checkbox = config
        .projects
        .get(&info.project_name)
        .is_some_and(|p| p.tasks_checkbox);
    let fingerprint = bytes.map(|b| quire_core::fingerprint::Fingerprint::of_bytes(b).token());
    let class = quire_core::fs_kind::classify(&info.path.join("tasks.md"));
    let locked = (checkbox && !quire_core::fs_kind::writable(class)).then(|| {
        err(
            "write_unreliable_fs",
            format!(
                "writes are disabled on this file system ({})",
                class.as_str()
            ),
        )
    });
    let text = bytes.map(|b| String::from_utf8_lossy(b).into_owned());
    let doc = match (preset.tasks, text) {
        (TasksGrammar::Phases, Some(t)) => parse::tasks(&t, preset.id_pattern.as_ref()),
        _ => parse::TasksDoc::default(),
    };
    let (done, total) = doc.progress();
    TasksView {
        structured: !doc.is_empty(),
        phases: doc
            .phases
            .into_iter()
            .map(|p| PhaseView {
                done: u32::try_from(p.done()).unwrap_or(u32::MAX),
                total: u32::try_from(p.total()).unwrap_or(u32::MAX),
                state: p.state(),
                phase: p,
            })
            .collect(),
        details: doc.details,
        done: u32::try_from(done).unwrap_or(u32::MAX),
        total: u32::try_from(total).unwrap_or(u32::MAX),
        fingerprint,
        checkbox,
        locked,
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct GatesView {
    /// Most recent first (LOG-01).
    pub runs: Vec<GateRun>,
    pub counts: GateCounts,
    /// Lines that could not be read (LOG-04).
    pub skipped: u32,
    pub skipped_lines: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct LessonsView {
    pub lessons: Vec<Lesson>,
    pub skipped: u32,
    pub skipped_lines: Vec<u32>,
}

fn jsonl_text(state: &AppState, instance_id: u32, name: &str) -> Result<String, CmdError> {
    let info = instance(state, instance_id)?;
    if !info.files.iter().any(|f| f.0 == name) {
        return Ok(String::new());
    }
    Ok(read_known(&info, name)?
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default())
}

fn lines_u32(v: &[usize]) -> Vec<u32> {
    v.iter()
        .map(|n| u32::try_from(*n).unwrap_or(u32::MAX))
        .collect()
}

#[tauri::command]
#[specta::specta]
pub async fn get_gates(
    state: State<'_, AppState>,
    instance_id: u32,
) -> Result<GatesView, CmdError> {
    let parsed: parse::Jsonl<GateRun> =
        parse::jsonl(&jsonl_text(&state, instance_id, "gates.jsonl")?);
    let mut runs = parsed.items;
    parse::newest_first(&mut runs);
    Ok(GatesView {
        counts: parse::gate_counts(&runs),
        runs,
        skipped: u32::try_from(parsed.skipped).unwrap_or(u32::MAX),
        skipped_lines: lines_u32(&parsed.skipped_lines),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_lessons(
    state: State<'_, AppState>,
    instance_id: u32,
) -> Result<LessonsView, CmdError> {
    let parsed: parse::Jsonl<Lesson> =
        parse::jsonl(&jsonl_text(&state, instance_id, "lessons.jsonl")?);
    Ok(LessonsView {
        lessons: parsed.items,
        skipped: u32::try_from(parsed.skipped).unwrap_or(u32::MAX),
        skipped_lines: lines_u32(&parsed.skipped_lines),
    })
}

// ------------------------------------------------------------------ external links

/// Opens an http(s) link in the system browser (VIW-07). Nothing else is accepted.
#[tauri::command]
#[specta::specta]
pub async fn open_external<R: Runtime>(app: AppHandle<R>, url: String) -> Result<(), CmdError> {
    let lower = url.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(err(
            "invalid_url",
            "only http and https links can be opened",
        ));
    }
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| err("open", e))
}
