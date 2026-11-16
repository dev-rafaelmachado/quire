#!/usr/bin/env bash
# T3 spike runner (disposable): builds and runs crates/quire-core/examples/watch_spike.rs
# against a directory, printing markdown tables and saving them to a file.
#
# Usage: scripts/watch-spike.sh <dir> [spike flags...]
#   scripts/watch-spike.sh ~/tmp                      # events + polling + scan (warm)
#   scripts/watch-spike.sh ~/tmp --drop-caches        # also cold scan (needs sudo; asks once)
#   scripts/watch-spike.sh /mnt/c/tmp --fs drvfs      # override the filesystem label
#   scripts/watch-spike.sh /mnt/c/tmp --observe 60    # only watch and print events (no writes)
# The output file can be set with OUT=path; default: $TMPDIR/quire-watch-spike-<timestamp>.md
set -euo pipefail

if [ $# -lt 1 ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 2
fi

repo_root=$(cd "$(dirname "$0")/.." && pwd)
target=$1
shift
[ -d "$target" ] || { echo "not a directory: $target" >&2; exit 1; }

# --drop-caches needs root; ask for the password up front so the spike can use `sudo -n`.
for arg in "$@"; do
  if [ "$arg" = "--drop-caches" ]; then sudo -v; fi
done

out=${OUT:-${TMPDIR:-/tmp}/quire-watch-spike-$(date +%Y%m%d-%H%M%S).md}
cargo build --release --quiet --manifest-path "$repo_root/Cargo.toml" -p quire-core --example watch_spike
cargo run --release --quiet --manifest-path "$repo_root/Cargo.toml" -p quire-core --example watch_spike -- \
  "$target" "$@" | tee "$out"
echo "saved to $out" >&2
