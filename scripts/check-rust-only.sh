#!/usr/bin/env bash
set -euo pipefail

source_ext=".r""b"
package_ext=".gem""spec"
package_file="Gem""file"
runtime_name="ru""by"

legacy_files="$(find . -path './.git' -prune -o -type f \( -name "*${source_ext}" -o -name "*${package_ext}" -o -name "${package_file}" -o -name "${package_file}.lock" \) -print)"
if [[ -n "$legacy_files" ]]; then
  echo "Legacy implementation files are not allowed:" >&2
  printf '%s\n' "$legacy_files" >&2
  exit 1
fi

if grep -RniI --exclude-dir=.git --exclude=check-rust-only.sh "$runtime_name" .; then
  echo "Legacy runtime references are not allowed in the source tree." >&2
  exit 1
fi
