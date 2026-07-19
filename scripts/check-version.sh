#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo_version="$(sed -n 's/^version = "\([^"]*\)"$/\1/p' Cargo.toml | head -n 1)"
npm_version="$(node -p "require('./dkui/package.json').version")"
tauri_version="$(node -p "require('./dkui/src-tauri/tauri.conf.json').version")"

for named_version in "npm:$npm_version" "tauri:$tauri_version"; do
  name="${named_version%%:*}"
  version="${named_version#*:}"
  if [[ "$version" != "$cargo_version" ]]; then
    echo "version mismatch: Cargo is $cargo_version but $name is $version" >&2
    exit 1
  fi
done

echo "version $cargo_version is consistent across Cargo, npm, and Tauri"
