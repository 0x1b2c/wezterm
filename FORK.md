中文: [FORK.zh-CN.md](FORK.zh-CN.md)

# FORK.md

A fork of [WezTerm](https://github.com/wezterm/wezterm), maintained by
[0x1b2c](https://github.com/0x1b2c).

## Overview

Not a downstream distribution: no rebranding and no intent to become a
distinct product. It does publish numbered releases with prebuilt
packages and a Homebrew tap, so it can be installed without building it (see
[Installing](#installing)). It carries a stack of patches
on top of upstream WezTerm's `main`: daily-driver fixes and features for latency,
multiplexer correctness, and workflow ergonomics not yet available upstream.

Each feature lives on its own branch; `master` integrates the whole stack. One
patch has been submitted upstream (see below); the rest are fork-only. Everything
else is unchanged from upstream. For general WezTerm usage, see
<https://wezterm.org/>.

Report problems with this fork's builds at
<https://github.com/0x1b2c/wezterm/issues>, not to upstream WezTerm: a problem
may come from one of the fork's patches.

## Problems this fork solves

The patches are grouped by theme, each rooted in a real annoyance from daily
use, most of it centered on WezTerm's multiplexer over SSH. Each entry names
the problem, then says what changes with this fork and what you need to know
to use it; the branch carrying the patch is linked at the end, and how each
fix works is explained in its commits. Build and developer-experience
patches, which change nothing users see, are listed by branch name in the
final section.

### The GUI freezes or lags badly under mux load

- **Reconnecting to a mux server with many tabs open freezes the GUI.**
  Reconnecting now completes without freezing, with several windows and
  around fifty tabs open.
  *Branch: [mux-deadlock-fix](https://github.com/0x1b2c/wezterm/compare/main...mux-deadlock-fix), submitted upstream as [PR #7771](https://github.com/wezterm/wezterm/pull/7771).*

- **Resizing a window with many tabs open freezes the GUI.** Resizing now
  stays responsive with several windows and around fifty tabs open.
  *Branch: [skip-resync-on-tab-resized](https://github.com/0x1b2c/wezterm/compare/ssh-proxy-error-msg...skip-resync-on-tab-resized).*

- **Programs that update the terminal title rapidly make the tab bar stutter.**
  The tab bar stays smooth while programs such as Codex rewrite their
  titles many times a second. When a program sets its title three or more
  times within a fifth of a second, its tab shows that title as it is, without
  calling a `format-tab-title` handler, until the title has stopped changing
  for a second.
  *Branch: [tab-title-perf](https://github.com/0x1b2c/wezterm/compare/build-tooling...tab-title-perf).*

- **Keystrokes feel sluggish.** Typed characters now appear without waiting
  for the next scheduled frame.
  *Branch: [keystroke-render-poll](https://github.com/0x1b2c/wezterm/compare/fork-base...keystroke-render-poll).*

### Declare window layouts once, spawn them on demand

- **Setting up the same windows and tabs by hand every time is tedious.** This
  fork adds Window Presets: declare window layouts in the config, then open
  and close them on demand from the launcher, so a configured set of windows
  is always one command away; `startup_windows` opens chosen presets when the
  mux server starts. A tab can name its directory, which may start with `~/`,
  and a line to run in its shell; a tab that names no directory opens in the
  previous tab's, so a preset for one project names its directory once. The
  short form `{ '~/project', cmd = 'make watch' }` stands for
  `{ cwd = '~/project', send_text = 'make watch\n' }`. A preset window's
  title starts with `[<preset name>]`, so external tools can recognize it.
  Presets are opened by the mux server, so these settings take effect once
  the mux server runs this fork.
  *Usage: [docs/fork/window-presets.md](docs/fork/window-presets.md). Branch: [window-presets](https://github.com/0x1b2c/wezterm/compare/session-experience...window-presets).*

- **There is no keybinding action to close a whole mux window at once**, which
  resetting a layout needs. `wezterm.mux.kill_window(mux_window)` now closes a mux
  window and all its tabs in one step. Binding it to `Cmd+Shift+W` is
  recommended:

  ```lua
  { key = 'w', mods = 'CMD|SHIFT', action = wezterm.action_callback(function(window)
    wezterm.mux.kill_window(window:mux_window())
  end) },
  ```
  *Branch: [lua-kill-window](https://github.com/0x1b2c/wezterm/compare/tab-title-perf...lua-kill-window).*

- **Closing one window can detach a mux domain that other windows are still
  using.** The domain now stays attached while any window still uses it.
  *Branch: [session-experience](https://github.com/0x1b2c/wezterm/compare/lua-kill-window...session-experience).*

### Typing over a slow connection is painful

- **On a high-latency link, every keystroke takes a full round trip before it
  appears on screen.** It gets especially bad when the mux server runs on a
  Mac that has dozed off, where each round trip stretches from milliseconds
  to seconds. Local Input Mode, switched on per pane, lets you type and edit
  a whole line locally and sends it only when you submit it. It is meant to
  keep a minimally usable environment on a bad network, not to be a full line
  editor. Known limitations: programs that keep repainting their input area,
  such as Claude Code, draw over the local text; `Cmd+V` pastes straight to
  the remote program; Tab completion, history search and other readline
  features do not work while you type locally; and a multi-line submit into a
  plain shell runs line by line. The usage guide describes them.
  *Usage: [docs/fork/local-input.md](docs/fork/local-input.md). Branch: [local-input](https://github.com/0x1b2c/wezterm/compare/window-presets...local-input).*

### Full Kitty Keyboard Protocol support

- **Programs running in a mux server (for example when the default domain is a
  unix domain, or over an SSH domain to a remote machine) miss keys once they
  turn on the Kitty Keyboard Protocol, and Ctrl+Shift combinations cannot be
  typed at all.** Upstream WezTerm supports the full protocol only when no mux
  server is involved; with this fork, programs in a mux server get it too:
  Ctrl+Shift combinations, Shift+Enter apart from Enter, Ctrl+I apart from
  Tab, and Ctrl+Alt letter shortcuts all reach them. It needs
  `enable_kitty_keyboard = true` in the config of both the GUI and the mux
  server, and both running this fork with protocol version 46.3 or later (see
  [Versioning](#versioning)). On a slow connection, keys sent in the Kitty
  Keyboard Protocol encoding are not echoed ahead of the server.
  `pane:send_text` on a pane in a mux server returns before the text reaches
  the mux server and no longer reports a failure to deliver it.
  *Branch: [kitty-keyboard](https://github.com/0x1b2c/wezterm/compare/mux-startup...kitty-keyboard).*

### One session shared by several machines

- **Returning to a machine after using the same mux session from another one
  leaves its window showing content laid out for the other machine's screen
  size.** The
  panes now take the size of the machine in use, like tmux's
  `window-size latest`: as soon as a window gains focus, receives a key
  press, a click or a scroll, or switches tabs, its panes are resized to fit
  it. While two machines show the session at the same time, the one not in
  use shows the other's size until it is used again.

- **ssh in every pane suddenly stops finding its keys and asks for the key's
  passphrase.** This happened after starting a WezTerm GUI from inside a
  pane. Starting a GUI that way no longer takes ssh-agent away from the other
  panes.
  *Branch: [shared-session](https://github.com/0x1b2c/wezterm/compare/windows-cross-build...shared-session).*

### The mux server should start reliably

- **When the mux server is slow to start, the GUI quits with "Connection
  refused" and leaves a mux server running with no window.** The GUI now waits
  up to 15 seconds for a mux server it has just started, so it opens normally
  even when the server takes a while to come up.
  *Branch: [mux-startup](https://github.com/0x1b2c/wezterm/compare/ssh-config-include...mux-startup).*

### WezTerm should not keep macOS busy in the background

- **On macOS 26 and later, Spotlight's indexing service writes gigabytes a
  day on WezTerm's behalf.** A terminal's window titles change several times
  a second, and macOS resubmits every window to Spotlight each time. WezTerm
  now turns that indexing off. To turn it back on, run
  `defaults write org.1b2c.wezterm NSEnableWindowTabIndexing -bool true`.
  *Branch: [spotlight-window-titles](https://github.com/0x1b2c/wezterm/compare/shared-session...spotlight-window-titles).*

### SSH domains should find hosts the way `ssh` does

- **An ssh domain fails with "failed to lookup address information" for a
  host that plain `ssh` connects to**, when the host is defined in a file
  pulled in by `Include`, such as `Include ~/.ssh/config.local`. Ssh domains
  now read included files the way `ssh` does (`ssh_config(5)`): a leading `~`
  is the home directory, and a relative path is looked up in `~/.ssh` from
  the user's config and in `/etc/ssh` from the system config, however deeply
  the includes are nested. Globs keep working.
  *Branch: [ssh-config-include](https://github.com/0x1b2c/wezterm/compare/spotlight-window-titles...ssh-config-include).*

### Failures should explain themselves

- **When the remote SSH proxy command fails, all you get is an opaque
  connection error.** The error now says what actually failed.
  *Branch: [ssh-proxy-error-msg](https://github.com/0x1b2c/wezterm/compare/keystroke-render-poll...ssh-proxy-error-msg).*

### Build and developer experience

These change nothing users see; listed for completeness.

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

- **[build-tooling](https://github.com/0x1b2c/wezterm/compare/objc-cargo-clippy-lint...build-tooling)**
  Adds the `justfile` with the build, bundle, and signing recipes, which CI
  also uses to sign the macOS package, and silences the "has no symbols"
  warnings that macOS builds print for empty object files.

- **[macos-bundle-identity](https://github.com/0x1b2c/wezterm/compare/local-input...macos-bundle-identity)**
  Gives macOS builds the fork's own bundle identifier with a stable ad-hoc
  signature (see [Bundle identifier and signing](#bundle-identifier-and-signing)).

- **[windows-cross-build](https://github.com/0x1b2c/wezterm/compare/macos-bundle-identity...windows-cross-build)**
  Cross-compiles and packages the Windows (MinGW) build with one command, and
  moves machine-specific deploy recipes into the uncommitted `justfile.local`.

- **[fork-release](https://github.com/0x1b2c/wezterm/compare/kitty-keyboard...fork-release)**
  Release plumbing: `wezterm --version` names the fork release and the mux
  protocol it speaks, local builds report the release they descend from, GitHub
  Actions builds the packages and publishes a release for every fork release
  tag, and the installation instructions for those releases (see
  [Installing](#installing) and [Versioning](#versioning)).

## Installing

Each release is published on the
[GitHub Releases](https://github.com/0x1b2c/wezterm/releases) page, with a
`.sha256` file beside every package.

### macOS: Homebrew

```sh
brew install --cask 0x1b2c/wezterm/wezterm-1b2c
```

The cask installs `WezTerm.app` and links `wezterm`, `wezterm-gui`,
`wezterm-mux-server` and `strip-ansi-escapes` into Homebrew's `bin`. It
conflicts with the official `wezterm` and `wezterm@nightly` casks, which link
the same names, so uninstall one before installing the other. The packages are
ad-hoc signed and not notarized; the cask clears the quarantine attribute from
the app after installing it.

### macOS: direct download

Download `WezTerm-1b2c-<version>-macos-arm64.zip` (Apple silicon) or
`WezTerm-1b2c-<version>-macos-x86_64.zip` (Intel) and unzip it. Because the app
is not notarized, macOS refuses to open it while the download's quarantine
attribute is set. Clear the attribute, then move the app to `/Applications`:

```sh
xattr -cr WezTerm.app
```

### Windows

Download `WezTerm-1b2c-<version>-windows-x86_64.zip`, unzip it, and run
`wezterm-gui.exe`. The executables are not signed, so Windows SmartScreen may
show "Windows protected your PC". Choose "More info", then "Run anyway". To
avoid the prompt for every executable, open the zip's Properties before
unzipping and tick "Unblock".

### Linux

`wezterm-1b2c-<version>-linux-x86_64-musl.tar.gz` holds static `wezterm` and
`wezterm-mux-server` binaries, for the command line and for running a mux
server on a remote machine. There is no GUI package for Linux; build one from
source (see [Building](#building)).

## Building

A fork build and an upstream WezTerm build cannot work together over the mux
protocol: the fork changes the protocol, so the two are incompatible. The fork
numbers its protocol version in a range of its own (`0x1b2c_UUNN`, with
upstream's protocol version in `UU` and the fork's revision in `NN`), so each
side recognizes the other as incompatible and refuses the connection with a
clear message instead of misbehaving. Run fork builds with the same protocol version at
both ends of a mux connection.

Upstream ships CI configuration but no simple local build script. This fork adds
a [`justfile`](justfile) at the repository root for that purpose. Run `just` to
list the available recipes.

The commonly used build recipes:

- `just build-arm64`: native arm64 release binaries (add `debug` for a debug build)
- `just bundle-arm64`: assemble `target/WezTerm.app` from the arm64 binaries
- `just build-x86`: `x86_64-apple-darwin` release binaries
- `just build-linux`: static `x86_64-unknown-linux-musl` binaries (requires `cross` and Docker; uses the same Docker host settings as `bundle-windows`)
- `just bundle-windows`: `x86_64-pc-windows-gnu` (MinGW) binaries packaged as `target/WezTerm-windows.zip` (requires `cross` installed from git and Docker; export `WEZTERM_CROSS_DOCKER_CONTEXT` from `justfile.local` to build on another Docker host, with `WEZTERM_CROSS_REMOTE=1` when that host cannot see this checkout)

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
with a requirement that names only the bundle identifier, so installing a new
build never asks for the grants again.

macOS keeps one permission record per bundle identifier. If this fork reused
upstream's identifier, official builds and fork builds would compete for that
single record: granting a permission to one would replace the other's, and
switching between them would keep asking for the grants again. A distinct
identifier keeps the two builds' permissions separate. (A shared identifier
would also let official builds satisfy the fork's requirement, since that
requirement names only the identifier.) This is not a claim of authorship:
the app name, copyright notices, and license are unchanged, and WezTerm
remains Wez Furlong's work.

## Versioning

Releases of this fork are numbered `MAJOR.MINOR.PATCH` and published under
the name `1b2c-<version>`, for example `1b2c-1.4.0`. That name is the git
tag, the GitHub release, and what `wezterm --version` prints, so a version
reported anywhere points straight at its tag. Upstream's own versions are
dates, so the two never look alike.

- **MINOR** goes up when a release brings a new batch of features, and
  PATCH starts again at 0.
- **PATCH** goes up for a release with no new features, typically one that
  only brings the fork up to date with upstream.
- **MAJOR** marks a milestone and changes only rarely, by deliberate choice
  rather than by rule.

The first release is 1.4.0 rather than 1.0.0 because four batches of
features came before it: the mux and latency fixes of late March and early
April 2026, the tab and window management work and Window Presets of late
April, Local Input Mode in May, and stable identities for preset windows in
October. They were never released on their own, so 1.1 to 1.3 have no tags.

`wezterm --version` also prints the mux **protocol version**, as in
`wezterm 1b2c-1.4.0 (protocol 46.3)`: upstream's protocol version, then the
fork's own protocol revision. The protocol version is independent of the
release version: a GUI and a mux server can talk to each other only when
both numbers match (see [Building](#building)). Release notes say so
whenever a release changes it, because every machine sharing mux sessions
then has to be upgraded together.

Scripts keep seeing upstream's format: `wezterm.version` in Lua,
`TERM_PROGRAM_VERSION`, and the update check still report a date-based
version, so comparisons written against upstream versions keep working. The
fork version is available to configurations as
[`wezterm.fork_version`](docs/config/lua/wezterm/fork_version.md).

### Release tags and history

This fork is a stack of patches rebased onto upstream, which rewrites its
history. To keep release tags meaningful, history is only appended to
between upstream syncs: after a release, new commits go on top of the
stack, and moving commits into the branch they belong to waits for the next
upstream sync, when the whole stack is rebased anyway. Within one upstream
base, every release tag is therefore an ancestor of `master`.

An upstream sync breaks that ancestry: the tags of earlier releases point at
the stack as it was before the rebase, and they stay valid as records of
exactly what was released. Each release's notes name the upstream commit it
is based on. A local build's version says which release it was built after:
`1b2c-1.4.0-dev.3+abcd1234` is three commits past 1.4.0, built from commit
`abcd1234`.

## License and attribution

The license is unchanged from upstream WezTerm. All upstream copyright and
attribution notices are preserved. See [LICENSE.md](LICENSE.md).
