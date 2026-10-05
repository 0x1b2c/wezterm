#!/usr/bin/env bash
# Prints the label this fork's release workflow puts into package names.
#
# For a release tag (`1b2c-*`), the label is the tag itself, and the tag is
# also written to `.tag`, which the build reads as the fork version, so the
# binaries report exactly the version they were released as. Any other
# build is labelled by commit and leaves the version to the build itself.
set -euo pipefail

if [ "${GITHUB_REF_TYPE:-}" = "tag" ]; then
  printf '%s\n' "$GITHUB_REF_NAME" > .tag
  printf '%s\n' "$GITHUB_REF_NAME"
else
  printf '1b2c-ci-%s\n' "$(git rev-parse --short=8 HEAD)"
fi
