#!/usr/bin/env bash
set -euo pipefail

legacy_files="$(find . -path './.git' -prune -o -type f \( -name '*.rb' -o -name '*.gemspec' -o -name 'Gemfile' -o -name 'Gemfile.lock' \) -print)"
if [[ -n "$legacy_files" ]]; then
  echo "Legacy implementation files are not allowed:" >&2
  printf '%s\n' "$legacy_files" >&2
  exit 1
fi

if grep -RniI --exclude-dir=.git --exclude=check-rust-only.sh 'ruby' .; then
  echo "Legacy runtime references are not allowed in the source tree." >&2
  exit 1
fi
