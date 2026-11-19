#!/usr/bin/env bash
# Set the package version in Cargo.toml and Cargo.lock.
# Usage: set-version.sh <version>
set -euo pipefail
version=${1:?usage: set-version.sh <version>}
name=$(sed -n 's/^name *= *"\(.*\)"/\1/p' Cargo.toml | head -n1)

# the first `version =` line is the [package] one
sed -i "0,/^version *= *\".*\"/s//version = \"$version\"/" Cargo.toml
# the package's own entry in the lock file
NAME="$name" VERSION="$version" perl -0pi -e \
  's/(\[\[package\]\]\nname = "\Q$ENV{NAME}\E"\nversion = )"[^"]*"/$1"$ENV{VERSION}"/' Cargo.lock

grep -q "^version = \"$version\"" Cargo.toml
grep -A1 "^name = \"$name\"" Cargo.lock | grep -q "version = \"$version\""
