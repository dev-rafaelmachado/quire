//! Write commands (design: "Shell Tauri: comandos e eventos novos"). Each one is an explicit
//! user action in the UI (SAV-07): nothing here is called on a timer, on navigation or on a
//! reload. All writes go through `quire_core::write` (TD-12); after one, the spec folder is
//! reconciled right away so the list, the index and the progress follow without waiting for
//! the watcher (PRG-01, INS-03).

use quire_core::fingerprint::Fingerprint;
use quire_core::index::InstanceInfo;
use quire_core::write::{self, NotEditable, TextFormat, WriteError};
use quire_core::{fs_kind, tasks_edit};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Runtime, State};

use crate::commands::{CmdError, TasksView, err, instance, tasks_view_of};
use crate::state::AppState;

fn write_err(e: &WriteError) -> CmdError {
    err(e.code(), e)
}

fn listed(info: &InstanceInfo) -> Vec<String> {
    info.files.iter().map(|f| f.0.clone()).collect()
}

fn fingerprint_arg(token: &str) -> Result<Fingerprint, CmdError> {
    Fingerprint::from_token(token).ok_or_else(|| err("invalid", format!("bad fingerprint {token}")))
}

/// Reconciles the folder of `info` now (index, list, progress) instead of waiting for the
/// watcher. A failure to start it is not a failed write.
fn after_write<R: Runtime>(app: &AppHandle<R>, info: &InstanceInfo) {
    let _ = crate::live::request_scan(app, vec![info.path.clone()]);
}

/// A Markdown file opened for editing (EDT-01, EDT-08, CNF-01).
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct EditableFile {
    pub name: String,
    /// The text as the editor shows it (LF line endings, no BOM); `None` when locked by the
    /// content (not UTF-8, binary, too large).
    pub text: Option<String>,
    pub format: Option<TextFormat>,
    /// Fingerprint token of the bytes read: the base of the next save.
    pub fingerprint: Option<String>,
    /// Why Editar and Dividir are disabled (EDT-08), as a translatable code; `None` when the
    /// file is editable.
    pub locked: Option<CmdError>,
    /// The file is read-only: editing is allowed, saving offers a copy (SAV-04).
    pub read_only: bool,
}

fn locked_by_name(name: &str) -> Option<CmdError> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".jsonl") {
        Some(err("edit_jsonl", format!("{name} is written by agents")))
    } else if !lower.ends_with(".md") {
        Some(err("edit_not_markdown", format!("{name} is not Markdown")))
    } else {
        None
    }
}

fn locked_by_content(e: NotEditable) -> CmdError {
    let code = match e {
        NotEditable::NotUtf8 => "edit_not_utf8",
        NotEditable::Binary => "edit_binary",
        NotEditable::TooLarge => "edit_too_large",
    };
    err(code, e)
}

#[tauri::command]
#[specta::specta]
pub async fn open_for_edit(
    state: State<'_, AppState>,
    instance_id: u32,
    name: String,
) -> Result<EditableFile, CmdError> {
    let info = instance(&state, instance_id)?;
    let bytes = crate::commands::read_known(&info, &name)?
        .ok_or_else(|| err("gone", format!("{name} was removed")))?;
    let path = info.path.join(&name);
    let mut out = EditableFile {
        name: name.clone(),
        text: None,
        format: None,
        fingerprint: Some(Fingerprint::of_bytes(&bytes).token()),
        locked: locked_by_name(&name),
        read_only: std::fs::metadata(&path).is_ok_and(|m| m.permissions().readonly()),
    };
    if out.locked.is_some() {
        return Ok(out);
    }
    match write::decode(&bytes) {
        Ok((text, format)) => {
            out.text = Some(text);
            out.format = Some(format);
        }
        Err(e) => {
            out.locked = Some(locked_by_content(e));
            return Ok(out);
        }
    }
    let class = fs_kind::classify(&path);
    if !fs_kind::writable(class) {
        out.locked = Some(write_err(&WriteError::UnreliableFs(
            class.as_str().to_owned(),
        )));
    }
    Ok(out)
}

/// The fingerprint token of a file of the instance now, `None` when it does not exist. The UI
/// compares it with its base when the watcher reports a change (TD-14, CNF-04..06).
#[tauri::command]
#[specta::specta]
pub async fn file_fingerprint(
    state: State<'_, AppState>,
    instance_id: u32,
    name: String,
) -> Result<Option<String>, CmdError> {
    let info = instance(&state, instance_id)?;
    Ok(crate::commands::read_known(&info, &name)?.map(|b| Fingerprint::of_bytes(&b).token()))
}

#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SaveInput {
    pub instance_id: u32,
    pub name: String,
    pub text: String,
    pub format: TextFormat,
    /// The fingerprint the buffer was loaded from. `None` re-creates a file deleted during the
    /// edit, after the user confirmed it (CNF-07); it never replaces an existing file.
    pub base: Option<String>,
    /// The user confirmed normalizing mixed line endings (SAV-03, A4).
    pub confirm_mixed: bool,
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Saved {
    /// The new base of the buffer.
    pub fingerprint: String,
    /// The format now on disk (mixed line endings are gone after a save).
    pub format: TextFormat,
}

/// Saves the buffer (EDT-05, SAV-01..05, SAV-08, CNF-02, CNF-03). Conflicts come back as the
/// error code `write_conflict`; nothing is written then.
#[tauri::command]
#[specta::specta]
pub async fn save_file<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    input: SaveInput,
) -> Result<Saved, CmdError> {
    let info = instance(&state, input.instance_id)?;
    if input.format.mixed && !input.confirm_mixed {
        return Err(err(
            "write_mixed_eol",
            "the file mixes line endings; confirm the normalization first",
        ));
    }
    let bytes = write::encode(&input.text, &input.format);
    let written = match &input.base {
        Some(base) => {
            let base = fingerprint_arg(base)?;
            let path = write::editable_path(&info.path, &input.name, &listed(&info))
                .map_err(|e| write_err(&e))?;
            write::save_over(&path, &bytes, &base)
        }
        None => {
            // The file is gone from disk and maybe from the index: the name only has to be a
            // Markdown name of this folder.
            let path =
                write::editable_path(&info.path, &input.name, std::slice::from_ref(&input.name))
                    .map_err(|e| write_err(&e))?;
            write::save_new(&path, &bytes)
        }
    }
    .map_err(|e| write_err(&e))?;
    after_write(&app, &info);
    Ok(Saved {
        fingerprint: written.token(),
        format: TextFormat {
            mixed: false,
            ..input.format
        },
    })
}

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SavedCopy {
    pub name: String,
    pub fingerprint: String,
}

/// "Salvar como cópia" (CNF-08, SAV-04): a new file next to the original, never over one.
#[tauri::command]
#[specta::specta]
pub async fn save_copy<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    instance_id: u32,
    name: String,
    text: String,
    format: TextFormat,
    stamp: String,
) -> Result<SavedCopy, CmdError> {
    let info = instance(&state, instance_id)?;
    // Same name rules as a save, against the folder's own list (the original may be gone).
    write::editable_path(&info.path, &name, std::slice::from_ref(&name))
        .map_err(|e| write_err(&e))?;
    let bytes = write::encode(
        &text,
        &TextFormat {
            mixed: false,
            ..format
        },
    );
    let (copy, fp) =
        write::save_conflict_copy(&info.path, &name, &bytes, &stamp).map_err(|e| write_err(&e))?;
    after_write(&app, &info);
    Ok(SavedCopy {
        name: copy,
        fingerprint: fp.token(),
    })
}

/// Checks or unchecks a task (CHK-02, CHK-03), only when the project opted in (CHK-01).
/// Returns the tasks view of what was written, so progress updates at once (CHK-05, PRG-01).
#[tauri::command]
#[specta::specta]
pub async fn toggle_task<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    instance_id: u32,
    task_id: String,
    line: u32,
    done: bool,
    base: String,
) -> Result<TasksView, CmdError> {
    let info = instance(&state, instance_id)?;
    let enabled = state
        .config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .projects
        .get(&info.project_name)
        .is_some_and(|p| p.tasks_checkbox);
    if !enabled {
        return Err(err(
            "checkbox_off",
            "checking tasks is off for this project",
        ));
    }
    let path =
        write::editable_path(&info.path, "tasks.md", &listed(&info)).map_err(|e| write_err(&e))?;
    let base = fingerprint_arg(&base)?;
    let (bytes, _) = tasks_edit::toggle_file(&path, line as usize, &task_id, done, &base)
        .map_err(|e| err(e.code(), &e))?;
    after_write(&app, &info);
    Ok(tasks_view_of(&state, &info, Some(&bytes)))
}

/// Temporary files of the write pipeline left in the spec folder by a crash (SAV-10).
#[tauri::command]
#[specta::specta]
pub async fn orphan_temps(
    state: State<'_, AppState>,
    instance_id: u32,
) -> Result<Vec<String>, CmdError> {
    let info = instance(&state, instance_id)?;
    write::orphan_temps(&info.path).map_err(|e| err("io", e))
}

/// Removes one of them, on an explicit click (SAV-10).
#[tauri::command]
#[specta::specta]
pub async fn remove_orphan_temp(
    state: State<'_, AppState>,
    instance_id: u32,
    name: String,
) -> Result<(), CmdError> {
    let info = instance(&state, instance_id)?;
    write::remove_orphan_temp(&info.path, &name).map_err(|e| write_err(&e))
}
