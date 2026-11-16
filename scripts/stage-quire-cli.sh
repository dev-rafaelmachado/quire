#!/usr/bin/env bash
# Builds the console helper and puts it where tauri.bundle-windows.conf.json (bundle.externalBin)
# expects it: apps/desktop/src-tauri/binaries/quire-cli-<target triple>[.exe]. Run it before
# `tauri build --config <absolute path to tauri.bundle-windows.conf.json>` (TD-20). That config
# is passed only when bundling, so `cargo check`, `clippy`, `test` and `tauri dev` never need it.
set -euo pipefail
cd "$(dirname "$0")/.."
triple=$(rustc -vV | sed -n 's/^host: //p')
cargo build --release -p quire-cli
ext=""
[[ "$triple" == *windows* ]] && ext=".exe"
mkdir -p apps/desktop/src-tauri/binaries
cp "target/release/quire-cli$ext" "apps/desktop/src-tauri/binaries/quire-cli-$triple$ext"
echo "staged apps/desktop/src-tauri/binaries/quire-cli-$triple$ext"
