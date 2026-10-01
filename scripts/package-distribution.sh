#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
RUBY_PREFIX=${RUBY_PREFIX:?RUBY_PREFIX must point to the relocatable Ruby installation}
SEMAURI_PLATFORM=${SEMAURI_PLATFORM:?SEMAURI_PLATFORM must name the target platform}
OUT_DIR=${OUT_DIR:-"$ROOT_DIR/dist"}

VERSION=$(sed -n 's/.*VERSION = "\([^"]*\)".*/\1/p' "$ROOT_DIR/lib/semauri/version.rb")
if [[ -z "$VERSION" ]]; then
  echo "Unable to determine Semauri version" >&2
  exit 1
fi

if [[ ! -x "$RUBY_PREFIX/bin/ruby" ]]; then
  echo "Ruby runtime is missing: $RUBY_PREFIX/bin/ruby" >&2
  exit 1
fi

RUBY_VERSION=$($RUBY_PREFIX/bin/ruby -e 'print RUBY_VERSION')
ARCHIVE_BASENAME="semauri-${VERSION}-${SEMAURI_PLATFORM}"
WORK_DIR=$(mktemp -d)
STAGE_DIR="$WORK_DIR/$ARCHIVE_BASENAME"
trap 'rm -rf "$WORK_DIR"' EXIT

mkdir -p "$STAGE_DIR/bin" "$STAGE_DIR/app" "$STAGE_DIR/runtime" "$OUT_DIR"

cp "$ROOT_DIR/distribution/bin/semauri" "$STAGE_DIR/bin/semauri"
chmod +x "$STAGE_DIR/bin/semauri"

cp -R "$ROOT_DIR/bin" "$ROOT_DIR/lib" "$STAGE_DIR/app/"
cp "$ROOT_DIR/LICENSE" "$ROOT_DIR/NOTICE" "$ROOT_DIR/README.md" "$STAGE_DIR/"
cp -R "$RUBY_PREFIX" "$STAGE_DIR/runtime/ruby"

cat > "$STAGE_DIR/manifest.json" <<JSON
{
  "name": "semauri",
  "version": "$VERSION",
  "platform": "$SEMAURI_PLATFORM",
  "compiler": "ruby-reference",
  "private_runtime": {
    "name": "ruby",
    "version": "$RUBY_VERSION"
  }
}
JSON

# Prove the copied runtime is actually relocatable before publishing it.
"$STAGE_DIR/bin/semauri" version
"$STAGE_DIR/bin/semauri" check "$ROOT_DIR/examples/hello.sema"

ARCHIVE_PATH="$OUT_DIR/$ARCHIVE_BASENAME.tar.gz"
tar -czf "$ARCHIVE_PATH" -C "$WORK_DIR" "$ARCHIVE_BASENAME"

echo "$ARCHIVE_PATH"
