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

## Problems this fork solves

The patches are grouped by theme, each rooted in a real annoyance from daily
use, most of it centered on WezTerm's multiplexer over SSH. Each entry names
the problem first; the branch implementing the fix is linked at the end. One
entry is one branch; the commits within a branch are cohesive and are not
broken out here. Build and developer-experience patches, which have no
user-facing symptom, are listed by branch name in the final section.

### The GUI freezes or lags badly under mux load

- **Reconnecting to a mux server with many tabs open freezes the GUI.** A slow
  write on the mux channel could block the read path and deadlock the whole
  session. Fixed by splitting the mux client and server dispatch into
  independent reader and writer tasks, with notifications handled in a separate
  task.
  *Branch: [mux-deadlock-fix](https://github.com/0x1b2c/wezterm/compare/main...mux-deadlock-fix), submitted upstream as [PR #7771](https://github.com/wezterm/wezterm/pull/7771).*

- **Resizing a window with many tabs open freezes the GUI.** Every `TabResized`
  event triggered a full resync of all panes, an O(n²) cost that locked up the
  interface. Fixed by skipping the redundant resync.
  *Branch: [skip-resync-on-tab-resized](https://github.com/0x1b2c/wezterm/compare/ssh-proxy-error-msg...skip-resync-on-tab-resized).*

- **Programs that update the terminal title rapidly make the tab bar stutter.**
  Frequent OSC title updates re-ran the `format-tab-title` machinery on every
  update, thrashing the render loop. Fixed by caching and short-circuiting that
  work.
  *Branch: [tab-title-perf](https://github.com/0x1b2c/wezterm/compare/objc-cargo-clippy-lint...tab-title-perf).*

- **Keystrokes feel sluggish because rendering waits for the next scheduled
  frame.** Fixed by triggering an immediate render poll right after a
  keystroke, reducing perceived input latency.
  *Branch: [keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/mux-deadlock-fix...keystroke-render-poll).*

### Declare window layouts once, spawn them on demand

- **Setting up the same windows and tabs by hand every time is tedious.** This
  fork adds window presets: declare window layouts in the config, then spawn
  and kill them on demand via launcher flags, so a known-good working set is
  always one command away. Includes a `SpawnCommand` `text` field so a preset
  tab can auto-run a command in the default shell.
  *Branch: [window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets).*

- **There is no keybinding action to close a whole mux window at once**, which
  resetting a layout needs. Added a Lua action that atomically kills a mux
  window.
  *Branch: [lua-kill-window](https://github.com/0x1b2c/wezterm/compare/tab-title-perf...lua-kill-window).*

- **Closing one window can detach a mux domain that other windows are still
  using.** Fixed by keeping the domain attached while any window still uses it.
  *Branch: [session-experience](https://github.com/0x1b2c/wezterm/compare/lua-kill-window...session-experience).*

### Typing over a slow connection is painful

- **On a high-latency link, every keystroke costs a full round trip before it
  appears on screen.** This gets especially bad when the mux server runs on a
  Mac that has dozed off: macOS power management throttles the sleeping
  machine, and each round trip stretches from milliseconds to seconds. This
  fork adds a per-pane Local input mode that buffers keystrokes client-side
  and ships the line to the PTY only on submit. It is a convenience tool aimed
  at a minimally usable environment when the network is bad, not a perfect
  line editor. It is the newest patch in the stack, with known edge cases
  around paste and full key capture still open.
  *Branch: [local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input).*

### Failures should explain themselves

- **When the remote SSH proxy command fails, all you get is an opaque
  connection error.** Fixed by surfacing a clear error message that says what
  actually failed.
  *Branch: [ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg).*

### Build and developer experience

These carry no user-facing symptom; listed for completeness.

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

- `just build-arm64`: native arm64 release binaries (add `debug` for a debug build)
- `just bundle-arm64`: assemble `target/WezTerm.app` from the arm64 binaries
- `just build-x86`: `x86_64-apple-darwin` release binaries
- `just build-linux`: static `x86_64-unknown-linux-musl` binaries (requires `cross` and Docker)

The `deploy-*` recipes are specific to the author's own machines and install
paths, and are not meant for general use.

If you prefer to build without `just`, follow upstream's source build
instructions at <https://wezterm.org/install/source.html>, building from this
fork's `master` branch instead of upstream `main`.

## License and attribution

The license is unchanged from upstream WezTerm. All upstream copyright and
attribution notices are preserved. See [LICENSE.md](LICENSE.md).
