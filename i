#!/usr/bin/env sh
set -eu

repo="https://github.com/crush/unixish"

if command -v cargo >/dev/null 2>&1; then
  cargo install --git "$repo" unixish
  exit 0
fi

if command -v powershell >/dev/null 2>&1; then
  powershell -ExecutionPolicy Bypass -Command "irm https://raw.githubusercontent.com/crush/unixish/main/w | iex"
  exit 0
fi

echo "install failed"
exit 1
