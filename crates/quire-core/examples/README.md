# quire-core examples

## `watch_spike.rs` (T3 spike, disposable)

Measures, on the filesystem of a directory you choose:

1. **Native events** (`notify` recommended backend: inotify / ReadDirectoryChangesW / FSEvents):
   for create, in-place edit, atomic editor save (tmp + rename), remove, create folder, file in
   a just-created folder and remove folder, whether the event arrived and its latency.
2. **Polling cost** with ~2000 files: a synchronous mtime + size stat-walk (cost per cycle) and
   `notify::PollWatcher` at 1 s, 2 s and 5 s (process CPU and detection latency).
3. **Scan cost** with the `ignore` crate over ~50 synthetic repos that contain `.git`,
   `node_modules`, `target` and `dist` (with decoy `.specs` folders inside them), warm and,
   with `--drop-caches`, cold; plus the cost of setting up a recursive native watch.

Everything is created under `<dir>/quire-watch-spike-<pid>/` and deleted at the end
(`--keep` keeps it). Tables go to stdout, ready to paste into `summary.md`; the filesystem
type is detected from `/proc/self/mountinfo` (override with `--fs LABEL`).
Results so far: `.specs/features/quire-v0-1/summary.md`, "Resultados dos spikes › T3".

### Run

Linux, WSL (Linux side) or Git Bash:

```bash
scripts/watch-spike.sh <dir> [--drop-caches] [--fs LABEL] [--only events,poll,scan] [--reps N]
```

Windows native, e.g. for a `\\wsl$` path (PowerShell, from the repo root):

```powershell
cargo run --release -p quire-core --example watch_spike -- '\\wsl$\Ubuntu\home\me\tmp' --fs 'wsl-unc'
```

### Scenarios still to run (TD-05)

| Where Quire runs | Directory | Command |
| --- | --- | --- |
| WSL2 (Linux app) | WSL ext4, e.g. `~/tmp` | `scripts/watch-spike.sh ~/tmp` |
| WSL2 (Linux app) | DrvFs, e.g. `/mnt/c/tmp` | `scripts/watch-spike.sh /mnt/c/tmp` |
| Windows native | `\\wsl$\<distro>\...` | PowerShell command above |
| Windows native | NTFS, e.g. `C:\tmp` | `cargo run --release -p quire-core --example watch_spike -- C:\tmp` |

The automatic test writes from the same side that watches. Editors and agents often write
from the *other* side (a Windows editor on a WSL file, or a WSL agent on `C:\`), which is the
case where events are known to go missing. Check it with the observe mode, which never writes:

```bash
scripts/watch-spike.sh /mnt/c/tmp --observe 60              # native backend
scripts/watch-spike.sh /mnt/c/tmp --observe 60 --poll-ms 3000   # polling, for comparison
```

and, while it runs, create, edit, save atomically and delete files in that folder from the
other side. Note which operations print an event.
