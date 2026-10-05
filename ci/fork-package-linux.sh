#!/usr/bin/env bash
# Packages the static Linux binaries: the CLI and the mux server, for use on
# remote hosts. There is no GUI in this package.
#
#   ci/fork-package-linux.sh <binary dir> <package name>
#
# Produces <package name>.tar.gz, whose single top-level entry is the
# <package name> directory, and <package name>.tar.gz.sha256.
set -euo pipefail

BIN_DIR="$1"
NAME="$2"

STAGE="$(mktemp -d)"
DEST="$STAGE/$NAME"

mkdir -p "$DEST"
for bin in wezterm wezterm-mux-server; do
  cp "$BIN_DIR/$bin" "$DEST/"
  strip "$DEST/$bin"
done

tar -czf "$NAME.tar.gz" -C "$STAGE" "$NAME"
sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256"
rm -rf "$STAGE"
echo "Built: $NAME.tar.gz"
