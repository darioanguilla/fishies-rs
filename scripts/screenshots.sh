#!/usr/bin/env bash
# Regenerate the PNGs in docs/screenshots.
#
# Renders a few scenes to HTML (an ignored test), then screenshots them with
# a headless Chromium-based browser. Set BROWSER to pick one; by default it
# uses Microsoft Edge on Windows, or chromium/google-chrome elsewhere.
set -euo pipefail
cd "$(dirname "$0")/.."

cargo test --quiet screenshots -- --ignored

if [[ -z "${BROWSER:-}" ]]; then
  for b in "/c/Program Files (x86)/Microsoft/Edge/Application/msedge.exe" \
           "/c/Program Files/Microsoft/Edge/Application/msedge.exe" \
           chromium chromium-browser google-chrome; do
    if command -v "$b" > /dev/null 2>&1 || [[ -x "$b" ]]; then BROWSER=$b; break; fi
  done
fi
[[ -n "${BROWSER:-}" ]] || { echo "no Chromium-based browser found, set BROWSER" >&2; exit 1; }

# browsers on Windows need Windows paths
winpath() { if command -v cygpath > /dev/null; then cygpath -w "$1"; else echo "$1"; fi; }

size=$(cat target/screenshots/size.txt)
mkdir -p docs/screenshots
for page in target/screenshots/*.html; do
  name=$(basename "$page" .html)
  png="$PWD/docs/screenshots/$name.png"
  "$BROWSER" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
    --window-size="$size" --screenshot="$(winpath "$png")" \
    "file:///$(winpath "$PWD/$page")" > /dev/null 2>&1
  echo "docs/screenshots/$name.png"
done
