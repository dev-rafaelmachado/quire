//! `quire-cli`: see the library docs. Installed on Windows as `quire-cli.exe` next to
//! `quire.exe`, and called by `bin\quire.cmd`, which is what the `PATH` points to.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};

/// Starts the app detached from this console and returns at once; the single instance of the
/// app takes it from there (CLI-04).
fn launch(app: &Path, args: &[OsString]) -> std::io::Result<()> {
    let mut cmd = Command::new(app);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Ok(cwd) = std::env::current_dir() {
        cmd.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn().map(|_| ())
}

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let helper = std::env::current_exe().unwrap_or_default();
    let code = quire_cli::run(
        &args,
        &helper,
        launch,
        &mut std::io::stdout(),
        &mut std::io::stderr(),
    );
    std::process::exit(code);
}
