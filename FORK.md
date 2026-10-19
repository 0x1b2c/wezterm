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
  *Branch: [keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/fork-base...keystroke-render-poll).*

### Declare window layouts once, spawn them on demand

- **Setting up the same windows and tabs by hand every time is tedious.** This
  fork adds Window Presets: declare window layouts in the config, then spawn
  and kill them on demand via launcher flags, so a known-good working set is
  always one command away. Includes a `SpawnCommand` `send_text` field so a
  preset tab can auto-run a command in the default shell. A preset tab's
  `cwd` may start with `~/`, and a tab that omits `cwd` opens in the previous
  tab's directory, so a preset for one project names its directory once. A
  tab may also give its directory as its first entry and a line to run in its
  shell as `cmd`. A preset window's title starts with `[<preset name>]`, so
  external tools can recognize it.
  *Usage: [docs/fork/window-presets.md](docs/fork/window-presets.md). Branch: [window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets).*

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
  fork adds Local Input Mode, a per-pane input mode that buffers keystrokes
  client-side and ships the line to the PTY only on submit. It is a convenience tool aimed
  at a minimally usable environment when the network is bad, not a perfect
  line editor. It is the newest patch in the stack, with known edge cases
  around paste and full key capture still open.
  *Usage: [docs/fork/local-input.md](docs/fork/local-input.md). Branch: [local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input).*

### Programs in a mux pane should get the keys they asked for

- **In a mux pane, a program that turns on the kitty keyboard protocol stops
  recognizing some keys, and Ctrl+Shift combinations cannot be typed at
  all.** The mux server runs the pane's terminal, so it answered the
  program's request and reported the protocol as on; but the GUI encodes the
  keys, and it never learned the pane's keyboard mode, so it kept sending the
  legacy encoding. A program that trusted the answer discarded legacy
  sequences such as `ESC ^E` for Ctrl+Alt+E, and Shift+Enter arrived as a
  plain Enter. The mux server now sends the pane's kitty keyboard flags to
  its clients along with the rest of the pane's state, and the GUI encodes
  keys for a mux pane with them, as it already did for a local pane. The
  protocol takes effect only when `enable_kitty_keyboard` is `true` for both
  the GUI and the mux server. Keys typed in the moment between a program
  switching the protocol on or off and the GUI hearing of it still use the
  previous encoding. Unlike legacy keys, kitty-encoded keys are not echoed
  ahead of the server on a slow connection. They are sent without waiting for
  the mux server, in order with the other keys, pastes and text typed into
  the pane; for the same reason `pane:send_text` on a mux pane now returns
  before the text reaches the mux server, and no longer reports a failure to
  deliver it.
  *Branch: not yet split out from `master`.*

### One session shared by several machines

- **Returning to a machine after using the same mux session from another one
  leaves its window showing content laid out for the other machine.** Each
  pane has one size on the mux server, set by whichever client resized it
  last, and a client only reports its size when its own window changes size,
  so the content overflows a smaller window or leaves a larger one
  half-filled until the window is resized by hand. The size now follows the
  client that was used last, like tmux's `window-size latest`: when a window
  gains focus, receives a key press, a click or a scroll, or switches tabs,
  it reports its own size for its active tab if the panes do not already
  have it. While two machines show the session at the same time, the one not
  in use shows the other's size until it is used again.
  *Branch: not yet split out from `master`.*

- **ssh in every pane suddenly stops finding its keys and asks for the key's
  passphrase.** Panes reach ssh-agent through a link the mux server keeps
  pointed at the agent of the most recently active client. A GUI started from
  inside a pane inherits that pane's `SSH_AUTH_SOCK`, which is the link
  itself, and reports it as its own agent; the mux server then pointed the
  link at itself, and every pane lost the agent until the session was
  restarted. The mux server now refuses a target that leads back to its own
  link, directly or through other links, keeps the working one, and logs
  `[fix:agent-self-link]`.
  *Branch: not yet split out from `master`.*

### The mux server should start reliably

- **Right after login, the GUI quits with "Connection refused" and leaves a
  mux server running with no window.** A GUI that finds no mux server starts
  one, and then gave it about two and a half seconds to start listening. Just
  after login the system is busy, and the server's startup plus the re-exec it
  does to daemonize can take several seconds, so the GUI gave up and exited
  while the server it had started came up a moment later. A client now keeps
  trying to reach a server it has just started for up to 15 seconds.
  *Branch: not yet split out from `master`.*

### WezTerm should not keep macOS busy in the background

- **On macOS 26 and later, Spotlight's indexing service writes gigabytes a
  day on WezTerm's behalf.** AppKit indexes every window of an app into
  Spotlight and resubmits all of them whenever any window title changes, with
  no coalescing; a terminal's titles change several times a second (shell
  directories, program titles, progress characters), and Spotlight rejects
  every one of those updates. WezTerm now turns this indexing off at launch
  through AppKit's undocumented `NSEnableWindowTabIndexing` default,
  registered so that `defaults write org.1b2c.wezterm
  NSEnableWindowTabIndexing -bool true` can still turn it back on.
  *Branch: not yet split out from `master`.*

### SSH domains should find hosts the way `ssh` does

- **An ssh domain fails with "failed to lookup address information" for a
  host that plain `ssh` connects to.** The host was defined in a file pulled
  in by `Include ~/.ssh/config.local`. WezTerm did not expand `~` in an
  `Include` argument, and it resolved a relative argument against the
  directory of the file containing the `Include`, so the included file was
  never read and the alias was treated as a literal host name. WezTerm now
  follows `ssh_config(5)`: a leading `~` is the home directory, and a
  relative path is resolved against `~/.ssh` in a user config file and
  against `/etc/ssh` in the system config file, however deeply the includes
  are nested. Globs keep working.
  *Branch: not yet split out from `master`.*

### Failures should explain themselves

- **When the remote SSH proxy command fails, all you get is an opaque
  connection error.** Fixed by surfacing a clear error message that says what
  actually failed.
  *Branch: [ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg).*

### Build and developer experience

These carry no user-facing symptom; listed for completeness.

- **[fork-base](https://github.com/0x1b2c/wezterm/compare/mux-deadlock-fix...fork-base)**
  The fork's own documentation (this file, `docs/fork/`, the README banner),
  the instructions for coding agents, and the fork's mux protocol version
  range. It sits right above `mux-deadlock-fix`, which stays directly on
  upstream for its pull request, so that every feature branch above it can
  update the fork's documentation in the same commit as the feature.

- **[macos-rerun-if-changed](https://github.com/0x1b2c/wezterm/compare/skip-resync-on-tab-resized...macos-rerun-if-changed)**
  Fixes the build script's `rerun-if-changed` path so macOS builds do not
  rebuild unnecessarily.

- ~~**[vendor-openssl-musl](https://github.com/0x1b2c/wezterm/compare/macos-rerun-if-changed...vendor-openssl-musl)**
  Vendors OpenSSL for musl targets to enable static cross-compilation.~~
  Superseded upstream: [wezterm#8028](https://github.com/wezterm/wezterm/pull/8028)
  (August 2026) makes the same `async_ossl` change, so the fork's version
  (April 2026) was dropped when the stack was rebased in October 2026. The
  original commit is kept as the tag
  [`fork/superseded/vendor-openssl-musl`](https://github.com/0x1b2c/wezterm/tree/fork/superseded/vendor-openssl-musl).
  The branch now carries only the `Cross.toml` setup that lets `cross` build
  the static Linux binaries in its container.

- **[objc-cargo-clippy-lint](https://github.com/0x1b2c/wezterm/compare/vendor-openssl-musl...objc-cargo-clippy-lint)**
  Whitelists the `objc` crate's `cargo-clippy` cfg in the workspace lint config
  to silence spurious warnings.

- **[macos-bundle-identity](https://github.com/0x1b2c/wezterm/compare/local-input...macos-bundle-identity)**
  Adds the `justfile` with the build and bundle recipes, and gives macOS
  builds the fork's own bundle identifier with a stable ad-hoc signature (see
  [Bundle identifier and signing](#bundle-identifier-and-signing)).

- **[windows-cross-build](https://github.com/0x1b2c/wezterm/compare/macos-bundle-identity...windows-cross-build)**
  Cross-compiles and packages the Windows (MinGW) build with one command, and
  moves machine-specific deploy recipes into the uncommitted `justfile.local`.

Beyond the branches above, the stack also carries a few commits tagged
`[local]`: a macOS `visibleFrame` window-geometry workaround, log tagging for
crash attribution, and build-noise suppression. These are experimental or
environment-specific attempts rather than user-facing features, and are not
documented individually.

## Building

Fork builds number their mux protocol in a range of their own
(`0x1b2c_UUNN`, with upstream's protocol version in `UU` and the fork's
revision in `NN`), so a fork build refuses to connect to an upstream WezTerm
mux server and the reverse, instead of exchanging messages neither side can
decode. Run the same fork build at both ends of a mux connection.

Upstream ships CI configuration but no simple local build script. This fork adds
a [`justfile`](justfile) at the repository root for that purpose. Run `just` to
list the available recipes.

The commonly used build recipes:

- `just build-arm64`: native arm64 release binaries (add `debug` for a debug build)
- `just bundle-arm64`: assemble `target/WezTerm.app` from the arm64 binaries
- `just build-x86`: `x86_64-apple-darwin` release binaries
- `just build-linux`: static `x86_64-unknown-linux-musl` binaries (requires `cross` and Docker)
- `just bundle-windows`: `x86_64-pc-windows-gnu` (MinGW) binaries packaged as `target/WezTerm-windows.zip` (requires `cross` installed from git and Docker; export `WEZTERM_WINDOWS_DOCKER_CONTEXT` from `justfile.local` to build on another Docker host, with `WEZTERM_WINDOWS_CROSS_REMOTE=1` when that host cannot see this checkout)

Machine-specific deploy recipes live in an uncommitted `justfile.local`, which
the justfile imports when present; the repository carries only the generic
build and bundle recipes.

If you prefer to build without `just`, follow upstream's source build
instructions at <https://wezterm.org/install/source.html>, building from this
fork's `master` branch instead of upstream `main`.

### Bundle identifier and signing

macOS builds of this fork use the bundle identifier `org.1b2c.wezterm`, not
upstream's `com.github.wez.wezterm`. The reason is privacy permissions. These
builds are ad-hoc signed, without a developer certificate, and macOS binds
privacy grants (Accessibility, Full Disk Access, and the like) to the app's
designated requirement. A plain ad-hoc signature ties that requirement to the
binary's hash, so every rebuild would lose its grants; the recipes instead sign
with a requirement that names only the bundle identifier, which every future
build satisfies.

macOS keeps one permission record per bundle identifier. If this fork reused
upstream's identifier, official builds and fork builds would compete for that
single record: granting a permission to one would replace the other's, and
switching between them would keep asking for the grants again. A distinct
identifier keeps the two builds' permissions separate. (A shared identifier
would also let official builds satisfy the fork's requirement, since that
requirement names only the identifier.) This is not a claim of authorship:
the app name, copyright notices, and license are unchanged, and WezTerm
remains Wez Furlong's work.

## License and attribution

The license is unchanged from upstream WezTerm. All upstream copyright and
attribution notices are preserved. See [LICENSE.md](LICENSE.md).
