#!/usr/bin/env bash
# Copy LICENSE/README into the wrapper and every platform package.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cli="$root/cli-npm"
cp "$root/LICENSE" "$cli/LICENSE"
cp "$root/FORK.md" "$cli/packages/.fork.md"
for dir in "$cli"/packages/*/; do
  cp "$root/LICENSE" "$dir/LICENSE"
  cat > "$dir/README.md" <<EOF
# $(node -p "require('$dir/package.json').name")

Native \`componentize-qjs\` CLI for this platform. Install
\`@di-framework/componentize-qjs\` instead of this package directly.

See https://github.com/di-framework/componentize-qjs
EOF
done
