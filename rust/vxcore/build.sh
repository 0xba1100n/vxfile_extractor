#!/bin/bash
# Local build helper: produces whatever targets this machine can actually build.
set -e
cd "$(dirname "$0")/.."
TARGETS="${*:-x86_64-unknown-linux-gnu}"
mkdir -p dist
for t in $TARGETS; do
  echo "== $t"
  cargo build --release --offline --target "$t"
  ext=""
  case "$t" in *windows*) ext=".exe";; esac
  cp "target/$t/release/vx$ext" "dist/vx-$t$ext"
done
ls -la dist