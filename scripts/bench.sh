#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
N="${1:-50}"
if [[ ! -x "${SEMAURI_BIN:-target/release/semauri}" || ! -x target/release/semauri_bench ]]; then
  cargo build --release
fi
target/release/semauri_bench suite "$N"
