#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
N="${1:-50}"
if [[ ! -x "${SEMAURI_BIN:-target/release/semauri}" ]]; then
  cargo build --release
fi
python3 scripts/bench_driver.py "$N"
