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
forms, such as `~user/...`, are used as written. This expansion applies only
to window presets; a `SpawnCommand` used anywhere else passes `cwd` through
unchanged.

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
      {
        cwd = '~/projects/veil',
      },
    },
  },
}
```

See also: [startup_windows](startup_windows.md).
