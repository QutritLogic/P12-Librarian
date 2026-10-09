#!/bin/sh
# Builds the user guide PDF that the app opens on F1. Needs the typst CLI
# (cargo install --locked typst-cli).
set -e
cd "$(dirname "$0")"
typst compile --root .. user-guide.typ ../app/src-tauri/resources/user-guide.pdf
echo "Wrote app/src-tauri/resources/user-guide.pdf"
