#!/usr/bin/env bash
# Packages one architecture's release binaries as this fork's macOS app.
#
#   ci/fork-package-macos.sh <binary dir> <package name>
#
# Produces <package name>.zip, whose single top-level entry is WezTerm.app,
# and <package name>.zip.sha256. The bundle is assembled the way the
# justfile's bundle recipes do it and signed by its _sign-mac recipe.
set -euo pipefail

BIN_DIR="$1"
NAME="$2"

STAGE="$(mktemp -d)"
APP="$STAGE/WezTerm.app"

cp -r assets/macos/WezTerm.app "$APP"
rm -f "$APP"/*.dylib
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
for bin in wezterm-gui wezterm wezterm-mux-server strip-ansi-escapes; do
  cp "$BIN_DIR/$bin" "$APP/Contents/MacOS/$bin"
done
cp -r assets/shell-integration/* "$APP/Contents/Resources/"
cp -r assets/shell-completion "$APP/Contents/Resources/"
tic -xe wezterm -o "$APP/Contents/Resources/terminfo" termwiz/data/wezterm.terminfo
just _sign-mac "$APP"

ditto -c -k --keepParent --norsrc --noextattr --noacl "$APP" "$NAME.zip"
shasum -a 256 "$NAME.zip" > "$NAME.zip.sha256"
rm -rf "$STAGE"
echo "Built: $NAME.zip"
