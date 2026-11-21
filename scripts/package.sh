#!/usr/bin/env bash
# Build one portable x86-64 Linux flavor and write a reproducible tarball.
set -euo pipefail

version="${1:?usage: scripts/package.sh <version> <v2|v3>}"
level="${2:?usage: scripts/package.sh <version> <v2|v3>}"

case "$level" in
  v2)
    cpu="x86-64-v2"
    features="SSE4.2, SSSE3, POPCNT, CMPXCHG16B, LAHF/SAHF (x86-64-v2 baseline)"
    ;;
  v3)
    cpu="x86-64-v3"
    features="AVX, AVX2, BMI1, BMI2, FMA, LZCNT, MOVBE, XSAVE (x86-64-v3 baseline)"
    ;;
  *)
    echo "error: level must be v2 or v3" >&2
    exit 2
    ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
build_date="$(date -u -d "@$SOURCE_DATE_EPOCH" +%Y-%m-%dT%H:%M:%SZ)"
commit="$(git rev-parse HEAD)"
rustc_version="$(rustc -V)"
rustflags="-C target-cpu=$cpu --remap-path-prefix=$root=."
export RUSTFLAGS="$rustflags"

target_dir="$root/target/portable-$level"
cargo_target_dir="$target_dir"
CARGO_TARGET_DIR="$cargo_target_dir" cargo build \
  --locked \
  --release \
  --target x86_64-unknown-linux-gnu \
  --bins \
  -p enso \
  -p exa \
  -p benchkit

bindir="$target_dir/x86_64-unknown-linux-gnu/release"
stage="$(mktemp -d /tmp/opencode/enxa-package.XXXXXX)"
cp "$bindir/enso" "$bindir/exa" "$bindir/benchkit" "$stage/"
cp README.md LICENSE-MIT LICENSE-APACHE "$stage/"

cat > "$stage/RELEASE-INFO.txt" <<EOF
version: $version
flavor: x86-64-linux-$level
git_commit: $commit
rustc: $rustc_version
build_date: $build_date
source_date_epoch: $SOURCE_DATE_EPOCH
rustflags: $rustflags
target: x86_64-unknown-linux-gnu
required_cpu_features: $features
EOF

mkdir -p "$root/dist"
archive="$root/dist/enxa-${version}-x86_64-linux-${level}.tar.gz"
tar --sort=name --owner=0 --group=0 --numeric-owner \
  --mtime="@$SOURCE_DATE_EPOCH" --format=gnu -cf - \
  -C "$stage" \
  README.md LICENSE-APACHE LICENSE-MIT RELEASE-INFO.txt benchkit enso exa \
  | gzip -n > "$archive"

echo "$archive"
