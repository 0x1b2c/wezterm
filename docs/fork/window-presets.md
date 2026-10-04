# Window Presets

Declare named, multi-tab window layouts in the config, then spawn and kill
them on demand. The mux server materializes a whole preset in one action, so
a known-good working set is always one launcher action away instead of being
rebuilt tab by tab.

This page is the usage guide. Reference pages:
[`window_presets`](../config/lua/config/window_presets.md),
[`startup_windows`](../config/lua/config/startup_windows.md).

## Configuration

The API surface is small: each entry in a preset's `tabs` list is a
[SpawnCommand](../config/lua/SpawnCommand.md). The first entry creates the
window; the rest become additional tabs in order. The `send_text` field is a
fork addition: it writes the given bytes to the pane's stdin right after
spawn, which runs a command inside the default shell rather than replacing
the shell via `args`; a trailing newline is required for the shell to execute
the line. It is honored only by Window Presets materialization.

Raw `SpawnCommand` tables get repetitive for presets with many tabs in the
same project tree, so the author's actual config wraps them in a small Lua
helper where each tab is one of three compact forms:

* `{ 'dir' }` sets the cwd; `~/`-prefixed and absolute paths are used as
  given, and a bare name is anchored at the home directory.
* `{}` inherits the cwd from the previous tab.
* `{ cmd = '...' }` inherits the cwd and auto-runs a command in the default
  shell via `send_text`.

The cwd inheritance lives entirely in this helper; the underlying
`window_presets` API takes plain `SpawnCommand` entries.

```lua
local function build_tabs(entries)
  local tabs = {}
  local last_dir
  for _, ent in ipairs(entries) do
    local raw = ent[1]
    local dir
    if raw == nil then
      dir = last_dir
    elseif raw:sub(1, 2) == '~/' then
      dir = wezterm.home_dir .. raw:sub(2)
    elseif raw:sub(1, 1) == '/' then
      dir = raw
    else
      dir = wezterm.home_dir .. '/' .. raw
    end
    last_dir = dir

    local spawn = { cwd = dir }
    if ent.cmd then
      spawn.send_text = ent.cmd .. '\n'
    end
    table.insert(tabs, spawn)
  end
  return tabs
end

config.window_presets = {
  control_center = {
    tabs = build_tabs({
      { '~/Agentic/Skills' },
      { '~/Agentic/Config' },
      { '~/devel/live/typescript/site' },
      { cmd = 'bun run dev --host' }, -- same cwd as the previous tab
      {},                             -- plain shell, same cwd again
    }),
  },
  wezterm = {
    tabs = build_tabs({
      { '~/devel/cloned/wezterm' },
      {},
      {},
    }),
  },
}

-- Materialize these presets automatically when the mux server starts.
config.startup_windows = { 'control_center' }

-- Launcher entry points: Leader+w to fuzzy-pick and spawn,
-- Leader+Shift+w to pick a running preset and kill it.
config.leader = { key = 'g', mods = 'CTRL', timeout_milliseconds = 1000 }
config.keys = config.keys or {}
table.insert(config.keys, {
  key = 'w',
  mods = 'LEADER',
  action = wezterm.action.ShowLauncherArgs({ flags = 'FUZZY|WINDOW_PRESETS' }),
})
table.insert(config.keys, {
  key = 'W',
  mods = 'LEADER|SHIFT',
  action = wezterm.action.ShowLauncherArgs({ flags = 'FUZZY|WINDOW_PRESETS_KILL' }),
})
```

## Using the launcher

* `WINDOW_PRESETS` lists every preset with a status marker: running presets
  are marked distinctly from closed ones. Selecting a closed preset
  materializes it; selecting a running one is a no-op, so repeated
  invocations are idempotent.
* `WINDOW_PRESETS_KILL` lists only the running presets. Selecting one closes
  that window atomically, including all of its tabs.

A preset counts as "running" when a mux window carrying its marker exists.
The window a preset opens carries the preset name as a marker for its whole
life, set when the window is created; the window title plays no part, so a
program in the window retitling it does not make the preset look closed.
The marker lives on the mux server, which is how presets are discovered
across clients: materialize a preset from one machine and another attached
client sees it as running.

External tools can read the marker too: `wezterm cli list --format json`
reports a `window_preset` field for every pane, holding the name of the
preset that opened the pane's window, or `null` otherwise.

The `WINDOW_PRESETS` flag also composes with the general-purpose launcher;
the author's `Cmd+Shift+L` binding uses
`{ flags = 'FUZZY|WINDOW_PRESETS|DOMAINS' }` so presets, domains, and the
fuzzy filter share one menu.

## Startup windows

`config.startup_windows` names the presets the mux server materializes once
after its `mux-startup` event. Presets already running are skipped, so the
option is safe to keep configured when reattaching to a long-lived server.

Because the config is Lua, the list can be computed. For example, selecting a
profile via an environment variable at server launch:

```lua
local function parse_startup_windows()
  local env = os.getenv('WEZTERM_PROFILE')
  if not env or env == '' then
    return { 'control_center' }
  end
  local list = {}
  for w in env:gmatch('[^,]+') do
    table.insert(list, (w:gsub('^%s+', ''):gsub('%s+$', '')))
  end
  return list
end
config.startup_windows = parse_startup_windows()
```

```console
$ WEZTERM_PROFILE=wezterm,control_center wezterm
```

Note that `startup_windows` is executed by the mux server, so the environment
variable must reach the mux server process. The command above works when
launching the GUI spawns the local mux server, which inherits the GUI's
environment. A server started independently, such as a remote mux server,
needs the variable set in its own environment instead.

## Remote mux servers

Everything above works identically when the GUI is attached to a remote mux
server: the preset list comes from the server's config, and materialization
and kill both happen on the server. The launcher is the only entry point;
there are deliberately no per-preset key assignments.

## Notes and caveats

* Renaming or removing a preset in the config while its window is open
  orphans that window: the launcher no longer recognizes it, so selecting the
  new name opens a second window, and the old one can only be closed by hand.
* Two clients materializing the same closed preset at nearly the same moment
  can race past the idempotency check and produce two windows. The window
  for this race is narrow and it has not been observed in practice.
