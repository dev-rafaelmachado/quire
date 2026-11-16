#!/usr/bin/env bash
# Builds the console helper and puts it where tauri.windows.conf.json (bundle.externalBin)
# expects it: apps/desktop/src-tauri/binaries/quire-cli-<target triple>[.exe]. Run it from the
# repository root before `tauri build` on Windows (TD-20). Linux and macOS bundles do not use it.
set -euo pipefail
cd "$(dirname "$0")/.."
triple=$(rustc -vV | sed -n 's/^host: //p')
cargo build --release -p quire-cli
ext=""
[[ "$triple" == *windows* ]] && ext=".exe"
mkdir -p apps/desktop/src-tauri/binaries
cp "target/release/quire-cli$ext" "apps/desktop/src-tauri/binaries/quire-cli-$triple$ext"
echo "staged apps/desktop/src-tauri/binaries/quire-cli-$triple$ext"
