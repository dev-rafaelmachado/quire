//! T11 spike (v0.2, disposable): what "temp file + fsync + rename over" really does on the file
//! system of a folder. Decides P6 (writes on `\\wsl$`, DrvFs, 9p) and checks the assumptions of
//! the write pipeline (TD-12): atomicity for a concurrent reader, permissions, symbolic links,
//! a file held open by another process (Windows sharing modes), and the watcher events.
//!
//! ```text
//! cargo run --release -p quire-core --example rename_spike -- <folder on the FS to test>
//! ```
//!
//! It only writes inside `<folder>/quire-rename-spike-<pid>/`, and removes it at the end. The
//! exit code is 1 when a reader ever saw an empty, partial or missing file.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use notify::{RecursiveMode, Watcher};

const ROUNDS: usize = 400;
const SIZE: usize = 256 * 1024;

fn content(tag: u8) -> Vec<u8> {
    let mut v = vec![tag; SIZE];
    v.extend_from_slice(b"\nEND\n");
    v
}

/// The write of the design: hidden temp in the same folder, write, fsync, rename, fsync dir.
fn replace(target: &Path, bytes: &[u8], n: usize) -> std::io::Result<()> {
    let dir = target.parent().unwrap();
    let name = target.file_name().unwrap().to_string_lossy();
    let tmp = dir.join(format!(".{name}.quire-{}-{n}.tmp", std::process::id()));
    let result = (|| {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, target)?;
        #[cfg(unix)]
        File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn mountinfo_line(path: &Path) -> String {
    #[cfg(target_os = "linux")]
    {
        let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let text = fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
        let mut best: Option<(usize, String)> = None;
        for line in text.lines() {
            let mount = line.split(' ').nth(4).unwrap_or("").replace("\\040", " ");
            if canon.starts_with(&mount) && best.as_ref().is_none_or(|(l, _)| mount.len() > *l) {
                best = Some((mount.len(), line.to_owned()));
            }
        }
        best.map(|(_, l)| l).unwrap_or_else(|| "(not found)".into())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = path;
        "(not Linux)".into()
    }
}

fn atomicity(dir: &Path) -> bool {
    let target = dir.join("spec.md");
    let (a, b) = (content(b'a'), content(b'b'));
    fs::write(&target, &a).unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let reads = Arc::new(AtomicUsize::new(0));
    let bad: Arc<Mutex<Vec<String>>> = Arc::default();
    let reader = {
        let (stop, reads, bad, target) = (stop.clone(), reads.clone(), bad.clone(), target.clone());
        let (a, b) = (a.clone(), b.clone());
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                let mut buf = Vec::new();
                match File::open(&target).and_then(|mut f| f.read_to_end(&mut buf)) {
                    Ok(_) if buf == a || buf == b => {}
                    Ok(_) => bad
                        .lock()
                        .unwrap()
                        .push(format!("partial read of {} bytes", buf.len())),
                    Err(e) => bad.lock().unwrap().push(format!("read error: {e}")),
                }
                reads.fetch_add(1, Ordering::Relaxed);
            }
        })
    };
    let t = Instant::now();
    let mut write_errors = Vec::new();
    for n in 0..ROUNDS {
        if let Err(e) = replace(&target, if n % 2 == 0 { &b } else { &a }, n) {
            write_errors.push(format!("round {n}: {e} (os error {:?})", e.raw_os_error()));
        }
    }
    let elapsed = t.elapsed();
    stop.store(true, Ordering::Relaxed);
    reader.join().unwrap();
    let bad = bad.lock().unwrap();
    println!(
        "atomicity: {ROUNDS} replacements of {} KB in {:.0} ms ({:.2} ms each), {} concurrent reads, {} bad reads, {} write errors",
        SIZE / 1024,
        elapsed.as_secs_f64() * 1e3,
        elapsed.as_secs_f64() * 1e3 / ROUNDS as f64,
        reads.load(Ordering::Relaxed),
        bad.len(),
        write_errors.len()
    );
    for b in bad.iter().take(5) {
        println!("  bad read: {b}");
    }
    for e in write_errors.iter().take(5) {
        println!("  write error: {e}");
    }
    let leftovers: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".tmp"))
        .collect();
    println!("  temp files left: {}", leftovers.len());
    bad.is_empty()
}

fn permissions(dir: &Path) {
    let target = dir.join("perm.md");
    fs::write(&target, b"x").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
        replace(&target, b"y", 9_001).unwrap();
        let mode = fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        println!(
            "permissions: original 0640, after rename {mode:04o} -> {}",
            if mode == 0o640 {
                "kept by the FS"
            } else {
                "NOT kept: the pipeline must copy the mode (step 5)"
            }
        );
    }
    let mut ro = fs::metadata(&target).unwrap().permissions();
    ro.set_readonly(true);
    fs::set_permissions(&target, ro).unwrap();
    match replace(&target, b"z", 9_002) {
        Ok(()) => println!(
            "read-only target: rename over it SUCCEEDED (readonly now {}) -> the pipeline must refuse before (SAV-04)",
            fs::metadata(&target).unwrap().permissions().readonly()
        ),
        Err(e) => println!(
            "read-only target: rename over it failed: {e} (os error {:?})",
            e.raw_os_error()
        ),
    }
    let mut rw = fs::metadata(&target).unwrap().permissions();
    #[allow(clippy::permissions_set_readonly_false)]
    rw.set_readonly(false);
    fs::set_permissions(&target, rw).unwrap();
}

fn symlink(dir: &Path) {
    #[cfg(unix)]
    {
        let real = dir.join("real.md");
        let link = dir.join("link.md");
        fs::write(&real, b"real").unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        replace(&link, b"through link", 9_003).unwrap();
        let still_link = fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink();
        println!(
            "symlink: rename over the link -> link is {} and the target holds {:?} -> the pipeline must resolve the link first (SAV-05)",
            if still_link {
                "still a link"
            } else {
                "REPLACED by a regular file"
            },
            String::from_utf8_lossy(&fs::read(&real).unwrap())
        );
    }
    #[cfg(not(unix))]
    {
        let _ = dir;
        println!("symlink: not checked on this OS");
    }
}

fn held_open(dir: &Path) {
    let target = dir.join("open.md");
    fs::write(&target, b"old").unwrap();
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x1;
        const FILE_SHARE_WRITE: u32 = 0x2;
        const FILE_SHARE_DELETE: u32 = 0x4;
        for (label, share) in [
            ("read|write (no delete)", FILE_SHARE_READ | FILE_SHARE_WRITE),
            (
                "read|write|delete",
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            ),
        ] {
            let held = OpenOptions::new()
                .read(true)
                .share_mode(share)
                .open(&target)
                .unwrap();
            let r = replace(&target, b"new", 9_100 + share as usize);
            println!(
                "held open with share {label}: rename {}",
                match &r {
                    Ok(()) => "succeeded".to_owned(),
                    Err(e) => format!("failed: {e} (os error {:?})", e.raw_os_error()),
                }
            );
            drop(held);
        }
    }
    #[cfg(not(windows))]
    {
        let mut held = File::open(&target).unwrap();
        replace(&target, b"new", 9_100).unwrap();
        let mut old = String::new();
        held.read_to_string(&mut old).unwrap();
        println!(
            "held open (Unix): rename succeeded; the open handle still reads {old:?}, the path reads {:?}",
            fs::read_to_string(&target).unwrap()
        );
    }
}

fn events(dir: &Path) {
    let target = dir.join("watched.md");
    fs::write(&target, b"v1").unwrap();
    let seen: Arc<Mutex<Vec<String>>> = Arc::default();
    let sink = seen.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(e) = res {
            let names: Vec<String> = e
                .paths
                .iter()
                .map(|p| {
                    p.file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                })
                .collect();
            sink.lock().unwrap().push(format!("{:?} {names:?}", e.kind));
        }
    })
    .unwrap();
    watcher.watch(dir, RecursiveMode::NonRecursive).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    replace(&target, b"v2", 9_200).unwrap();
    std::thread::sleep(Duration::from_millis(800));
    drop(watcher);
    let seen = seen.lock().unwrap();
    println!("watcher events for one replacement ({}):", seen.len());
    for e in seen.iter() {
        println!("  {e}");
    }
}

fn main() {
    let base: PathBuf = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = base.join(format!("quire-rename-spike-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    println!("folder: {}", dir.display());
    println!("os: {} {}", std::env::consts::OS, std::env::consts::ARCH);
    println!("mountinfo: {}", mountinfo_line(&dir));
    let ok = atomicity(&dir);
    permissions(&dir);
    symlink(&dir);
    held_open(&dir);
    events(&dir);
    fs::remove_dir_all(&dir).unwrap();
    std::process::exit(if ok { 0 } else { 1 });
}
