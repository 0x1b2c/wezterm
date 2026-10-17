---
tags:
  - spawn
  - window_presets
---
# `window_presets`

{{since('nightly')}}

Declares named, multi-tab window templates that can be materialized as a
single atomic unit by the mux server.

Each entry is keyed by a preset name and contains a list of
[SpawnCommand](../SpawnCommand.md) entries, one per tab. The first entry
creates the window itself; subsequent entries are added as additional tabs in
the order given. The window a preset opens carries the preset name as a
marker for its whole life; programs that change the window title do not
affect it.

A preset is "running" when a window carrying its marker is present in the
mux. Materializing a preset that is already running is a no-op, which keeps
repeated invocations idempotent.

Presets can be:

* Auto-spawned at mux startup via [startup_windows](startup_windows.md).
* Surfaced through the launcher with the `WINDOW_PRESETS` flag (running
  presets are marked, closed ones can be materialized) or
  `WINDOW_PRESETS_KILL` (only running presets are listed; selecting one
  closes that window).

A tab's `cwd` may be written relative to the home directory: `~` names the
home directory itself, and a path starting with `~/` is resolved under it.
The home directory is that of the process materializing the preset, normally
the mux server whose configuration declares it; a preset on a remote mux
server therefore expands `~` to the remote machine's home directory. Other
forms, such as `~user/...`, are used as written.

A tab that omits `cwd` opens in the directory of the previous tab, so a
preset whose tabs share a project directory names it only once. Each tab
inherits the `cwd` the previous tab was given, after `~` expansion, not the
directory its shell has moved to since: a `cd` written through `send_text`
does not carry over. The previous tab's `args` and `domain` make no
difference; when the inherited path does not exist in the new tab's domain,
that domain falls back as it does for any missing directory (the local domain
opens the home directory). Until some tab names a directory, tabs that omit
`cwd` open in the default directory, which is
[default_cwd](default_cwd.md) or else the home directory.

A tab may also be written in two shorter forms. Its directory may be given
as the tab's first positional entry: `{ '~/projects/veil' }` is the same as
`{ cwd = '~/projects/veil' }`. A line to run in the tab's shell may be given
as `cmd`, which is typed followed by Enter: `{ cmd = 'make watch' }` is the
same as `{ send_text = 'make watch\n' }`. Unlike `args`, which runs a program
in place of the shell, `cmd` leaves the shell running once the line
finishes. A tab that gives its directory both ways, or that has both `cmd`
and `send_text`, is a configuration error.

The shorthand forms, the `~` expansion and the inheritance apply only to
window presets; a `SpawnCommand` used anywhere else treats `cwd` as it
always has.

```lua
config.window_presets = {
  ['control_center'] = {
    tabs = {
      { args = { 'htop' } },
      { args = { 'journalctl', '-f' } },
      { label = 'shell' },
    },
  },
  ['veil'] = {
    tabs = {
      {
        cwd = '~/projects/veil',
        args = { 'nvim', '.' },
      },
      -- No cwd: opens in ~/projects/veil, like the tab above.
      {},
      -- The same directory written as the first entry, with a line to run
      -- in the tab's shell.
      { '~/projects/veil', cmd = 'make watch' },
    },
  },
}
```

See also: [startup_windows](startup_windows.md).
