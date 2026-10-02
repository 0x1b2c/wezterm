# Default recipe lists available recipes when running `just` without arguments.
default:
    @just --list

# Build native arm64 binaries. `profile` is `release` (default) or `debug`.
build-arm64 profile="release":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{ profile }}" = "release" ]; then
        cargo build --release -p wezterm-gui -p wezterm -p wezterm-mux-server -p strip-ansi-escapes
    elif [ "{{ profile }}" = "debug" ]; then
        cargo build -p wezterm-gui -p wezterm -p wezterm-mux-server -p strip-ansi-escapes
    else
        echo "Error: profile must be 'release' or 'debug', got '{{ profile }}'"
        exit 1
    fi

# Build x86_64-apple-darwin release binaries (4 binaries, including strip-ansi-escapes for the bundle).
build-x86:
    cargo build --release --target x86_64-apple-darwin -p wezterm-gui -p wezterm -p wezterm-mux-server -p strip-ansi-escapes

# Cross-build static Linux binaries (wezterm + wezterm-mux-server) for x86_64-unknown-linux-musl.
build-linux:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cross >/dev/null 2>&1; then
        echo "Error: 'cross' not found. Install with: cargo install cross --git https://github.com/cross-rs/cross"
        exit 1
    fi
    if ! docker info >/dev/null 2>&1; then
        echo "Error: Docker is not running. Start Docker Desktop first."
        exit 1
    fi
    cross build --release --target x86_64-unknown-linux-musl -p wezterm -p wezterm-mux-server
    strip target/x86_64-unknown-linux-musl/release/wezterm 2>/dev/null || true
    strip target/x86_64-unknown-linux-musl/release/wezterm-mux-server 2>/dev/null || true

# Build target/WezTerm.app from native arm64 binaries. `profile` is forwarded to build-arm64.
bundle-arm64 profile="release": (build-arm64 profile)
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "{{ profile }}" = "release" ]; then
        TARGET_DIR="target/release"
    else
        TARGET_DIR="target/debug"
    fi
    DEST="target/WezTerm.app"
    rm -rf "$DEST"
    cp -r assets/macos/WezTerm.app "$DEST"
    rm -f "$DEST"/*.dylib
    mkdir -p "$DEST/Contents/MacOS" "$DEST/Contents/Resources"
    for bin in wezterm-gui wezterm wezterm-mux-server strip-ansi-escapes; do
        cp "$TARGET_DIR/$bin" "$DEST/Contents/MacOS/$bin"
    done
    cp -r assets/shell-integration/* "$DEST/Contents/Resources/"
    cp -r assets/shell-completion "$DEST/Contents/Resources/"
    tic -xe wezterm -o "$DEST/Contents/Resources/terminfo" termwiz/data/wezterm.terminfo
    just _sign-mac "$DEST"
    echo "Built: $DEST"

# Build target/WezTerm-x86.app from x86_64-apple-darwin release binaries.
bundle-x86: build-x86
    #!/usr/bin/env bash
    set -euo pipefail
    TARGET_DIR="target/x86_64-apple-darwin/release"
    DEST="target/WezTerm-x86.app"
    rm -rf "$DEST"
    cp -r assets/macos/WezTerm.app "$DEST"
    rm -f "$DEST"/*.dylib
    mkdir -p "$DEST/Contents/MacOS" "$DEST/Contents/Resources"
    for bin in wezterm-gui wezterm wezterm-mux-server strip-ansi-escapes; do
        cp "$TARGET_DIR/$bin" "$DEST/Contents/MacOS/$bin"
    done
    cp -r assets/shell-integration/* "$DEST/Contents/Resources/"
    cp -r assets/shell-completion "$DEST/Contents/Resources/"
    tic -xe wezterm -o "$DEST/Contents/Resources/terminfo" termwiz/data/wezterm.terminfo
    just _sign-mac "$DEST"
    echo "Built: $DEST"

# Internal: ad-hoc sign a WezTerm.app with identifier-only designated
# requirements. TCC binds privacy grants to the designated requirement; a
# plain ad-hoc signature pins it to the binary's cdhash, which changes on every
# rebuild and silently invalidates the grants. Naming only the identifier lets
# every future build satisfy it. The nested executables are signed first so
# the bundle can be sealed without --deep. See "Bundle identifier and signing"
# in FORK.md.
_sign-mac app:
    #!/usr/bin/env bash
    set -euo pipefail
    APP="{{ app }}"
    for bin in wezterm wezterm-mux-server strip-ansi-escapes; do
        exe="$APP/Contents/MacOS/$bin"
        if [ -f "$exe" ]; then
            codesign --force --sign - --identifier "org.1b2c.wezterm.$bin" \
                -r="designated => identifier \"org.1b2c.wezterm.$bin\"" "$exe"
        fi
    done
    codesign --force --sign - -r='designated => identifier "org.1b2c.wezterm"' "$APP"

# Deploy native arm64 release to /Applications/WezTerm.app. Flags: `+mux`, `+full-bundle`.
deploy-release *flags="":
    #!/usr/bin/env bash
    set -euo pipefail
    just _deploy-mac release {{ flags }}

# Deploy a native arm64 debug build to /Applications/WezTerm.app. Flags identical to deploy-release.
deploy-debug *flags="":
    #!/usr/bin/env bash
    set -euo pipefail
    just _deploy-mac debug {{ flags }}

# Internal: shared mac deploy implementation parameterised by profile.
_deploy-mac profile *flags="":
    #!/usr/bin/env bash
    set -euo pipefail
    has_mux=false
    has_full=false
    for f in {{ flags }}; do
        case "$f" in
            +mux) has_mux=true ;;
            +full-bundle) has_full=true ;;
            *) echo "unknown flag: $f"; exit 1 ;;
        esac
    done
    APP="/Applications/WezTerm.app/Contents/MacOS"
    if [ "{{ profile }}" = "release" ]; then
        TARGET_DIR="target/release"
        CARGO_PROFILE_FLAG="--release"
    else
        TARGET_DIR="target/debug"
        CARGO_PROFILE_FLAG=""
    fi
    if [ "$has_full" = true ]; then
        just bundle-arm64 {{ profile }}
        rsync -a --delete target/WezTerm.app/ /Applications/WezTerm.app/
        if [ "$has_mux" = true ]; then
            killall wezterm-mux-server 2>/dev/null || true
        fi
    else
        if [ "$has_mux" = true ]; then
            cargo build $CARGO_PROFILE_FLAG -p wezterm-gui -p wezterm -p wezterm-mux-server
        else
            cargo build $CARGO_PROFILE_FLAG -p wezterm-gui -p wezterm
        fi
        cp "$TARGET_DIR/wezterm-gui" "$APP/"
        cp "$TARGET_DIR/wezterm" "$APP/"
        if [ "$has_mux" = true ]; then
            killall wezterm-mux-server 2>/dev/null || true
            cp "$TARGET_DIR/wezterm-mux-server" "$APP/"
        fi
        just _sign-mac /Applications/WezTerm.app
        # Copy the CLI tools out of the signed bundle so they carry the same
        # identifier-based signature as their in-bundle counterparts.
        cp "$APP/wezterm" /opt/homebrew/bin/wezterm
        if [ "$has_mux" = true ]; then
            cp "$APP/wezterm-mux-server" /opt/homebrew/bin/wezterm-mux-server
        fi
    fi
    echo "Deployed (profile={{ profile }}, mux=$has_mux, full-bundle=$has_full)"

# Deploy the x86 bundle to host `a2` by replacing /Applications/WezTerm.app there.
deploy-a2: bundle-x86
    #!/usr/bin/env bash
    set -euo pipefail
    ARCHIVE="target/wezterm-x86_64.tar.zst"
    tar -C target -cf - WezTerm-x86.app/ | zstd -T0 -f -o "$ARCHIVE"
    scp "$ARCHIVE" a2:/tmp/
    ssh a2 'export PATH="/usr/local/bin:$PATH" && cd /tmp && tar --use-compress-program=unzstd -xf wezterm-x86_64.tar.zst && rsync -a --delete WezTerm-x86.app/ /Applications/WezTerm.app/ && rm -rf WezTerm-x86.app wezterm-x86_64.tar.zst'
    echo "Deployed to a2"

# Deploy the linux CLI + mux-server to host `archer` under /usr/local/bin.
deploy-archer: build-linux
    #!/usr/bin/env bash
    set -euo pipefail
    OUT="target/wezterm-linux-x86_64"
    ARCHIVE="target/wezterm-linux-x86_64.tar.zst"
    rm -rf "$OUT"
    mkdir -p "$OUT"
    cp target/x86_64-unknown-linux-musl/release/wezterm "$OUT/wezterm"
    cp target/x86_64-unknown-linux-musl/release/wezterm-mux-server "$OUT/wezterm-mux-server"
    cat >"$OUT/deploy.sh" <<'DEPLOY'
    #!/bin/bash
    set -euo pipefail
    DIR="$(dirname "$0")"
    INSTALL="/usr/local/bin"
    sudo install -m 755 "$DIR/wezterm" "$INSTALL/wezterm"
    sudo install -m 755 "$DIR/wezterm-mux-server" "$INSTALL/wezterm-mux-server"
    echo "Deployed to $INSTALL"
    DEPLOY
    chmod +x "$OUT/deploy.sh"
    tar -C target -cf - wezterm-linux-x86_64/ | zstd -T0 -f -o "$ARCHIVE"
    scp "$ARCHIVE" archer:/tmp/
    ssh -t archer 'cd /tmp && tar --use-compress-program=unzstd -xf wezterm-linux-x86_64.tar.zst && wezterm-linux-x86_64/deploy.sh && rm -rf wezterm-linux-x86_64 wezterm-linux-x86_64.tar.zst'
    echo "Deployed to archer"

# Dispatcher: `just deploy a2` -> `just deploy-a2`; `just deploy archer` -> `just deploy-archer`.
deploy host *args:
    just deploy-{{ host }} {{ args }}
