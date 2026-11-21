#!/usr/bin/env bash
# Extract one version's section from CHANGELOG.md for GitHub release notes.
set -euo pipefail
version="${1:?usage: scripts/releasenotes.sh <version> [output-file]}"
output="${2:-}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

notes="$(awk -v v="$version" '
  index($0, "## [" v "]") == 1 {
    found = 1
    sub(/^## \[[^]]+\] - /, "## ", $0)
    print
    next
  }
  found && /^## / { exit }
  found { print }
' CHANGELOG.md)"

if [[ -z "$notes" ]]; then
  echo "error: no CHANGELOG.md section found for $version" >&2
  exit 1
fi

if [[ -n "$output" ]]; then
  printf '%s\n' "$notes" > "$output"
else
  printf '%s\n' "$notes"
fi
