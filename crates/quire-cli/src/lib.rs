//! The `quire` command for Windows terminals (DEB-06, TD-20).
//!
//! `quire.exe` is a GUI binary, so Windows does not connect its output to the terminal it was
//! started from. This console helper answers `--help`, `--version` and argument errors itself,
//! with the same text and exit codes the app prints on Linux, and starts the app for every
//! valid call without waiting. The grammar is the `plugins.cli` section of `tauri.conf.json`,
//! embedded at build time, and the command is built the way tauri-plugin-cli builds it, so the
//! two never diverge (a test in the desktop shell compares them).

use std::ffi::OsString;
use std::path::Path;

use clap::{Arg, ArgAction, Command, error::ErrorKind};
use serde::Deserialize;

const TAURI_CONF: &str = include_str!("../../../apps/desktop/src-tauri/tauri.conf.json");

/// The program name the app uses in its help (the binary name).
pub const NAME: &str = "quire";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Conf {
    version: String,
    plugins: Plugins,
}

#[derive(Debug, Deserialize)]
struct Plugins {
    cli: CliConf,
}

/// The subset of tauri-plugin-cli's grammar this project uses.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CliConf {
    description: Option<String>,
    #[serde(default)]
    args: Vec<ArgConf>,
    #[serde(default)]
    subcommands: std::collections::BTreeMap<String, CliConf>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ArgConf {
    name: String,
    index: Option<usize>,
    short: Option<char>,
    #[serde(default)]
    takes_value: bool,
    #[serde(default)]
    required: bool,
    description: Option<String>,
}

fn conf() -> Result<Conf, String> {
    serde_json::from_str(TAURI_CONF).map_err(|e| format!("bad embedded tauri.conf.json: {e}"))
}

fn arg(a: &ArgConf) -> Arg {
    let mut arg = Arg::new(a.name.clone());
    if a.index.is_none() {
        arg = arg.long(a.name.clone());
        if let Some(short) = a.short {
            arg = arg.short(short);
        }
    }
    if let Some(d) = &a.description {
        arg = arg.help(d.clone());
    }
    arg = arg.action(if a.takes_value {
        ArgAction::Set
    } else {
        ArgAction::Count
    });
    if let Some(i) = a.index {
        arg = arg.index(i);
    }
    arg.required(a.required)
}

/// Same construction as tauri-plugin-cli's `get_app` for the fields this grammar uses.
fn build(name: String, version: &str, about: Option<&String>, cli: &CliConf) -> Command {
    let mut cmd = Command::new(name).author("").version(version.to_owned());
    if let Some(about) = about {
        cmd = cmd.about(about.clone());
    }
    // tauri-plugin-cli 2.5's `Config::long_description()` returns the description, which turns
    // on clap's long help layout for `--help`; matched on purpose.
    if let Some(long) = &cli.description {
        cmd = cmd.long_about(long.clone());
    }
    for a in &cli.args {
        cmd = cmd.arg(arg(a));
    }
    for (sub_name, sub) in &cli.subcommands {
        cmd = cmd.subcommand(build(
            sub_name.clone(),
            version,
            sub.description.as_ref(),
            sub,
        ));
    }
    cmd
}

/// The `quire` command as the app defines it.
pub fn command() -> Result<Command, String> {
    let conf = conf()?;
    let about = conf.plugins.cli.description.clone();
    Ok(build(
        NAME.to_owned(),
        &conf.version,
        about.as_ref(),
        &conf.plugins.cli,
    ))
}

/// What the helper does with a command line (`args` includes the program name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Print `text` on stdout (`code` 0) or stderr (other codes) and exit with `code`.
    Print { text: String, code: i32 },
    /// A valid call: start the app with these arguments (program name excluded).
    Launch(Vec<OsString>),
}

/// Decides like the app does on Linux (`main.rs` setup): help and version on stdout with 0,
/// argument errors on stderr with 2, as `quire: failed to parse arguments: …`.
pub fn decide(args: &[OsString]) -> Outcome {
    let cmd = match command() {
        Ok(c) => c,
        Err(e) => {
            return Outcome::Print {
                text: format!("{NAME}: {e}"),
                code: 1,
            };
        }
    };
    // The user typed `quire`: never show `quire-cli` or the full path of the helper.
    let argv = std::iter::once(OsString::from(NAME)).chain(args.iter().skip(1).cloned());
    match cmd.try_get_matches_from(argv) {
        Ok(_) => Outcome::Launch(args.iter().skip(1).cloned().collect()),
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp => Outcome::Print {
                text: e.to_string(),
                code: 0,
            },
            ErrorKind::DisplayVersion => Outcome::Print {
                text: format!("{NAME} {}", conf().map(|c| c.version).unwrap_or_default()),
                code: 0,
            },
            _ => Outcome::Print {
                text: format!("{NAME}: failed to parse arguments: {e}"),
                code: 2,
            },
        },
    }
}

/// The app binary next to the helper (`quire.exe` on Windows).
pub fn app_path(helper: &Path) -> Option<std::path::PathBuf> {
    let name = if cfg!(windows) { "quire.exe" } else { "quire" };
    helper.parent().map(|dir| dir.join(name))
}

/// Runs the helper: prints, or starts the app through `launch(app, args)` and returns at once.
/// Returns the exit code.
pub fn run(
    args: &[OsString],
    helper: &Path,
    launch: impl FnOnce(&Path, &[OsString]) -> std::io::Result<()>,
    out: &mut dyn std::io::Write,
    err: &mut dyn std::io::Write,
) -> i32 {
    match decide(args) {
        Outcome::Print { text, code } => {
            let _ = if code == 0 {
                writeln!(out, "{text}")
            } else {
                writeln!(err, "{text}")
            };
            code
        }
        Outcome::Launch(rest) => match app_path(helper) {
            Some(app) if app.is_file() => match launch(&app, &rest) {
                Ok(()) => 0,
                Err(e) => {
                    let _ = writeln!(err, "{NAME}: cannot start {}: {e}", app.display());
                    1
                }
            },
            _ => {
                let _ = writeln!(
                    err,
                    "{NAME}: the Quire app was not found next to {}",
                    helper.display()
                );
                1
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn os(args: &[&str]) -> Vec<OsString> {
        std::iter::once(NAME)
            .chain(args.iter().copied())
            .map(OsString::from)
            .collect()
    }

    #[test]
    fn help_version_and_errors_are_answered_here() {
        // Called through its full path, the helper still presents itself as `quire`.
        let full = vec![
            OsString::from("C:\\Program Files\\Quire\\quire-cli.exe"),
            "--help".into(),
        ];
        assert!(
            matches!(decide(&full), Outcome::Print { code: 0, text } if text.contains("Usage: quire [path]"))
        );
        assert!(
            matches!(decide(&os(&["--help"])), Outcome::Print { code: 0, text } if text.contains("Usage: quire"))
        );
        assert!(
            matches!(decide(&os(&["open", "--help"])), Outcome::Print { code: 0, text } if text.contains("target"))
        );
        assert_eq!(
            decide(&os(&["--version"])),
            Outcome::Print {
                text: format!("quire {}", conf().unwrap().version),
                code: 0
            }
        );
        assert!(
            matches!(decide(&os(&["--bogus"])), Outcome::Print { code: 2, text } if text.starts_with("quire: failed to parse arguments: error: unexpected argument '--bogus'"))
        );
        assert!(matches!(
            decide(&os(&["open"])),
            Outcome::Print { code: 2, .. }
        ));
    }

    #[test]
    fn valid_calls_start_the_app_with_the_same_arguments() {
        assert_eq!(decide(&os(&[])), Outcome::Launch(vec![]));
        assert_eq!(
            decide(&os(&["."])),
            Outcome::Launch(vec![OsString::from(".")])
        );
        assert_eq!(
            decide(&os(&["open", "ACME-142"])),
            Outcome::Launch(vec!["open".into(), "ACME-142".into()])
        );
    }

    #[test]
    fn run_launches_the_sibling_app_or_explains_why_not() {
        let dir = tempfile_dir();
        let helper = dir.join(if cfg!(windows) {
            "quire-cli.exe"
        } else {
            "quire-cli"
        });
        let (mut out, mut err) = (Vec::new(), Vec::new());
        // No app next to the helper: message and code 1, nothing launched.
        let code = run(
            &os(&["."]),
            &helper,
            |_, _| panic!("must not launch"),
            &mut out,
            &mut err,
        );
        assert_eq!(code, 1);
        assert!(String::from_utf8_lossy(&err).contains("not found"));

        let app = app_path(&helper).unwrap();
        std::fs::write(&app, b"").unwrap();
        let mut seen = None;
        let code = run(
            &os(&["open", "ACME-142"]),
            &helper,
            |a, args| {
                seen = Some((a.to_path_buf(), args.to_vec()));
                Ok(())
            },
            &mut out,
            &mut err,
        );
        assert_eq!(code, 0);
        assert_eq!(
            seen,
            Some((app.clone(), vec!["open".into(), "ACME-142".into()]))
        );

        // Help goes to stdout and launches nothing.
        let mut out = Vec::new();
        let code = run(
            &os(&["--help"]),
            &helper,
            |_, _| panic!("must not launch"),
            &mut out,
            &mut err,
        );
        assert_eq!(code, 0);
        assert!(String::from_utf8_lossy(&out).contains("Usage: quire"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A fresh directory under the system temp dir (no extra dependency for one test).
    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "quire-cli-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
