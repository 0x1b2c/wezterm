---
tags:
  - spawn
  - window_presets
  - startup_windows
---
# `startup_windows`

{{since('nightly')}}

A list of preset names that the mux server should materialize automatically
once the `mux-startup` event has been processed. Each entry must reference a
key defined in [window_presets](window_presets.md).

Presets that are already running (i.e. a window with the matching title
already exists in the mux) are skipped, so this option is safe to leave
configured even when reattaching to an existing mux server.

```lua
config.window_presets = {
  ['control_center'] = {
    tabs = {
      { args = { 'htop' } },
      { label = 'shell' },
    },
  },
}

config.startup_windows = { 'control_center' }
```

See also: [window_presets](window_presets.md).
