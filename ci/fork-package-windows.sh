#!/usr/bin/env bash
# Packages the Windows release binaries the way the justfile's
# bundle-windows recipe does.
#
#   ci/fork-package-windows.sh <binary dir> <package name>
#
# Produces <package name>.zip, whose single top-level entry is the
# <package name> directory, and <package name>.zip.sha256.
set -euo pipefail

BIN_DIR="$1"
NAME="$2"

STAGE="$(mktemp -d)"
DEST="$STAGE/$NAME"

mkdir -p "$DEST/mesa"
for bin in wezterm-gui wezterm wezterm-mux-server strip-ansi-escapes; do
  cp "$BIN_DIR/$bin.exe" "$DEST/"
done
cp assets/windows/conhost/conpty.dll assets/windows/conhost/OpenConsole.exe "$DEST/"
cp assets/windows/angle/libEGL.dll assets/windows/angle/libGLESv2.dll "$DEST/"
cp assets/windows/mesa/opengl32.dll "$DEST/mesa/"

OUT="$PWD/$NAME.zip"
(cd "$STAGE" && 7z a -tzip "$OUT" "$NAME" >/dev/null)
sha256sum "$NAME.zip" > "$NAME.zip.sha256"
rm -rf "$STAGE"
echo "Built: $NAME.zip"
