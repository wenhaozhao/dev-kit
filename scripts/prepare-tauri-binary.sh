#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-$(rustc -vV | sed -n 's/^host: //p')}"
binary_dir="$root/dkui/src-tauri/binaries"

binary_name=devkit
if [[ "$target" == *windows* ]]; then
  binary_name=devkit.exe
fi

cargo build --manifest-path "$root/Cargo.toml" -p dev-kit --release --target "$target"
mkdir -p "$binary_dir"
sidecar_name="devkit-$target"
if [[ "$target" == *windows* ]]; then
  sidecar_name="$sidecar_name.exe"
fi
cp "$root/target/$target/release/$binary_name" "$binary_dir/$sidecar_name"

echo "prepared $binary_dir/$sidecar_name"
