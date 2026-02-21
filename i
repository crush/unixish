#!/usr/bin/env sh
set -eu

repo="https://github.com/crush/unixish"

if command -v cargo >/dev/null 2>&1; then
  cargo install --git "$repo" unixish
  exit 0
fi

echo "cargo"
exit 1
