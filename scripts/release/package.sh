#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
SEMAURI_PLATFORM=${SEMAURI_PLATFORM:?SEMAURI_PLATFORM must name the target platform}
SEMAURI_BINARY=${SEMAURI_BINARY:-"$ROOT_DIR/target/release/semauri"}
OUT_DIR=${OUT_DIR:-"$ROOT_DIR/dist"}

VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT_DIR/Cargo.toml" | head -n 1)
if [[ -z "$VERSION" ]]; then
  echo "Unable to determine Semauri version from Cargo.toml" >&2
  exit 1
fi

if [[ ! -x "$SEMAURI_BINARY" ]]; then
  echo "Semauri binary is missing or not executable: $SEMAURI_BINARY" >&2
  exit 1
fi

ARCHIVE_BASENAME="semauri-${VERSION}-${SEMAURI_PLATFORM}"
WORK_DIR=$(mktemp -d)
STAGE_DIR="$WORK_DIR/$ARCHIVE_BASENAME"
trap 'rm -rf "$WORK_DIR"' EXIT

mkdir -p "$STAGE_DIR/bin" "$OUT_DIR"
cp "$SEMAURI_BINARY" "$STAGE_DIR/bin/semauri"
chmod +x "$STAGE_DIR/bin/semauri"
cp "$ROOT_DIR/LICENSE" "$ROOT_DIR/NOTICE" "$ROOT_DIR/README.md" "$STAGE_DIR/"

cat > "$STAGE_DIR/manifest.json" <<JSON
{
  "name": "semauri",
  "version": "$VERSION",
  "platform": "$SEMAURI_PLATFORM",
  "compiler": "rust-native"
}
JSON

"$STAGE_DIR/bin/semauri" version
"$STAGE_DIR/bin/semauri" check "$ROOT_DIR/examples/hello.sema"

ARCHIVE_PATH="$OUT_DIR/$ARCHIVE_BASENAME.tar.gz"
tar -czf "$ARCHIVE_PATH" -C "$WORK_DIR" "$ARCHIVE_BASENAME"
echo "$ARCHIVE_PATH"
