#!/usr/bin/env sh
set -eu

REPO="edujbarrios/semauri"
INSTALL_ROOT=${SEMAURI_INSTALL_ROOT:-"$HOME/.semauri"}
BIN_DIR=${SEMAURI_BIN_DIR:-"$HOME/.local/bin"}
REQUESTED_VERSION=${SEMAURI_VERSION:-""}

fail() {
  echo "semauri-install: $*" >&2
  exit 1
}

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"

case "$(uname -s)" in
  Linux) OS=linux ;;
  Darwin) OS=macos ;;
  *) fail "unsupported operating system: $(uname -s). Supported: Linux and macOS." ;;
esac

case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  arm64|aarch64) ARCH=arm64 ;;
  *) fail "unsupported architecture: $(uname -m). Supported: x86_64 and arm64." ;;
esac

if [ -n "$REQUESTED_VERSION" ]; then
  VERSION=${REQUESTED_VERSION#v}
else
  RELEASE_JSON=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest") || fail "unable to resolve the latest Semauri release"
  TAG=$(printf '%s\n' "$RELEASE_JSON" | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
  [ -n "$TAG" ] || fail "latest GitHub release did not contain a tag"
  VERSION=${TAG#v}
fi

PLATFORM="$OS-$ARCH"
ASSET="semauri-$VERSION-$PLATFORM.tar.gz"
BASE_URL="https://github.com/$REPO/releases/download/v$VERSION"
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

ARCHIVE="$TMP_DIR/$ASSET"
CHECKSUMS="$TMP_DIR/SHA256SUMS"

echo "Installing Semauri $VERSION for $PLATFORM..."
curl -fL --retry 3 -o "$ARCHIVE" "$BASE_URL/$ASSET" || fail "release asset not found: $ASSET"
curl -fL --retry 3 -o "$CHECKSUMS" "$BASE_URL/SHA256SUMS" || fail "SHA256SUMS not found for v$VERSION"

EXPECTED=$(grep "  $ASSET$" "$CHECKSUMS" | awk '{print $1}')
[ -n "$EXPECTED" ] || fail "checksum for $ASSET is missing"

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL=$(sha256sum "$ARCHIVE" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
else
  fail "sha256sum or shasum is required to verify the download"
fi

[ "$EXPECTED" = "$ACTUAL" ] || fail "checksum verification failed for $ASSET"

echo "Checksum verified."

EXTRACT_DIR="$TMP_DIR/extracted"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$ARCHIVE" -C "$EXTRACT_DIR"
PACKAGE_DIR="$EXTRACT_DIR/semauri-$VERSION-$PLATFORM"
[ -x "$PACKAGE_DIR/bin/semauri" ] || fail "release archive has an invalid layout"

mkdir -p "$INSTALL_ROOT/versions" "$BIN_DIR"
VERSION_DIR="$INSTALL_ROOT/versions/$VERSION"
rm -rf "$VERSION_DIR"
mv "$PACKAGE_DIR" "$VERSION_DIR"
ln -sfn "$VERSION_DIR" "$INSTALL_ROOT/current"
ln -sfn "$INSTALL_ROOT/current/bin/semauri" "$BIN_DIR/semauri"

"$BIN_DIR/semauri" version

echo "Installed at $VERSION_DIR"
echo "Command: $BIN_DIR/semauri"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    echo "Note: $BIN_DIR is not currently in PATH. Add it to your shell configuration."
    ;;
esac
