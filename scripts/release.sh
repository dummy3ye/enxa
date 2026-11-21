#!/usr/bin/env bash
# Local pre-tag dry run for a release. Prints everything needed to review before
# committing, tagging and pushing. Never creates or pushes a tag.
set -euo pipefail
version="${1:?usage: scripts/release.sh <version>}"

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

branch="$(git branch --show-current)"
if [[ "$branch" != "main" ]]; then
  echo "error: releases must be prepared on main, found '$branch'" >&2
  exit 1
fi

manifest_version="$(python3 -c 'import re; print(re.search(r"^version = \"([^\"]+)\"", open("Cargo.toml").read(), re.M).group(1))')"
if [[ "$manifest_version" != "$version" ]]; then
  echo "error: Cargo.toml version '$manifest_version' != requested '$version'" >&2
  exit 1
fi

if ! grep -q "^## \[$version\]" CHANGELOG.md; then
  echo "error: CHANGELOG.md has no dated section for $version" >&2
  exit 1
fi

echo "== git status =="
git status --short

./scripts/check.sh
cargo test -p verify --release --locked --test gate -- --ignored --nocapture

echo "== release notes =="
bash scripts/releasenotes.sh "$version"

for level in v2 v3; do
  bash scripts/package.sh "$version" "$level"
done

echo "== checksums =="
(
  cd dist
  sha256sum enxa-*.tar.gz > SHA256SUMS
  cat SHA256SUMS
  sha256sum -c SHA256SUMS
)

for level in v2 v3; do
  archive="$root/dist/enxa-${version}-x86_64-linux-${level}.tar.gz"
  unpacked="$(mktemp -d /tmp/opencode/enxa-smoke.XXXXXX)"
  tar xzf "$archive" -C "$unpacked"
  echo "== smoke test $level =="
  "$unpacked/enso" --version
  "$unpacked/exa" --version
  "$unpacked/benchkit" --version
  grep -E '^(version|flavor|rustflags):' "$unpacked/RELEASE-INFO.txt"
  tar tzf "$archive"
done

echo "== v3 benchmarks =="
RUSTFLAGS="-C target-cpu=x86-64-v3 --remap-path-prefix=$root=." \
  cargo run --locked --release --target x86_64-unknown-linux-gnu \
  -p benchkit -- --solver enso --n 100000 --seed 1
RUSTFLAGS="-C target-cpu=x86-64-v3 --remap-path-prefix=$root=." \
  cargo run --locked --release --target x86_64-unknown-linux-gnu \
  -p benchkit -- --solver enso --n 100000 --seed 1 --facelets --batch
RUSTFLAGS="-C target-cpu=x86-64-v3 --remap-path-prefix=$root=." \
  cargo run --locked --release --target x86_64-unknown-linux-gnu \
  -p benchkit -- --solver exa --n 50 --depth 8 --seed 1

echo "DRY_RUN_OK (no tag created, nothing pushed)"
