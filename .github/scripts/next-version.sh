#!/usr/bin/env bash
# Print the version to release for the current commit, or nothing when there
# is nothing new to release.
#
# The version comes from the Conventional Commits since the last v* tag:
#   feat:                       -> minor (0.1.1 -> 0.2.0)
#   fix: / perf:                -> patch (0.2.0 -> 0.2.1)
#   type!: or "BREAKING CHANGE" -> major (minor while the version is 0.x)
# A version bumped by hand in Cargo.toml wins over the computed one.
set -euo pipefail

current=$(sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -n1)
last=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2> /dev/null || true)
last=${last#v}

if [[ -z "$last" ]]; then
  echo "$current" # first release
  exit 0
fi
newest=$(printf '%s\n%s\n' "$last" "$current" | sort -V | tail -n1)
if [[ "$current" != "$last" && "$newest" == "$current" ]]; then
  echo "$current" # bumped by hand
  exit 0
fi

subjects=$(git log --no-merges --format='%s' "v$last..HEAD")
bodies=$(git log --no-merges --format='%b' "v$last..HEAD")
IFS=. read -r major minor patch <<< "${last%%-*}"

if grep -qE '^[a-z]+(\([^)]*\))?!:' <<< "$subjects" || grep -q 'BREAKING CHANGE' <<< "$bodies"; then
  if ((major == 0)); then
    minor=$((minor + 1)) patch=0
  else
    major=$((major + 1)) minor=0 patch=0
  fi
elif grep -qE '^feat(\([^)]*\))?:' <<< "$subjects"; then
  minor=$((minor + 1)) patch=0
elif grep -qE '^(fix|perf)(\([^)]*\))?:' <<< "$subjects"; then
  patch=$((patch + 1))
else
  exit 0 # only docs, ci, tests, ...
fi
echo "$major.$minor.$patch"
