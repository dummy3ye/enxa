#!/usr/bin/env bash
# Reject Rust source filenames that contain '_' or '-'.
find . -path ./target -prune -o -name '*[_-]*.rs' -print | grep . && exit 1 || exit 0
