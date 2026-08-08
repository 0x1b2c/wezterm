# FORK.md

A personal fork of [WezTerm](https://github.com/wezterm/wezterm), maintained by
[0x1b2c](https://github.com/0x1b2c).

## Overview

A personal fork, not a downstream distribution: no separate release channels, no
rebranding, no intent to become a distinct product. It carries a stack of patches
on top of upstream WezTerm's `main`: daily-driver fixes and features for latency,
multiplexer correctness, and workflow ergonomics not yet available upstream.

Each feature lives on its own branch; `master` integrates the whole stack. One
patch has been submitted upstream (see below); the rest are fork-only. Everything
else is unchanged from upstream. For general WezTerm usage, see
<https://wezterm.org/>.

## What this fork adds

Listed roughly by significance: the most impactful fixes and features first,
build and developer-experience patches last. Each entry is one branch; the
commits within a branch are cohesive and are not broken out here.

### Fixes and features

- **[mux-deadlock-fix](https://github.com/0x1b2c/wezterm/compare/main...mux-deadlock-fix)** — *submitted upstream as [PR #7771](https://github.com/wezterm/wezterm/pull/7771)*
  Splits the mux client and server dispatch into independent reader and writer
  tasks, and handles mux notifications in a separate task. Eliminates a GUI
  freeze on reconnect when many tabs are open, where a slow write could block
  the read path and deadlock the session.

- **[skip-resync-on-tab-resized](https://github.com/0x1b2c/wezterm/compare/ssh-proxy-error-msg...skip-resync-on-tab-resized)**
  Skips the full pane resync on `TabResized`, removing an O(n²) cost that froze
  the GUI when many tabs were open.

- **[window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets)**
  Adds window preset configuration, PDUs, server-side dispatch, and launcher
  flags to spawn and kill predefined window layouts, including a `SpawnCommand`
  `text` field so a preset tab can auto-run a command in the default shell.

- **[local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input)** — per-pane Local input mode
  Buffers keystrokes client-side and ships them to the PTY only on submit,
  eliminating per-keystroke round-trip latency for long-form input over a mux
  connection. The newest patch in the stack and still evolving; some edge cases
  around paste and full key capture remain open.

- **[tab-title-perf](https://github.com/0x1b2c/wezterm/compare/objc-cargo-clippy-lint...tab-title-perf)**
  Caches and short-circuits `format-tab-title` work so frequent OSC title
  updates no longer thrash the tab bar render loop.

- **[keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/mux-deadlock-fix...keystroke-render-poll)**
  Triggers an immediate render poll right after a keystroke instead of waiting
  for the next scheduled frame, reducing perceived input latency.

- **[lua-kill-window](https://github.com/0x1b2c/wezterm/compare/tab-title-perf...lua-kill-window)**
  Adds a Lua keybinding action to atomically kill a mux window.

- **[session-experience](https://github.com/0x1b2c/wezterm/compare/lua-kill-window...session-experience)**
  Avoids detaching a mux domain that other windows are still using.

- **[ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg)**
  Surfaces a clear error message when the remote SSH proxy command fails,
  instead of an opaque connection failure.

### Build and developer experience

- **[macos-rerun-if-changed](https://github.com/0x1b2c/wezterm/compare/skip-resync-on-tab-resized...macos-rerun-if-changed)**
  Fixes the build script's `rerun-if-changed` path so macOS builds do not
  rebuild unnecessarily.

- **[vendor-openssl-musl](https://github.com/0x1b2c/wezterm/compare/macos-rerun-if-changed...vendor-openssl-musl)**
  Vendors OpenSSL for musl targets to enable static cross-compilation.

- **[objc-cargo-clippy-lint](https://github.com/0x1b2c/wezterm/compare/vendor-openssl-musl...objc-cargo-clippy-lint)**
  Whitelists the `objc` crate's `cargo-clippy` cfg in the workspace lint config
  to silence spurious warnings.

Beyond the branches above, the stack also carries a few commits tagged
`[local]`: a macOS `visibleFrame` window-geometry workaround, log tagging for
crash attribution, and build-noise suppression. These are experimental or
environment-specific attempts rather than user-facing features, and are not
documented individually.

## Building

Upstream ships CI configuration but no simple local build script. This fork adds
a [`justfile`](justfile) at the repository root for that purpose. Run `just` to
list the available recipes.

The commonly used build recipes:

- `just build-arm64` — native arm64 release binaries (add `debug` for a debug build)
- `just bundle-arm64` — assemble `target/WezTerm.app` from the arm64 binaries
- `just build-x86` — `x86_64-apple-darwin` release binaries
- `just build-linux` — static `x86_64-unknown-linux-musl` binaries (requires `cross` and Docker)

The `deploy-*` recipes are specific to the author's own machines and install
paths, and are not meant for general use.

If you prefer to build without `just`, follow upstream's source build
instructions at <https://wezterm.org/install/source.html>, building from this
fork's `master` branch instead of upstream `main`.

## License and attribution

The license is unchanged from upstream WezTerm. All upstream copyright and
attribution notices are preserved. See [LICENSE.md](LICENSE.md).
