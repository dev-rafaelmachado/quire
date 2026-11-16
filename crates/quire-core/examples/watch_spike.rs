//! T3 spike (disposable): file watcher delivery, polling cost and `ignore` scan cost.
//!
//! Usage: `watch_spike <dir> [--only events,poll,scan] [--reps N] [--files N] [--repos N]
//!        [--drop-caches] [--fs LABEL] [--keep]`
//!        `watch_spike <dir> --observe SECS [--poll-ms MS]`   (watch only, never writes)
//!
//! Everything is created under `<dir>/quire-watch-spike-<pid>/` and removed at the end
//! (unless `--keep`). Progress goes to stderr; the markdown tables go to stdout.
//! See `examples/README.md`.

use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant, SystemTime};

use cpu_time::ProcessTime;
use ignore::{DirEntry, WalkBuilder, WalkState};
use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
use notify::{Config, Event, EventKind, PollWatcher, RecursiveMode, Watcher};

type Res<T> = Result<T, Box<dyn Error>>;
type EventRx = Receiver<notify::Result<Event>>;
type FsOp = Box<dyn Fn() -> io::Result<()>>;

const IGNORED_DIRS: [&str; 4] = ["node_modules", "target", "dist", ".git"];

// ---------------------------------------------------------------- args

struct Args {
    root: PathBuf,
    only: Option<Vec<String>>,
    reps: usize,
    files: usize,
    repos: usize,
    drop_caches: bool,
    fs_label: Option<String>,
    keep: bool,
    observe_secs: Option<u64>,
    poll_ms: Option<u64>,
}

fn parse_args() -> Res<Args> {
    let mut it = std::env::args().skip(1);
    let mut args = Args {
        root: PathBuf::new(),
        only: None,
        reps: 20,
        files: 2000,
        repos: 50,
        drop_caches: false,
        fs_label: None,
        keep: false,
        observe_secs: None,
        poll_ms: None,
    };
    let mut root = None;
    while let Some(a) = it.next() {
        let mut val = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--only" => args.only = Some(val("--only")?.split(',').map(str::to_owned).collect()),
            "--reps" => args.reps = val("--reps")?.parse()?,
            "--files" => args.files = val("--files")?.parse()?,
            "--repos" => args.repos = val("--repos")?.parse()?,
            "--fs" => args.fs_label = Some(val("--fs")?),
            "--observe" => args.observe_secs = Some(val("--observe")?.parse()?),
            "--poll-ms" => args.poll_ms = Some(val("--poll-ms")?.parse()?),
            "--drop-caches" => args.drop_caches = true,
            "--keep" => args.keep = true,
            "-h" | "--help" => {
                eprintln!(
                    "usage: watch_spike <dir> [--only events,poll,scan] [--reps N] [--files N] [--repos N] [--drop-caches] [--fs LABEL] [--keep]\n       watch_spike <dir> --observe SECS [--poll-ms MS]"
                );
                std::process::exit(0);
            }
            other if root.is_none() && !other.starts_with("--") => {
                root = Some(PathBuf::from(other))
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    args.root = root.ok_or("missing <dir>")?;
    Ok(args)
}

impl Args {
    fn wants(&self, phase: &str) -> bool {
        self.only
            .as_ref()
            .is_none_or(|o| o.iter().any(|p| p == phase))
    }
}

// ---------------------------------------------------------------- environment

fn read(path: &str) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_owned())
}

/// Filesystem type of the mount that contains `path` (Linux: /proc/self/mountinfo).
fn detect_fs(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with(r"\\wsl$") || s.starts_with(r"\\wsl.localhost") {
        return "\\\\wsl$ (9p via Windows)".into();
    }
    let Some(info) = read("/proc/self/mountinfo") else {
        return "desconhecido (use --fs)".into();
    };
    let mut best: Option<(usize, String)> = None;
    for line in info.lines() {
        let fields: Vec<&str> = line.split(' ').collect();
        let (Some(mount), Some(sep)) = (fields.get(4), fields.iter().position(|f| *f == "-"))
        else {
            continue;
        };
        let mount = mount.replace("\\040", " ");
        let fstype = fields.get(sep + 1).copied().unwrap_or("?");
        if path.starts_with(&mount) && best.as_ref().is_none_or(|(len, _)| mount.len() > *len) {
            best = Some((mount.len(), format!("{fstype} ({mount})")));
        }
    }
    best.map_or_else(|| "desconhecido".into(), |(_, label)| label)
}

fn environment(args: &Args, dir: &Path) -> (String, String) {
    let release = read("/proc/sys/kernel/osrelease").unwrap_or_else(|| std::env::consts::OS.into());
    let wsl = release.to_lowercase().contains("microsoft")
        || std::env::var_os("WSL_DISTRO_NAME").is_some();
    let fs = args.fs_label.clone().unwrap_or_else(|| detect_fs(dir));
    let watches = read("/proc/sys/fs/inotify/max_user_watches").unwrap_or_else(|| "n/a".into());
    let header = format!(
        "**Ambiente:** {} {} · kernel `{}` · WSL: {} · FS: `{}` · inotify max_user_watches: {}",
        std::env::consts::OS,
        std::env::consts::ARCH,
        release,
        if wsl { "sim" } else { "não" },
        fs,
        watches
    );
    let short = fs.split(' ').next().unwrap_or("?").to_owned();
    (header, short)
}

// ---------------------------------------------------------------- helpers

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn pct(xs: &[f64], q: f64) -> f64 {
    let mut s = xs.to_vec();
    s.sort_by(f64::total_cmp);
    let idx = ((s.len() as f64 - 1.0) * q).round() as usize;
    s.get(idx).copied().unwrap_or(f64::NAN)
}

fn fmt_ms(xs: &[f64], q: f64) -> String {
    if xs.is_empty() {
        "—".into()
    } else {
        format!("{:.2} ms", pct(xs, q))
    }
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        f64::NAN
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn write_file(path: &Path, body: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, body)
}

fn drain(rx: &EventRx) {
    while rx.try_recv().is_ok() {}
}

fn kind_label(kind: &EventKind) -> String {
    match kind {
        EventKind::Create(_) => "create".into(),
        EventKind::Remove(_) => "remove".into(),
        EventKind::Modify(ModifyKind::Data(_)) => "modify:data".into(),
        EventKind::Modify(ModifyKind::Metadata(_)) => "modify:meta".into(),
        EventKind::Modify(ModifyKind::Name(mode)) => match mode {
            RenameMode::To => "rename:to".into(),
            RenameMode::From => "rename:from".into(),
            RenameMode::Both => "rename:both".into(),
            _ => "rename".into(),
        },
        EventKind::Modify(_) => "modify".into(),
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => "close-write".into(),
        EventKind::Access(_) => "access".into(),
        EventKind::Any => "any".into(),
        EventKind::Other => "other".into(),
    }
}

/// Waits for the first event that touches `target`, then keeps collecting kinds for 250 ms.
fn wait_for(
    rx: &EventRx,
    target: &Path,
    t0: Instant,
    timeout: Duration,
) -> Option<(Duration, BTreeSet<String>)> {
    let deadline = t0 + timeout;
    let mut first: Option<Duration> = None;
    let mut kinds = BTreeSet::new();
    loop {
        let limit = first.map_or(deadline, |f| {
            (t0 + f + Duration::from_millis(250)).min(deadline)
        });
        let now = Instant::now();
        if now >= limit {
            break;
        }
        match rx.recv_timeout(limit - now) {
            Ok(Ok(ev)) if ev.paths.iter().any(|p| p == target) => {
                first.get_or_insert_with(|| t0.elapsed());
                kinds.insert(kind_label(&ev.kind));
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => break,
        }
    }
    first.map(|f| (f, kinds))
}

fn canonical(p: &Path) -> io::Result<PathBuf> {
    // On Windows canonicalize() yields `\\?\` paths that notify does not echo back.
    if cfg!(windows) {
        Ok(p.to_path_buf())
    } else {
        p.canonicalize()
    }
}

// ---------------------------------------------------------------- 1. native events

struct OpStats {
    name: &'static str,
    arrived: usize,
    total: usize,
    lat: Vec<f64>,
    kinds: BTreeSet<String>,
}

fn events_test(base: &Path, reps: usize, fs: &str) -> Res<String> {
    let dir = base.join("events");
    fs::create_dir_all(&dir)?;
    let dir = canonical(&dir)?;
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&dir, RecursiveMode::Recursive)?;
    std::thread::sleep(Duration::from_millis(300));

    let names = [
        "criar arquivo",
        "editar in-place (truncate + write)",
        "rename atômico (tmp + rename)",
        "remover arquivo",
        "criar pasta",
        "arquivo em pasta recém-criada (mesmo instante)",
        "arquivo em pasta criada há 50 ms",
        "remover pasta",
    ];
    let mut stats: Vec<OpStats> = names
        .iter()
        .map(|n| OpStats {
            name: n,
            arrived: 0,
            total: 0,
            lat: vec![],
            kinds: BTreeSet::new(),
        })
        .collect();
    let timeout = Duration::from_secs(2);

    for rep in 0..reps {
        let file = dir.join(format!("spec-{rep}.md"));
        let tmp = dir.join(format!(".spec-{rep}.md.tmp"));
        let folder = dir.join(format!("feat-{rep}"));
        let folder2 = dir.join(format!("feat2-{rep}"));
        let nested = folder2.join("spec.md");
        let folder3 = dir.join(format!("feat3-{rep}"));
        let nested3 = folder3.join("spec.md");
        let ops: Vec<(PathBuf, FsOp)> = vec![
            (
                file.clone(),
                Box::new({
                    let f = file.clone();
                    move || fs::write(&f, "# spec\n")
                }),
            ),
            (
                file.clone(),
                Box::new({
                    let f = file.clone();
                    move || {
                        let mut h = fs::OpenOptions::new().write(true).truncate(true).open(&f)?;
                        h.write_all(b"# spec\n\nedited in place\n")
                    }
                }),
            ),
            (
                file.clone(),
                Box::new({
                    let (f, t) = (file.clone(), tmp.clone());
                    move || {
                        fs::write(&t, "# spec\n\nsaved by an editor\n")?;
                        fs::rename(&t, &f)
                    }
                }),
            ),
            (
                file.clone(),
                Box::new({
                    let f = file.clone();
                    move || fs::remove_file(&f)
                }),
            ),
            (
                folder.clone(),
                Box::new({
                    let d = folder.clone();
                    move || fs::create_dir(&d)
                }),
            ),
            (
                nested.clone(),
                Box::new({
                    let (d, n) = (folder2.clone(), nested.clone());
                    move || {
                        fs::create_dir(&d)?;
                        fs::write(&n, "# nested\n")
                    }
                }),
            ),
            (
                nested3.clone(),
                Box::new({
                    let (d, n) = (folder3.clone(), nested3.clone());
                    move || {
                        fs::create_dir(&d)?;
                        std::thread::sleep(Duration::from_millis(50));
                        fs::write(&n, "# nested\n")
                    }
                }),
            ),
            (
                folder.clone(),
                Box::new({
                    let d = folder.clone();
                    move || fs::remove_dir(&d)
                }),
            ),
        ];
        for (i, (target, op)) in ops.iter().enumerate() {
            std::thread::sleep(Duration::from_millis(60));
            drain(&rx);
            let t0 = Instant::now();
            op()?;
            let res = wait_for(&rx, target, t0, timeout);
            if let Some(s) = stats.get_mut(i) {
                s.total += 1;
                if let Some((lat, kinds)) = res {
                    s.arrived += 1;
                    s.lat.push(ms(lat));
                    s.kinds.extend(kinds);
                }
            }
        }
        fs::remove_dir_all(&folder2)?;
        fs::remove_dir_all(&folder3)?;
        eprint!("\r  eventos: {}/{reps}", rep + 1);
    }
    eprintln!();
    drop(watcher);

    let mut out = String::from(
        "| FS | Operação | Chegou | Latência p50 | Latência máx | Eventos no alvo |\n| --- | --- | ---: | ---: | ---: | --- |\n",
    );
    for s in &stats {
        let kinds: Vec<&str> = s.kinds.iter().map(String::as_str).collect();
        out.push_str(&format!(
            "| {fs} | {} | {}/{} | {} | {} | {} |\n",
            s.name,
            s.arrived,
            s.total,
            fmt_ms(&s.lat, 0.5),
            fmt_ms(&s.lat, 1.0),
            kinds.join(", ")
        ));
    }
    Ok(out)
}

// ---------------------------------------------------------------- 2. polling

fn make_poll_tree(dir: &Path, files: usize) -> io::Result<usize> {
    // 4 primary files per spec, 10 specs per repo: 2000 files = 50 repos.
    let mut n = 0;
    let body = "# synthetic\n".repeat(60);
    'outer: for repo in 0.. {
        for spec in 0..10 {
            for name in ["spec.md", "design.md", "tasks.md", "summary.md"] {
                if n >= files {
                    break 'outer;
                }
                write_file(
                    &dir.join(format!("repo-{repo:02}/.specs/features/feat-{spec}/{name}")),
                    &body,
                )?;
                n += 1;
            }
        }
    }
    Ok(n)
}

type Snapshot = HashMap<PathBuf, (Option<SystemTime>, u64)>;

fn snapshot(dir: &Path, out: &mut Snapshot) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            snapshot(&entry.path(), out)?;
        } else {
            let meta = entry.metadata()?;
            out.insert(entry.path(), (meta.modified().ok(), meta.len()));
        }
    }
    Ok(())
}

fn poll_test(base: &Path, files: usize, fs: &str) -> Res<String> {
    let dir = base.join("poll");
    let n = make_poll_tree(&dir, files)?;
    let dir = canonical(&dir)?;
    let mut out = format!(
        "{n} arquivos em {} repos sintéticos (só `.specs`).\n\n| FS | Modo | Intervalo | Tempo por ciclo (wall) | CPU por ciclo | CPU média | Latência de detecção |\n| --- | --- | ---: | ---: | ---: | ---: | ---: |\n",
        n.div_ceil(40)
    );

    // a) synchronous stat-walk (what a poller does each cycle): mtime + size snapshot and diff.
    let mut prev = Snapshot::new();
    snapshot(&dir, &mut prev)?;
    let (mut walls, mut cpus) = (vec![], vec![]);
    for _ in 0..15 {
        let (t0, c0) = (Instant::now(), ProcessTime::now());
        let mut cur = Snapshot::with_capacity(prev.len());
        snapshot(&dir, &mut cur)?;
        let changed = cur.iter().filter(|(k, v)| prev.get(*k) != Some(*v)).count();
        std::hint::black_box(changed);
        prev = cur;
        walls.push(ms(t0.elapsed()));
        cpus.push(ms(c0.elapsed()));
    }
    let cpu_cycle = mean(&cpus);
    for secs in [1u64, 2, 5] {
        out.push_str(&format!(
            "| {fs} | stat-walk síncrono (mtime + tamanho) | {secs} s | {:.2} ms (máx {:.2}) | {:.2} ms | {:.3}% | ≤ {secs} s (teórica) |\n",
            mean(&walls),
            pct(&walls, 1.0),
            cpu_cycle,
            cpu_cycle / (secs as f64 * 1000.0) * 100.0
        ));
    }

    // b) notify::PollWatcher at each interval: process CPU over a window and detection latency.
    for secs in [1u64, 2, 5] {
        let interval = Duration::from_secs(secs);
        let window = Duration::from_secs((secs * 3).max(10));
        let (tx, rx) = mpsc::channel();
        let mut watcher = PollWatcher::new(tx, Config::default().with_poll_interval(interval))?;
        let t_watch = Instant::now();
        watcher.watch(&dir, RecursiveMode::Recursive)?;
        let initial = t_watch.elapsed();
        let target = dir.join("repo-00/.specs/features/feat-0/tasks.md");
        let (start, c0) = (Instant::now(), ProcessTime::now());
        // Modify one file mid-window, at an offset that is not aligned to the interval.
        std::thread::sleep(window / 2 + Duration::from_millis(337));
        drain(&rx);
        let t0 = Instant::now();
        fs::OpenOptions::new()
            .append(true)
            .open(&target)?
            .write_all(b"- [x] T1: done\n")?;
        let detected = wait_for(&rx, &target, t0, interval * 3);
        let remaining = window.saturating_sub(start.elapsed());
        std::thread::sleep(remaining);
        let cpu = c0.elapsed();
        let wall = start.elapsed();
        drop(watcher);
        out.push_str(&format!(
            "| {fs} | notify `PollWatcher` (scan inicial {:.0} ms) | {secs} s | — | — | {:.3}% | {} |\n",
            ms(initial),
            cpu.as_secs_f64() / wall.as_secs_f64() * 100.0,
            detected.map_or_else(|| "não detectou".into(), |(d, _)| format!("{:.0} ms", ms(d)))
        ));
        eprintln!("  polling {secs}s: ok");
    }
    Ok(out)
}

// ---------------------------------------------------------------- 3. scan with `ignore`

struct ScanTree {
    repos: usize,
    files: usize,
    dirs: usize,
}

fn make_repos(dir: &Path, repos: usize) -> io::Result<ScanTree> {
    let mut files = 0;
    let mut w = |p: PathBuf, body: &str| -> io::Result<()> {
        files += 1;
        write_file(&p, body)
    };
    for r in 0..repos {
        let repo = dir.join(format!("repo-{r:02}"));
        w(repo.join(".git/HEAD"), "ref: refs/heads/main\n")?;
        for o in 0..40 {
            w(
                repo.join(format!(".git/objects/{:02x}/{o:038x}", o % 16)),
                "blob",
            )?;
        }
        // Every 10th repo has no .gitignore: the hardcoded ignore list must still apply.
        if r % 10 != 9 {
            w(
                repo.join(".gitignore"),
                "node_modules/\ntarget/\ndist/\n*.log\n",
            )?;
        }
        for s in 0..30 {
            w(
                repo.join(format!("src/mod{}/file{s}.ts", s % 3)),
                "export const x = 1;\n",
            )?;
        }
        for p in 0..20 {
            w(
                repo.join(format!("node_modules/pkg-{p}/package.json")),
                "{}",
            )?;
            w(
                repo.join(format!("node_modules/pkg-{p}/index.js")),
                "module.exports = 1;\n",
            )?;
            for f in 0..8 {
                w(
                    repo.join(format!("node_modules/pkg-{p}/lib/f{f}.js")),
                    "1;\n",
                )?;
            }
        }
        for t in 0..100 {
            w(
                repo.join(format!("target/debug/deps/{}/obj{t}.o", t % 10)),
                "obj",
            )?;
        }
        for d in 0..10 {
            w(repo.join(format!("dist/chunk{d}.js")), "1;\n")?;
        }
        // Decoys that must never be reported.
        w(
            repo.join("node_modules/pkg-0/.specs/features/decoy/spec.md"),
            "# decoy\n",
        )?;
        w(
            repo.join("target/debug/.specs/features/decoy/spec.md"),
            "# decoy\n",
        )?;
        w(repo.join("dist/.specs/features/decoy/spec.md"), "# decoy\n")?;
        for s in 0..3 {
            for name in [
                "spec.md",
                "design.md",
                "tasks.md",
                "summary.md",
                "validation.md",
                "gates.jsonl",
                "lessons.jsonl",
            ] {
                w(
                    repo.join(format!(
                        ".specs/features/ACME-{}{s}-feature/{name}",
                        100 + r
                    )),
                    "# x\n",
                )?;
            }
        }
    }
    let mut dirs = 0;
    count_dirs(dir, &mut dirs)?;
    Ok(ScanTree { repos, files, dirs })
}

fn count_dirs(dir: &Path, n: &mut usize) -> io::Result<()> {
    *n += 1;
    for e in fs::read_dir(dir)? {
        let e = e?;
        if e.file_type()?.is_dir() {
            count_dirs(&e.path(), n)?;
        }
    }
    Ok(())
}

fn keep_entry(e: &DirEntry) -> bool {
    !(e.file_type().is_some_and(|t| t.is_dir()) && IGNORED_DIRS.iter().any(|d| e.file_name() == *d))
}

fn builder(root: &Path) -> WalkBuilder {
    let mut b = WalkBuilder::new(root);
    b.hidden(false)
        .git_ignore(true)
        .git_exclude(false)
        .git_global(false)
        .parents(false)
        .filter_entry(keep_entry);
    b
}

struct WalkResult {
    entries: usize,
    specs: Vec<PathBuf>,
    decoys: usize,
    time: Duration,
}

fn is_decoy(p: &Path) -> bool {
    p.components()
        .any(|c| IGNORED_DIRS.iter().any(|d| c.as_os_str() == *d))
}

fn walk_ignore(root: &Path) -> WalkResult {
    let t0 = Instant::now();
    let (mut entries, mut specs) = (0, vec![]);
    for e in builder(root).build().flatten() {
        entries += 1;
        if e.file_type().is_some_and(|t| t.is_dir()) && e.file_name() == ".specs" {
            specs.push(e.into_path());
        }
    }
    let decoys = specs.iter().filter(|p| is_decoy(p)).count();
    WalkResult {
        entries,
        specs,
        decoys,
        time: t0.elapsed(),
    }
}

fn walk_ignore_parallel(root: &Path) -> WalkResult {
    let t0 = Instant::now();
    let entries = AtomicUsize::new(0);
    let specs = AtomicUsize::new(0);
    let decoys = AtomicUsize::new(0);
    builder(root).build_parallel().run(|| {
        Box::new(|e| {
            if let Ok(e) = e {
                entries.fetch_add(1, Ordering::Relaxed);
                if e.file_type().is_some_and(|t| t.is_dir()) && e.file_name() == ".specs" {
                    specs.fetch_add(1, Ordering::Relaxed);
                    if is_decoy(e.path()) {
                        decoys.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            WalkState::Continue
        })
    });
    WalkResult {
        entries: entries.into_inner(),
        specs: vec![PathBuf::new(); specs.into_inner()],
        decoys: decoys.into_inner(),
        time: t0.elapsed(),
    }
}

/// Baseline without any ignore rules: every directory, including node_modules/target/.git.
fn walk_naive(root: &Path) -> io::Result<WalkResult> {
    fn go(d: &Path, r: &mut WalkResult) -> io::Result<()> {
        for e in fs::read_dir(d)? {
            let e = e?;
            r.entries += 1;
            if e.file_type()?.is_dir() {
                let p = e.path();
                if e.file_name() == ".specs" {
                    if is_decoy(&p) {
                        r.decoys += 1;
                    }
                    r.specs.push(p.clone());
                }
                go(&p, r)?;
            }
        }
        Ok(())
    }
    let t0 = Instant::now();
    let mut r = WalkResult {
        entries: 0,
        specs: vec![],
        decoys: 0,
        time: Duration::ZERO,
    };
    go(root, &mut r)?;
    r.time = t0.elapsed();
    Ok(r)
}

fn drop_caches() -> bool {
    Command::new("sh")
        .args([
            "-c",
            "sync && sudo -n sh -c 'echo 3 > /proc/sys/vm/drop_caches'",
        ])
        .status()
        .is_ok_and(|s| s.success())
}

fn scan_test(base: &Path, repos: usize, want_cold: bool, fs: &str) -> Res<String> {
    let dir = base.join("repos");
    let tree = make_repos(&dir, repos)?;
    let dir = canonical(&dir)?;
    let mut out = format!(
        "{} repos sintéticos, {} arquivos, {} diretórios (com `.git`, `node_modules`, `target` e `dist` em cada repo; 1 em cada 10 sem `.gitignore`; 3 pastas `.specs` de isca por repo dentro das pastas ignoradas).\n\n| FS | Varredura | Cache | Entradas visitadas | `.specs` achadas | Iscas reportadas | Tempo (mediana de 5) |\n| --- | --- | --- | ---: | ---: | ---: | ---: |\n",
        tree.repos, tree.files, tree.dirs
    );
    let row = |name: &str, cache: &str, r: &WalkResult, t: f64| {
        format!(
            "| {fs} | {name} | {cache} | {} | {} | {} | {t:.1} ms |\n",
            r.entries,
            r.specs.len(),
            r.decoys
        )
    };

    if want_cold {
        if drop_caches() {
            let r = walk_ignore(&dir);
            out.push_str(&row("`ignore` sequencial", "frio", &r, ms(r.time)));
            if drop_caches() {
                let r = walk_ignore_parallel(&dir);
                out.push_str(&row("`ignore` paralelo", "frio", &r, ms(r.time)));
            }
            if drop_caches() {
                let r = walk_naive(&dir)?;
                out.push_str(&row(
                    "sem regras de ignore (baseline)",
                    "frio",
                    &r,
                    ms(r.time),
                ));
            }
        } else {
            out.push_str(&format!(
                "| {fs} | — | frio | — | — | — | não medido (`sudo -n` falhou) |\n"
            ));
        }
    }
    let mut times = vec![];
    let mut last = walk_ignore(&dir);
    for _ in 0..5 {
        last = walk_ignore(&dir);
        times.push(ms(last.time));
    }
    out.push_str(&row(
        "`ignore` sequencial",
        "quente",
        &last,
        pct(&times, 0.5),
    ));
    let specs = last.specs.clone();
    times.clear();
    for _ in 0..5 {
        last = walk_ignore_parallel(&dir);
        times.push(ms(last.time));
    }
    out.push_str(&row("`ignore` paralelo", "quente", &last, pct(&times, 0.5)));
    times.clear();
    for _ in 0..5 {
        last = walk_naive(&dir)?;
        times.push(ms(last.time));
    }
    out.push_str(&row(
        "sem regras de ignore (baseline)",
        "quente",
        &last,
        pct(&times, 0.5),
    ));
    eprintln!("  scan: ok");

    // Watch setup cost: whole tree recursively vs only the .specs folders.
    out.push_str("\n| FS | Alvo do watch nativo | Diretórios observados | Tempo de setup |\n| --- | --- | ---: | ---: |\n");
    let (tx, _rx) = mpsc::channel();
    let mut w = notify::recommended_watcher(tx)?;
    let t0 = Instant::now();
    w.watch(&dir, RecursiveMode::Recursive)?;
    out.push_str(&format!(
        "| {fs} | raiz dos repos, recursivo | {} | {:.1} ms |\n",
        tree.dirs,
        ms(t0.elapsed())
    ));
    drop(w);
    let (tx, _rx) = mpsc::channel();
    let mut w = notify::recommended_watcher(tx)?;
    let mut spec_dirs = 0;
    let t0 = Instant::now();
    for s in &specs {
        w.watch(s, RecursiveMode::Recursive)?;
    }
    let setup = t0.elapsed();
    for s in &specs {
        count_dirs(s, &mut spec_dirs)?;
    }
    out.push_str(&format!(
        "| {fs} | só as pastas `.specs`, recursivo | {spec_dirs} | {:.1} ms |\n",
        ms(setup)
    ));
    Ok(out)
}

// ---------------------------------------------------------------- observe mode (never writes)

fn observe(root: &Path, secs: u64, poll_ms: Option<u64>) -> Res<()> {
    let root = canonical(root)?;
    let (tx, rx) = mpsc::channel();
    let mut watcher: Box<dyn Watcher> = match poll_ms {
        Some(p) => Box::new(PollWatcher::new(
            tx,
            Config::default().with_poll_interval(Duration::from_millis(p)),
        )?),
        None => Box::new(notify::recommended_watcher(tx)?),
    };
    watcher.watch(&root, RecursiveMode::Recursive)?;
    let backend = poll_ms.map_or_else(|| "nativo".to_owned(), |p| format!("polling {p} ms"));
    eprintln!(
        "observando {} ({backend}) por {secs} s; altere arquivos de outro lado (ex.: Windows) e veja os eventos",
        root.display()
    );
    let start = Instant::now();
    let end = start + Duration::from_secs(secs);
    let mut count = 0;
    while let Some(left) = end.checked_duration_since(Instant::now()) {
        match rx.recv_timeout(left) {
            Ok(Ok(ev)) => {
                count += 1;
                let paths: Vec<String> = ev.paths.iter().map(|p| p.display().to_string()).collect();
                println!(
                    "{:>9.3}s  {:<12} {}",
                    start.elapsed().as_secs_f64(),
                    kind_label(&ev.kind),
                    paths.join(" -> ")
                );
            }
            Ok(Err(e)) => println!("erro do watcher: {e}"),
            Err(_) => break,
        }
    }
    println!("{count} eventos em {secs} s");
    Ok(())
}

// ---------------------------------------------------------------- main

fn main() -> Res<()> {
    let args = parse_args()?;
    if let Some(secs) = args.observe_secs {
        return observe(&args.root, secs, args.poll_ms);
    }
    let base = args
        .root
        .join(format!("quire-watch-spike-{}", std::process::id()));
    fs::create_dir_all(&base)?;
    let (header, fs) = environment(&args, &canonical(&base)?);
    let mut report = format!("### Spike T3: watcher e scan\n\n{header}\n\n");

    let result = (|| -> Res<()> {
        if args.wants("events") {
            eprintln!(
                "[1/3] eventos nativos ({} repetições por operação)",
                args.reps
            );
            report.push_str(&format!(
                "#### Eventos nativos ({} repetições por operação)\n\n",
                args.reps
            ));
            report.push_str(&events_test(&base, args.reps, &fs)?);
            report.push('\n');
        }
        if args.wants("poll") {
            eprintln!("[2/3] custo do polling");
            report.push_str("#### Polling\n\n");
            report.push_str(&poll_test(&base, args.files, &fs)?);
            report.push('\n');
        }
        if args.wants("scan") {
            eprintln!("[3/3] varredura com `ignore`");
            report.push_str("#### Varredura\n\n");
            report.push_str(&scan_test(&base, args.repos, args.drop_caches, &fs)?);
        }
        Ok(())
    })();

    if !args.keep {
        fs::remove_dir_all(&base)?;
    }
    result?;
    print!("{report}");
    Ok(())
}
