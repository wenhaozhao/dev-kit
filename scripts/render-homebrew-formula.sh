#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  echo "usage: $0 VERSION ARM64_SHA256 X86_64_SHA256 OUTPUT_PATH" >&2
  exit 1
fi

version="$1"
arm64_sha="$2"
x86_64_sha="$3"
output="$4"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

sed -e "s/__VERSION__/$version/g" -e "s/__ARM64_SHA256__/$arm64_sha/g" -e "s/__X86_64_SHA256__/$x86_64_sha/g" "$root/packaging/homebrew/devkit.rb.template" > "$output"
