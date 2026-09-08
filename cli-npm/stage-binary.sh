#!/usr/bin/env bash
# Copy a built CLI into cli-npm/packages/<npm-platform>/bin/
# Usage: stage-binary.sh <npm-platform> <binary-path>
set -euo pipefail
root="$(cd "$(dirname "$0")" && pwd)"
platform="${1:?npm platform (darwin-arm64, linux-x64, ...)}"
binary="${2:?path to componentize-qjs binary}"
dest="$root/packages/$platform"
test -f "$dest/package.json" || { echo "unknown platform $platform" >&2; exit 1; }
test -f "$binary" || { echo "binary not found: $binary" >&2; exit 1; }
mkdir -p "$dest/bin"
name="componentize-qjs"
if [[ "$platform" == win32-* ]]; then
  name="componentize-qjs.exe"
fi
cp "$binary" "$dest/bin/$name"
chmod +x "$dest/bin/$name"
echo "staged $binary -> $dest/bin/$name"
