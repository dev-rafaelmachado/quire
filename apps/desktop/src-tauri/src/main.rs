// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod cli_delivery;
mod commands;
mod edit;
mod events;
mod live;
mod state;

/// Runtime the commands are compiled for: the real webview in the app, the mock one in tests.
/// The generated TypeScript does not depend on it.
#[cfg(not(test))]
type Rt = tauri::Wry;
#[cfg(test)]
type Rt = tauri::test::MockRuntime;

/// Commands and events of the app; also the source of the TypeScript bindings (TD-07).
fn specta_builder() -> tauri_specta::Builder<Rt> {
    tauri_specta::Builder::<Rt>::new()
        .commands(tauri_specta::collect_commands![
            commands::get_config,
            commands::set_config::<Rt>,
            commands::start_scan::<Rt>,
            commands::get_watch_status,
            commands::take_cli_open::<Rt>,
            commands::list_projects,
            commands::get_spec,
            commands::read_file,
            commands::get_tasks,
            commands::get_gates,
            commands::get_lessons,
            commands::open_external::<Rt>,
            edit::open_for_edit,
            edit::file_fingerprint,
            edit::save_file::<Rt>,
            edit::save_copy::<Rt>,
            edit::toggle_task::<Rt>,
            edit::orphan_temps,
            edit::remove_orphan_temp,
        ])
        .events(tauri_specta::collect_events![
            events::ScanProgress,
            events::SpecsChanged,
            events::SpecChanged,
            events::WatchStatus,
            events::CliNavigate
        ])
}

/// Generated TypeScript bindings, checked in and kept in sync by a test.
#[cfg(test)]
const BINDINGS: &str = "../src/lib/ipc/bindings.ts";

/// Second instance (CLI-04): focus the window and navigate to what was asked.
#[cfg(not(test))]
fn on_second_instance(app: &tauri::AppHandle, argv: Vec<String>, cwd: String) {
    use tauri::Manager;
    use tauri_plugin_cli::CliExt;
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    if let Ok(matches) = app.cli().matches_from(argv) {
        let request = cli::interpret(&matches, std::path::Path::new(&cwd));
        cli_delivery::deliver(app, request);
    }
}

#[cfg(not(test))]
fn main() {
    use tauri::Manager;
    use tauri_plugin_cli::CliExt;

    let builder = specta_builder();
    let state = match state::AppState::load() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("quire: cannot start: {e}");
            std::process::exit(1);
        }
    };
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_cli::init())
        .manage(state)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // 1. --help / --version answer in the terminal, before any window (CLI-05).
            let cwd = std::env::current_dir().unwrap_or_default();
            let request = match app.cli().matches() {
                Ok(m) => cli::interpret(&m, &cwd),
                Err(e) => {
                    eprintln!("quire: {e}");
                    std::process::exit(2);
                }
            };
            match &request {
                cli::CliRequest::Help(text) => {
                    println!("{text}");
                    std::process::exit(0);
                }
                cli::CliRequest::Version => {
                    println!("quire {}", app.package_info().version);
                    std::process::exit(0);
                }
                _ => {}
            }
            // 2. A running instance gets our argv and this process exits (CLI-04).
            app.handle()
                .plugin(tauri_plugin_single_instance::init(on_second_instance))?;
            // 3. Only now the window (created by hand: `create: false` in tauri.conf.json).
            if let Some(config) = app.config().app.windows.first() {
                tauri::WebviewWindowBuilder::from_config(app.handle(), config)?.build()?;
            }
            builder.mount_events(app);
            cli_delivery::deliver(app.handle(), request);
            let _ = app.get_webview_window("main");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the Quire application");
}

#[cfg(test)]
fn main() {}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};
    use specta_typescript::Typescript;
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{INVOKE_KEY, MockRuntime, mock_builder, mock_context, noop_assets};
    use tauri::webview::InvokeRequest;
    use tauri::{Manager, WebviewWindow};

    use super::*;

    fn manifest_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    const HEADER: &str = "// Generated by tauri-specta from apps/desktop/src-tauri. Do not edit by hand:\n// run `cargo test -p quire-desktop` to regenerate.\n";

    /// Fails when the checked-in bindings differ from the commands, after rewriting them.
    #[test]
    fn bindings_are_up_to_date() {
        let path = manifest_dir().join(BINDINGS);
        let before = std::fs::read_to_string(&path).unwrap_or_default();
        specta_builder()
            .export(Typescript::default().header(HEADER), &path)
            .unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(
            before == after,
            "bindings.ts was stale and has been regenerated; commit it"
        );
    }

    fn bash() -> PathBuf {
        let git_bash = Path::new(r"C:\Program Files\Git\bin\bash.exe");
        if cfg!(windows) && git_bash.exists() {
            git_bash.to_path_buf()
        } else {
            PathBuf::from("bash")
        }
    }

    /// Mock context with the real `plugins.cli` grammar from tauri.conf.json.
    fn context_with_cli() -> tauri::Context<MockRuntime> {
        let conf: Value = serde_json::from_str(
            &std::fs::read_to_string(manifest_dir().join("tauri.conf.json")).unwrap(),
        )
        .unwrap();
        let mut ctx = mock_context(noop_assets());
        ctx.config_mut()
            .plugins
            .0
            .insert("cli".into(), conf["plugins"]["cli"].clone());
        ctx
    }

    const FIXTURES_CONFIG: &str = "scan_paths = [\"ws/projects\"]\n[projects.acme_gateway]\npreset = \"exban\"\n[projects.acme_billing]\npreset = \"exban\"\n";

    fn app_on_fixtures(
        name: &str,
    ) -> (
        tauri::App<MockRuntime>,
        WebviewWindow<MockRuntime>,
        tempfile::TempDir,
    ) {
        app_on_fixtures_with(name, FIXTURES_CONFIG)
    }

    /// Fresh fixtures for this test only, with `config` as quire.toml.
    fn app_on_fixtures_with(
        name: &str,
        config_text: &str,
    ) -> (
        tauri::App<MockRuntime>,
        WebviewWindow<MockRuntime>,
        tempfile::TempDir,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("ws");
        let script = manifest_dir().join("../../../scripts/make-fixtures.sh");
        assert!(
            Command::new(bash())
                .arg(script)
                .arg(&ws)
                .status()
                .unwrap()
                .success()
        );
        let cfg = tmp.path().join("quire.toml");
        std::fs::write(&cfg, config_text).unwrap();
        let config = quire_core::Config::load(&cfg).unwrap();
        let state = state::AppState::new(
            config,
            Some(cfg),
            None,
            tmp.path().join(format!("{name}.sqlite")),
        )
        .unwrap();
        let builder = specta_builder();
        let app = mock_builder()
            .plugin(tauri_plugin_cli::init())
            .manage(state)
            .invoke_handler(builder.invoke_handler())
            .build(context_with_cli())
            .unwrap();
        // The mock app does not run the setup hook, so events are mounted here.
        builder.mount_events(&app);
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        (app, webview, tmp)
    }

    fn app_with_state(
        state: state::AppState,
    ) -> (tauri::App<MockRuntime>, WebviewWindow<MockRuntime>) {
        let builder = specta_builder();
        let app = mock_builder()
            .plugin(tauri_plugin_cli::init())
            .manage(state)
            .invoke_handler(builder.invoke_handler())
            .build(context_with_cli())
            .unwrap();
        builder.mount_events(&app);
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        (app, webview)
    }

    /// The default instance of ACME-142 in the fixtures: (instance id, folder on disk).
    fn acme_142(webview: &WebviewWindow<MockRuntime>) -> (Value, PathBuf) {
        let list = invoke(webview, "list_projects", json!({})).unwrap();
        let spec = list[1]["specs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["folder"] == "acme-142-service-tokens")
            .unwrap()
            .clone();
        let detail = invoke(webview, "get_spec", json!({ "specId": spec["id"] })).unwrap();
        let inst = &detail["spec"]["instances"][detail["instanceIndex"].as_u64().unwrap() as usize];
        (
            inst["id"].clone(),
            PathBuf::from(inst["path"].as_str().unwrap()),
        )
    }

    fn wait_scan(app: &tauri::App<MockRuntime>) {
        let state = app.state::<state::AppState>();
        let t0 = Instant::now();
        // The write starts the reconciliation before returning; wait until it is done.
        while state.scanning.load(std::sync::atomic::Ordering::SeqCst) {
            assert!(
                t0.elapsed() < Duration::from_secs(60),
                "scan did not finish"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[test]
    fn write_commands_over_ipc() {
        let (app, webview, _tmp) = app_on_fixtures("write");
        scan_and_wait(&app, &webview);
        let (instance_id, dir) = acme_142(&webview);
        let spec_path = dir.join("spec.md");
        let original = std::fs::read(&spec_path).unwrap();

        // Open: text, format and base; locks by name (EDT-08).
        let file = invoke(
            &webview,
            "open_for_edit",
            json!({ "instanceId": instance_id, "name": "spec.md" }),
        )
        .unwrap();
        assert_eq!(file["locked"], Value::Null);
        assert_eq!(file["readOnly"], false);
        let base = file["fingerprint"].as_str().unwrap().to_owned();
        let format = file["format"].clone();
        let text = file["text"].as_str().unwrap().to_owned();
        assert_eq!(
            quire_core::write::encode(&text, &serde_json::from_value(format.clone()).unwrap()),
            original
        );
        for (name, code) in [
            ("lessons.jsonl", "edit_jsonl"),
            ("payload.json", "edit_not_markdown"),
        ] {
            let f = invoke(
                &webview,
                "open_for_edit",
                json!({ "instanceId": instance_id, "name": name }),
            )
            .unwrap();
            assert_eq!(f["locked"]["code"], code, "{name}");
        }

        // Save with the right base: written, new base returned.
        let edited = format!("{text}\nLinha nova.\n");
        let saved = invoke(
            &webview,
            "save_file",
            json!({ "input": {
            "instanceId": instance_id, "name": "spec.md", "text": edited, "format": format,
            "base": base, "confirmMixed": false } }),
        )
        .unwrap();
        let disk = std::fs::read(&spec_path).unwrap();
        assert_eq!(
            disk,
            quire_core::write::encode(&edited, &serde_json::from_value(format.clone()).unwrap())
        );
        let new_base = saved["fingerprint"].as_str().unwrap().to_owned();
        assert_eq!(
            invoke(
                &webview,
                "file_fingerprint",
                json!({ "instanceId": instance_id, "name": "spec.md" })
            )
            .unwrap(),
            json!(new_base)
        );
        wait_scan(&app);

        // The old base is a conflict now; the disk keeps what is there.
        let stale = invoke(
            &webview,
            "save_file",
            json!({ "input": {
            "instanceId": instance_id, "name": "spec.md", "text": "mine", "format": format,
            "base": base, "confirmMixed": false } }),
        );
        assert_eq!(stale.unwrap_err()["code"], "write_conflict");
        assert_eq!(std::fs::read(&spec_path).unwrap(), disk);

        // Names outside the Markdown files of the folder are refused by the shell (SAV-09).
        for name in [
            "lessons.jsonl",
            "gates.jsonl",
            "payload.json",
            "../spec.md",
            "a/b.md",
            "missing.md",
        ] {
            let r = invoke(
                &webview,
                "save_file",
                json!({ "input": {
                "instanceId": instance_id, "name": name, "text": "x", "format": format,
                "base": new_base, "confirmMixed": false } }),
            );
            assert_eq!(r.unwrap_err()["code"], "write_refused", "{name}");
        }

        // Mixed line endings need the confirmation (SAV-03).
        let mut mixed = format.clone();
        mixed["mixed"] = json!(true);
        let r = invoke(
            &webview,
            "save_file",
            json!({ "input": {
            "instanceId": instance_id, "name": "spec.md", "text": "x", "format": mixed,
            "base": new_base, "confirmMixed": false } }),
        );
        assert_eq!(r.unwrap_err()["code"], "write_mixed_eol");
        assert_eq!(std::fs::read(&spec_path).unwrap(), disk);

        // Copy: a new file, the original untouched (CNF-08).
        let copy = invoke(
            &webview,
            "save_copy",
            json!({ "instanceId": instance_id, "name": "spec.md",
            "text": "buffer", "format": format, "stamp": "20261116-120000" }),
        )
        .unwrap();
        assert_eq!(copy["name"], "spec.conflito-20261116-120000.md");
        assert_eq!(
            std::fs::read(dir.join("spec.conflito-20261116-120000.md")).unwrap(),
            b"buffer"
        );
        assert_eq!(std::fs::read(&spec_path).unwrap(), disk);
        wait_scan(&app);

        // Checkbox writes are refused while the project has not opted in (CHK-01).
        let tasks = invoke(&webview, "get_tasks", json!({ "instanceId": instance_id })).unwrap();
        assert_eq!(tasks["checkbox"], false);
        let r = invoke(
            &webview,
            "toggle_task",
            json!({ "instanceId": instance_id, "taskId": "T8",
            "line": 1, "done": false, "base": tasks["fingerprint"] }),
        );
        assert_eq!(r.unwrap_err()["code"], "checkbox_off");

        // Orphan temps: listed on request, never in the file list, removed only by name (SAV-10).
        let orphan = quire_core::write::temp_name("spec.md", 4242, 1);
        std::fs::write(dir.join(&orphan), b"partial").unwrap();
        assert_eq!(
            invoke(
                &webview,
                "orphan_temps",
                json!({ "instanceId": instance_id })
            )
            .unwrap(),
            json!([orphan])
        );
        scan_and_wait(&app, &webview);
        let (instance_id, _) = acme_142(&webview);
        let detail_files = invoke(&webview, "get_spec", json!({ "specId": invoke(&webview, "list_projects", json!({})).unwrap()[1]["specs"].as_array().unwrap().iter().find(|s| s["folder"] == "acme-142-service-tokens").unwrap()["id"] })).unwrap();
        assert!(
            !detail_files["others"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o.as_str().unwrap().ends_with(".tmp")),
            "{detail_files}"
        );
        invoke(
            &webview,
            "remove_orphan_temp",
            json!({ "instanceId": instance_id, "name": orphan }),
        )
        .unwrap();
        assert!(!dir.join(&orphan).exists());
        let r = invoke(
            &webview,
            "remove_orphan_temp",
            json!({ "instanceId": instance_id, "name": "spec.md" }),
        );
        assert_eq!(r.unwrap_err()["code"], "write_refused");
        assert!(spec_path.exists());
    }

    #[test]
    fn a_save_settles_without_a_reload_cycle() {
        // T19 / TD-14: the app's own write is reconciled (by the write itself and maybe once
        // more by the watcher, merged when they overlap), and then everything stays quiet.
        use tauri::Listener;
        let (app, webview, _tmp) = app_on_fixtures("no-cycle");
        scan_and_wait(&app, &webview);
        let (instance_id, dir) = acme_142(&webview);
        // Let the watcher start and the first events of the scan pass.
        std::thread::sleep(Duration::from_millis(800));
        wait_scan(&app);
        let changed = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = changed.clone();
        app.listen("spec://changed", move |_| {
            counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let state = app.state::<state::AppState>();
        let runs = |s: &state::AppState| s.live.runs.load(std::sync::atomic::Ordering::SeqCst);
        let before = runs(&state);
        let file = invoke(
            &webview,
            "open_for_edit",
            json!({ "instanceId": instance_id, "name": "spec.md" }),
        )
        .unwrap();
        let text = format!("{}\nOutra linha.\n", file["text"].as_str().unwrap());
        invoke(
            &webview,
            "save_file",
            json!({ "input": {
            "instanceId": instance_id, "name": "spec.md", "text": text, "format": file["format"],
            "base": file["fingerprint"], "confirmMixed": false } }),
        )
        .unwrap();
        std::thread::sleep(Duration::from_secs(3));
        wait_scan(&app);
        let after_save = runs(&state);
        let events = changed.load(std::sync::atomic::Ordering::SeqCst);
        assert!(
            (1..=2).contains(&(after_save - before)),
            "{} reconciliations",
            after_save - before
        );
        assert!((1..=2).contains(&events), "{events} spec://changed events");
        // Quiet afterwards: nothing writes again, so nothing reconciles again.
        std::thread::sleep(Duration::from_secs(3));
        assert_eq!(
            runs(&state),
            after_save,
            "reconciliation cycle after a save"
        );
        assert_eq!(changed.load(std::sync::atomic::Ordering::SeqCst), events);
        assert!(
            std::fs::read_to_string(dir.join("spec.md"))
                .unwrap()
                .ends_with("Outra linha.\n")
        );
    }

    #[test]
    fn saving_one_identical_copy_makes_the_versions_badge_appear() {
        // INS-03: ACME-142 has identical copies in the fixtures (three: main, the acme-142
        // worktree and the acme-170 one); a save in one of them makes them diverge, and the
        // badge appears within 2 s with the number of copies, as in DUP-03 of v0.1.
        let (app, webview, _tmp) = app_on_fixtures("versions");
        scan_and_wait(&app, &webview);
        let spec_of = |w: &WebviewWindow<MockRuntime>| -> Value {
            let list = invoke(w, "list_projects", json!({})).unwrap();
            list[1]["specs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["folder"] == "acme-142-service-tokens")
                .unwrap()
                .clone()
        };
        let before = spec_of(&webview);
        assert_eq!(before["versionsBadge"], Value::Null, "{before}");
        assert!(before["instances"].as_array().unwrap().len() >= 2);
        let (instance_id, _) = acme_142(&webview);
        let file = invoke(
            &webview,
            "open_for_edit",
            json!({ "instanceId": instance_id, "name": "spec.md" }),
        )
        .unwrap();
        let t0 = Instant::now();
        invoke(&webview, "save_file", json!({ "input": {
            "instanceId": instance_id, "name": "spec.md", "text": format!("{}\nSó nesta cópia.\n", file["text"].as_str().unwrap()),
            "format": file["format"], "base": file["fingerprint"], "confirmMixed": false } })).unwrap();
        wait_scan(&app);
        let after = spec_of(&webview);
        let copies = after["instances"].as_array().unwrap().len();
        assert_eq!(after["versionsBadge"], json!(copies), "{after}");
        let detail = invoke(
            &webview,
            "get_spec",
            json!({ "specId": after["id"], "instanceId": instance_id }),
        )
        .unwrap();
        assert_eq!(detail["spec"]["versionsBadge"], json!(copies));
        assert!(t0.elapsed() < Duration::from_secs(2), "{:?}", t0.elapsed());
    }

    #[test]
    fn checkbox_and_progress_without_the_watcher() {
        // Polling every 60 s: within this test, only the write itself can update the list.
        let cfg = "scan_paths = [\"ws/projects\"]\npoll_interval_ms = 60000\n\n[watch]\nmode = \"poll\"\n\n[projects.acme_gateway]\npreset = \"exban\"\ntasks_checkbox = true\n\n[projects.acme_billing]\npreset = \"exban\"\n";
        let (app, webview, _tmp) = app_on_fixtures_with("checkbox", cfg);
        scan_and_wait(&app, &webview);
        let (instance_id, dir) = acme_142(&webview);
        let before = std::fs::read(dir.join("tasks.md")).unwrap();
        let tasks = invoke(&webview, "get_tasks", json!({ "instanceId": instance_id })).unwrap();
        assert_eq!(tasks["checkbox"], true);
        assert_eq!(
            (tasks["done"].as_u64(), tasks["total"].as_u64()),
            (Some(7), Some(8))
        );
        let pending = tasks["phases"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|p| p["tasks"].as_array().unwrap().iter())
            .find(|t| t["done"] == false)
            .unwrap()
            .clone();

        // A stale base is refused and nothing changes (CHK-03).
        let r = invoke(
            &webview,
            "toggle_task",
            json!({ "instanceId": instance_id, "taskId": pending["id"],
            "line": pending["line"], "done": false, "base": "1-00000000000000000000000000000000" }),
        );
        assert_eq!(r.unwrap_err()["code"], "write_conflict");
        // The line moved: refused.
        let r = invoke(
            &webview,
            "toggle_task",
            json!({ "instanceId": instance_id, "taskId": pending["id"],
            "line": pending["line"].as_u64().unwrap() + 1, "done": false, "base": tasks["fingerprint"] }),
        );
        assert_eq!(r.unwrap_err()["code"], "toggle_moved");
        assert_eq!(std::fs::read(dir.join("tasks.md")).unwrap(), before);

        let t0 = Instant::now();
        let view = invoke(
            &webview,
            "toggle_task",
            json!({ "instanceId": instance_id, "taskId": pending["id"],
            "line": pending["line"], "done": false, "base": tasks["fingerprint"] }),
        )
        .unwrap();
        assert_eq!(
            (view["done"].as_u64(), view["total"].as_u64()),
            (Some(8), Some(8))
        );
        let after = std::fs::read(dir.join("tasks.md")).unwrap();
        assert_eq!(before.len(), after.len());
        assert_eq!(before.iter().zip(&after).filter(|(a, b)| a != b).count(), 1);
        // The list follows from the write's own reconciliation (PRG-01, CHK-05).
        wait_scan(&app);
        let (instance_after, _) = acme_142(&webview);
        let list = invoke(&webview, "list_projects", json!({})).unwrap();
        let spec = list[1]["specs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["folder"] == "acme-142-service-tokens")
            .unwrap();
        let inst = spec["instances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["id"] == instance_after)
            .unwrap();
        assert_eq!(inst["progress"], json!([8, 8]));
        assert!(t0.elapsed() < Duration::from_secs(5), "{:?}", t0.elapsed());
    }

    #[test]
    fn relative_env_paths_are_refused_and_nothing_is_written() {
        // DEB-05, TD-21. Names unique to this test, relative to the working directory.
        let rel_dir = format!("quire-t4-{}", std::process::id());
        let rel_config = PathBuf::from(&rel_dir).join("quire.toml");
        let tmp = tempfile::tempdir().unwrap();
        let index = tmp.path().join("index.sqlite");

        let state = state::AppState::from_env(
            Some(rel_config.clone().into_os_string()),
            Some(index.clone().into_os_string()),
        )
        .unwrap();
        let (_app, webview) = app_with_state(state);
        let view = invoke(&webview, "get_config", json!({})).unwrap();
        assert_eq!(view["error"]["code"], "config_env_relative", "{view}");
        assert_eq!(view["editable"], false);
        assert_eq!(view["path"], Value::Null);
        let saved = invoke(
            &webview,
            "set_config",
            json!({ "settings": { "scanPaths": ["x"], "pollIntervalMs": 3000, "theme": null, "language": null, "projects": [] } }),
        );
        assert_eq!(saved.unwrap_err()["code"], "config_env_relative");
        assert!(
            !Path::new(&rel_dir).exists(),
            "nothing may be created relative to the cwd"
        );

        // Index: relative, or inside a watched .specs folder, stops the start.
        let rel_index = PathBuf::from(&rel_dir).join("index.sqlite");
        let err = state::AppState::from_env(None, Some(rel_index.into_os_string()))
            .err()
            .unwrap();
        assert!(err.contains("absolute"), "{err}");
        let inside = tmp.path().join(".specs").join("index.sqlite");
        let err = state::AppState::from_env(None, Some(inside.clone().into_os_string()))
            .err()
            .unwrap();
        assert!(err.contains(".specs"), "{err}");
        assert!(!Path::new(&rel_dir).exists());
        assert!(!inside.exists() && !tmp.path().join(".specs").exists());

        // Absolute paths work as before.
        let cfg = tmp.path().join("quire.toml");
        std::fs::write(&cfg, "scan_paths = []\n").unwrap();
        let state = state::AppState::from_env(
            Some(cfg.into_os_string()),
            Some(tmp.path().join("ok.sqlite").into_os_string()),
        )
        .unwrap();
        let (_app, webview) = app_with_state(state);
        let view = invoke(&webview, "get_config", json!({})).unwrap();
        assert_eq!(view["error"], Value::Null);
        assert_eq!(view["editable"], true);
    }

    fn invoke(
        webview: &WebviewWindow<MockRuntime>,
        cmd: &str,
        args: Value,
    ) -> Result<Value, Value> {
        let url = if cfg!(windows) {
            "http://tauri.localhost"
        } else {
            "tauri://localhost"
        };
        tauri::test::get_ipc_response(
            webview,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: url.parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|r| r.deserialize::<Value>().unwrap())
    }

    fn scan_and_wait(app: &tauri::App<MockRuntime>, webview: &WebviewWindow<MockRuntime>) {
        invoke(webview, "start_scan", json!({})).unwrap();
        let state = app.state::<state::AppState>();
        let t0 = Instant::now();
        while state.scanning.load(std::sync::atomic::Ordering::SeqCst) {
            assert!(
                t0.elapsed() < Duration::from_secs(60),
                "scan did not finish"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn wait_for(what: &str, timeout: Duration, check: impl FnMut() -> bool) -> Duration {
        wait_for_or(what, timeout, check, String::new)
    }

    /// Like `wait_for`, with `context()` added to the message on timeout.
    fn wait_for_or(
        what: &str,
        timeout: Duration,
        mut check: impl FnMut() -> bool,
        context: impl Fn() -> String,
    ) -> Duration {
        let t0 = Instant::now();
        while !check() {
            assert!(
                t0.elapsed() < timeout,
                "timed out waiting for {what}{}",
                context()
            );
            std::thread::sleep(Duration::from_millis(25));
        }
        t0.elapsed()
    }

    fn spec_of(list: &Value, folder: &str) -> Option<Value> {
        list.as_array()?
            .iter()
            .flat_map(|p| p["specs"].as_array().cloned().unwrap_or_default())
            .find(|s| s["folder"] == folder)
    }

    #[test]
    fn watcher_keeps_the_index_and_ui_in_line_with_the_disk() {
        use tauri::Listener;
        let (app, webview, tmp) = app_on_fixtures("watch");
        let changed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = changed.clone();
        app.listen("spec://changed", move |e| {
            sink.lock().unwrap().push(e.payload().to_owned())
        });
        scan_and_wait(&app, &webview);
        // On timeout, show what the watcher received: event kinds and paths differ per OS.
        let raw = || {
            format!(
                "\nraw events:\n  {}",
                live::raw_events(app.handle()).join("\n  ")
            )
        };

        let status = invoke(&webview, "get_watch_status", json!({})).unwrap();
        assert!(status["repos"].as_u64().unwrap() >= 4, "{status}");
        assert_eq!(
            status["polled"], 0,
            "local fixtures use native events: {status}"
        );

        // WAT-01: an edit made outside the app reaches the index and the UI within 2 s.
        let gw = tmp.path().join("ws/projects/acme_gateway/.specs/features");
        let tasks = gw.join("acme-170-rate-limit/tasks.md");
        let text = std::fs::read_to_string(&tasks)
            .unwrap()
            .replace("- [ ] T3:", "- [x] T3:");
        std::fs::write(&tasks, text).unwrap();
        let took = wait_for_or(
            "the edit in the list",
            Duration::from_secs(2),
            || {
                let list = invoke(&webview, "list_projects", json!({})).unwrap();
                let spec = spec_of(&list, "acme-170-rate-limit").unwrap();
                spec["instances"].as_array().unwrap().iter().any(|i| {
                    // Windows paths use `\`.
                    i["progress"] == json!([2, 3])
                        && i["path"]
                            .as_str()
                            .unwrap()
                            .replace('\\', "/")
                            .contains("projects/acme_gateway")
                })
            },
            raw,
        );
        eprintln!("WAT-01: edit visible in the list after {took:?}");
        wait_for_or(
            "spec://changed",
            Duration::from_secs(2),
            || {
                changed
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|p| p.contains("acme-170-rate-limit"))
            },
            raw,
        );

        // A spec folder created together with its file appears (the folder's content is rescanned).
        let new = gw.join("acme-300-new");
        std::fs::create_dir(&new).unwrap();
        std::fs::write(new.join("spec.md"), "# ACME-300 — Nova\n").unwrap();
        wait_for_or(
            "the new spec",
            Duration::from_secs(2),
            || {
                spec_of(
                    &invoke(&webview, "list_projects", json!({})).unwrap(),
                    "acme-300-new",
                )
                .is_some()
            },
            raw,
        );

        // Removing it removes it from the list.
        std::fs::remove_dir_all(&new).unwrap();
        wait_for_or(
            "the spec to go away",
            Duration::from_secs(2),
            || {
                spec_of(
                    &invoke(&webview, "list_projects", json!({})).unwrap(),
                    "acme-300-new",
                )
                .is_none()
            },
            raw,
        );
    }

    fn parse(app: &tauri::App<MockRuntime>, args: &[&str], cwd: &Path) -> cli::CliRequest {
        use tauri_plugin_cli::CliExt;
        let argv = std::iter::once("quire")
            .chain(args.iter().copied())
            .map(String::from)
            .collect();
        cli::interpret(&app.cli().matches_from(argv).unwrap(), cwd)
    }

    /// What the app prints for a command line on Linux (setup in `run`): text and exit code,
    /// `None` when it would go on to open the window.
    fn app_answer(app: &tauri::App<MockRuntime>, args: &[&str]) -> Option<(String, i32)> {
        use tauri_plugin_cli::CliExt;
        let argv = std::iter::once("quire")
            .chain(args.iter().copied())
            .map(String::from)
            .collect();
        match app.cli().matches_from(argv) {
            Err(e) => Some((format!("quire: {e}"), 2)),
            Ok(m) => match cli::interpret(&m, Path::new(".")) {
                cli::CliRequest::Help(text) => Some((text, 0)),
                cli::CliRequest::Version => {
                    Some((format!("quire {}", app.package_info().version), 0))
                }
                _ => None,
            },
        }
    }

    #[test]
    fn the_windows_console_helper_answers_like_the_app() {
        // DEB-06, TD-20: same help, errors and codes as the app, from the same grammar.
        let (app, _webview, _tmp) = app_on_fixtures("cli-parity");
        let name = app.package_info().name.clone();
        for args in [
            vec!["--help"],
            vec!["-h"],
            vec!["open", "--help"],
            vec!["help", "open"],
            vec!["--bogus"],
            vec!["open"],
            vec!["open", "a", "b"],
            vec![],
            vec!["."],
            vec!["open", "ACME-142"],
        ] {
            let argv: Vec<std::ffi::OsString> = std::iter::once("quire")
                .chain(args.iter().copied())
                .map(Into::into)
                .collect();
            let helper = match quire_cli::decide(&argv) {
                quire_cli::Outcome::Print { text, code } => Some((text, code)),
                quire_cli::Outcome::Launch(rest) => {
                    assert_eq!(rest, argv[1..].to_vec());
                    None
                }
            };
            // The mock app may carry another package name; the helper always says `quire`.
            let ours = app_answer(&app, &args)
                .map(|(t, c)| (t.replace(&format!("Usage: {name}"), "Usage: quire"), c));
            assert_eq!(helper, ours, "{args:?}");
        }
        // The version comes from the same tauri.conf.json.
        let conf: Value = serde_json::from_str(
            &std::fs::read_to_string(manifest_dir().join("tauri.conf.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            quire_cli::decide(&["quire".into(), "--version".into()]),
            quire_cli::Outcome::Print {
                text: format!("quire {}", conf["version"].as_str().unwrap()),
                code: 0
            }
        );
    }

    #[test]
    fn command_line_grammar() {
        let (app, _webview, _tmp) = app_on_fixtures("cli-grammar");
        let cwd = Path::new("/somewhere");
        assert!(
            matches!(parse(&app, &["--help"], cwd), cli::CliRequest::Help(h) if h.contains("open"))
        );
        assert!(matches!(
            parse(&app, &["open", "--help"], cwd),
            cli::CliRequest::Help(_)
        ));
        assert_eq!(parse(&app, &["--version"], cwd), cli::CliRequest::Version);
        assert_eq!(parse(&app, &[], cwd), cli::CliRequest::Nothing);
        assert_eq!(
            parse(&app, &["."], cwd),
            cli::CliRequest::Here(cwd.join("."))
        );
        assert_eq!(
            parse(&app, &["open", "ACME-142"], cwd),
            cli::CliRequest::Open {
                query: "ACME-142".into(),
                cwd: cwd.to_path_buf()
            }
        );
    }

    #[test]
    fn command_line_navigation_end_to_end() {
        let (app, webview, tmp) = app_on_fixtures("cli-nav");
        scan_and_wait(&app, &webview);
        let ws = tmp.path().join("ws").canonicalize().unwrap();
        let list = invoke(&webview, "list_projects", json!({})).unwrap();
        let spec_id = |folder: &str, project: usize| -> Value {
            list[project]["specs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["folder"] == folder)
                .unwrap()["id"]
                .clone()
        };
        let take = || invoke(&webview, "take_cli_open", json!({})).unwrap();

        // CLI-02: a key in one project opens it; a key in two projects asks to pick.
        cli_delivery::deliver(app.handle(), parse(&app, &["open", "ACME-142"], &ws));
        assert_eq!(
            take(),
            json!({ "kind": "spec", "specId": spec_id("acme-142-service-tokens", 1), "instanceId": null, "file": null })
        );
        cli_delivery::deliver(app.handle(), parse(&app, &["open", "acme-200"], &ws));
        let pick = take();
        assert_eq!(pick["kind"], "pick");
        assert_eq!(pick["specIds"].as_array().unwrap().len(), 2);

        // CLI-01: `quire .` inside the worktree of acme-170 lists its specs, highlighted.
        let wt = ws.join("external/acme-170/acme_gateway");
        cli_delivery::deliver(app.handle(), parse(&app, &["."], &wt.join("src")));
        let here = take();
        assert_eq!(here["kind"], "highlight", "{here}");
        assert_eq!(here["specIds"].as_array().unwrap().len(), 3);

        // CLI-03: a path to a spec file opens that spec in that instance.
        let file = wt.join(".specs/features/acme-170-rate-limit/tasks.md");
        cli_delivery::deliver(
            app.handle(),
            parse(&app, &["open", file.to_str().unwrap()], &ws),
        );
        let by_path = take();
        assert_eq!(by_path["kind"], "spec");
        assert_eq!(by_path["specId"], spec_id("acme-170-rate-limit", 1));
        assert!(by_path["instanceId"].is_number());
        assert_eq!(by_path["file"], "tasks.md");

        cli_delivery::deliver(app.handle(), parse(&app, &["open", "NOPE-1"], &ws));
        assert_eq!(take(), json!({ "kind": "notFound", "query": "NOPE-1" }));
        assert_eq!(take(), Value::Null, "taking clears it");
    }

    #[test]
    fn preferences_are_saved_and_applied_without_restart() {
        let (app, webview, tmp) = app_on_fixtures("prefs");
        scan_and_wait(&app, &webview);
        let config = invoke(&webview, "get_config", json!({})).unwrap();
        assert_eq!(config["editable"], true);
        let missing = tmp.path().join("missing").display().to_string();
        let saved = invoke(
            &webview,
            "set_config",
            json!({ "settings": {
                "scanPaths": ["ws/projects", missing],
                "pollIntervalMs": 2000,
                "theme": "light",
                "projects": [
                    { "name": "acme_gateway", "preset": "exban", "alias": "gateway", "issueUrlBase": "https://tracker.example/browse/" },
                    { "name": "acme_billing", "preset": "generic", "alias": null, "issueUrlBase": null }
                ]
            }}),
        )
        .unwrap();
        assert_eq!(saved["theme"], "light");
        assert_eq!(saved["pollIntervalMs"], 2000);
        let status = saved["scanPathStatus"].as_array().unwrap();
        assert_eq!(status[0]["error"], Value::Null);
        assert_eq!(status[1]["error"]["code"], "path_not_found", "{status:?}");
        assert!(
            status[1]["error"]["message"]
                .as_str()
                .unwrap()
                .contains("does not exist"),
            "{status:?}"
        );
        let text = std::fs::read_to_string(tmp.path().join("quire.toml")).unwrap();
        assert!(
            text.contains("alias = \"gateway\"") && text.contains("theme = \"light\""),
            "{text}"
        );

        // Applied without restart: the rescan picks up the alias and the new preset.
        scan_and_wait(&app, &webview);
        wait_for("the alias in the list", Duration::from_secs(5), || {
            let list = invoke(&webview, "list_projects", json!({})).unwrap();
            list.as_array()
                .unwrap()
                .iter()
                .any(|p| p["alias"] == "gateway")
                && list
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["name"] == "acme_billing" && p["preset"] == "generic")
        });

        let bad = invoke(
            &webview,
            "set_config",
            json!({ "settings": { "scanPaths": [], "pollIntervalMs": 10, "theme": null, "projects": [] } }),
        );
        assert_eq!(bad.unwrap_err()["code"], "poll_interval_range");
        let bad_preset = invoke(
            &webview,
            "set_config",
            json!({ "settings": { "scanPaths": [], "pollIntervalMs": 3000, "theme": null,
                "projects": [{ "name": "x", "preset": "nope", "alias": null, "issueUrlBase": null }] } }),
        );
        assert_eq!(bad_preset.unwrap_err()["code"], "config_invalid");
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("quire.toml")).unwrap(),
            text,
            "nothing written on error"
        );
    }

    #[test]
    fn list_projects_end_to_end_over_ipc() {
        let (app, webview, _tmp) = app_on_fixtures("list");
        assert_eq!(
            invoke(&webview, "list_projects", json!({})).unwrap(),
            json!([]),
            "empty cache before scan"
        );
        scan_and_wait(&app, &webview);

        let list = invoke(&webview, "list_projects", json!({ "filter": null })).unwrap();
        let names: Vec<&str> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["acme_billing", "acme_gateway"]);
        let gateway = &list[1];
        assert_eq!(gateway["preset"], "exban");
        let s170 = gateway["specs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["folder"] == "acme-170-rate-limit")
            .unwrap();
        assert_eq!(s170["versionsBadge"], 3);
        assert_eq!(s170["issueKey"], "ACME-170");
        let default = &s170["instances"][s170["defaultIndex"].as_u64().unwrap() as usize];
        assert_eq!(default["kind"], "worktree");
        assert_eq!(default["branch"], "feat/acme-170-rate-limit");
        assert_eq!(default["progress"], json!([2, 3]));

        let filtered = invoke(
            &webview,
            "list_projects",
            json!({ "filter": { "text": "bill-31" } }),
        )
        .unwrap();
        assert_eq!(filtered.as_array().unwrap().len(), 1);
        assert_eq!(filtered[0]["specs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn spec_files_tasks_gates_and_lessons_over_ipc() {
        let (app, webview, _tmp) = app_on_fixtures("spec");
        scan_and_wait(&app, &webview);
        let list = invoke(&webview, "list_projects", json!({})).unwrap();
        let find = |project: usize, folder: &str| -> Value {
            list[project]["specs"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["folder"] == folder)
                .unwrap()
                .clone()
        };

        let s142 = find(1, "acme-142-service-tokens");
        let detail = invoke(&webview, "get_spec", json!({ "specId": s142["id"] })).unwrap();
        assert_eq!(
            detail["tabs"],
            json!([
                "spec.md",
                "design.md",
                "tasks.md",
                "summary.md",
                "validation.md",
                "gates.jsonl",
                "lessons.jsonl"
            ])
        );
        assert_eq!(detail["others"], json!(["notes.md", "payload.json"]));
        assert_eq!(
            detail["header"][1]["url"],
            "https://tracker.example/browse/ACME-142"
        );
        assert_eq!(detail["project"]["displayName"], "acme_gateway");
        let instance_id = detail["spec"]["instances"]
            [detail["instanceIndex"].as_u64().unwrap() as usize]["id"]
            .clone();

        let other = detail["spec"]["instances"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["id"].clone())
            .find(|i| *i != instance_id)
            .unwrap();
        let picked = invoke(
            &webview,
            "get_spec",
            json!({ "specId": s142["id"], "instanceId": other }),
        )
        .unwrap();
        assert_ne!(picked["instanceIndex"], detail["instanceIndex"]);

        let file = invoke(
            &webview,
            "read_file",
            json!({ "instanceId": instance_id, "name": "payload.json" }),
        )
        .unwrap();
        assert_eq!(
            (file["kind"].as_str(), file["large"].as_bool()),
            (Some("json"), Some(false))
        );
        assert!(file["text"].as_str().unwrap().contains("shown-once"));
        let escape = invoke(
            &webview,
            "read_file",
            json!({ "instanceId": instance_id, "name": "../../../.git/config" }),
        );
        assert_eq!(escape.unwrap_err()["code"], "not_found");

        let tasks = invoke(&webview, "get_tasks", json!({ "instanceId": instance_id })).unwrap();
        assert_eq!(
            (tasks["done"].as_u64(), tasks["total"].as_u64()),
            (Some(7), Some(8))
        );
        assert_eq!(tasks["phases"][1]["state"], "inProgress");
        assert_eq!(
            tasks["phases"][1]["diagram"],
            json!([["T4", "T5"], ["T4", "T6"]])
        );

        let bill = find(0, "bill-31-invoice-export");
        let bill_detail = invoke(&webview, "get_spec", json!({ "specId": bill["id"] })).unwrap();
        let bill_instance = bill_detail["spec"]["instances"][0]["id"].clone();
        let gates = invoke(
            &webview,
            "get_gates",
            json!({ "instanceId": bill_instance }),
        )
        .unwrap();
        assert_eq!(gates["skipped"], 2);
        assert_eq!(
            gates["counts"],
            json!({ "runs": 3, "passes": 2, "fails": 1 })
        );
        assert_eq!(gates["runs"][0]["level"], "final", "newest first");
        let lessons = invoke(
            &webview,
            "get_lessons",
            json!({ "instanceId": bill_instance }),
        )
        .unwrap();
        assert_eq!(lessons["lessons"][0]["signal"], "spec_precision_gap");

        let config = invoke(&webview, "get_config", json!({})).unwrap();
        assert_eq!(config["pollIntervalMs"], 3000);
        let bad = invoke(
            &webview,
            "open_external",
            json!({ "url": "file:///etc/passwd" }),
        );
        assert_eq!(bad.unwrap_err()["code"], "invalid_url");
    }
}
