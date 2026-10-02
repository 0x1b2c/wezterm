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

# The Docker daemon comes from the caller's environment: set DOCKER_CONTEXT to build
# on another host, and CROSS_REMOTE=1 when that daemon cannot see this checkout's files.
# Cross-build Windows binaries for x86_64-pc-windows-gnu (MinGW).
build-windows:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cross >/dev/null 2>&1; then
        echo "Error: 'cross' not found. Install with: cargo install cross --git https://github.com/cross-rs/cross"
        exit 1
    fi
    if ! docker info >/dev/null 2>&1; then
        echo "Error: Docker is not reachable (DOCKER_CONTEXT=${DOCKER_CONTEXT:-default}). Start Docker or point DOCKER_CONTEXT at a running daemon."
        exit 1
    fi
    # cross runs an x86_64 Linux toolchain inside its amd64-only image.
    if ! rustup toolchain list | grep -q '^stable-x86_64-unknown-linux-gnu'; then
        echo "Error: missing toolchain. Install with: rustup toolchain add stable-x86_64-unknown-linux-gnu --profile minimal --force-non-host"
        exit 1
    fi
    export DOCKER_DEFAULT_PLATFORM=linux/amd64
    # A host-side rustc wrapper such as sccache does not exist in the container,
    # and stripping at link time avoids needing a Windows strip tool on the host.
    export RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_PROFILE_RELEASE_STRIP=symbols
    export CROSS_CONTAINER_OPTS="${CROSS_CONTAINER_OPTS:-} -e RUSTC_WRAPPER= -e CARGO_BUILD_RUSTC_WRAPPER= -e CARGO_PROFILE_RELEASE_STRIP=symbols"
    cross build --release --target x86_64-pc-windows-gnu -p wezterm-gui -p wezterm -p wezterm-mux-server -p strip-ansi-escapes

# Assemble target/WezTerm-windows/ and zip it as target/WezTerm-windows.zip.
bundle-windows: build-windows
    #!/usr/bin/env bash
    set -euo pipefail
    SRC="target/x86_64-pc-windows-gnu/release"
    DEST="target/WezTerm-windows"
    ZIP="target/WezTerm-windows.zip"
    rm -rf "$DEST" "$ZIP"
    mkdir -p "$DEST/mesa"
    for bin in wezterm-gui wezterm wezterm-mux-server strip-ansi-escapes; do
        cp "$SRC/$bin.exe" "$DEST/"
    done
    cp assets/windows/conhost/conpty.dll assets/windows/conhost/OpenConsole.exe "$DEST/"
    cp assets/windows/angle/libEGL.dll assets/windows/angle/libGLESv2.dll "$DEST/"
    cp assets/windows/mesa/opengl32.dll "$DEST/mesa/"
    # macOS attaches extended attributes that would become AppleDouble entries.
    xattr -cr "$DEST"
    ditto -c -k --norsrc --noextattr --noqtn --noacl --keepParent "$DEST" "$ZIP"
    echo "Built: $ZIP"

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

# Machine-specific recipes (deploying to particular hosts and install paths)
# live in an uncommitted justfile.local next to this file, imported when present.
import? 'justfile.local'
