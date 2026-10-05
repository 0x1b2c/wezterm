## mux_window:get_preset_name()

{{since('nightly')}}

Returns the name of the [window preset](../config/window_presets.md) that
opened this window, or `nil` when the window was not opened from a preset.

The name is fixed when the window is created and cannot be changed from Lua.
Retitling the window does not affect it. In a GUI attached to a mux server it
reflects the preset the server recorded, even if the window was opened from
another machine.

This example shows the preset name at the left of the tab bar:

```lua
local wezterm = require 'wezterm'

wezterm.on('update-status', function(window, pane)
  local name = window:mux_window():get_preset_name()
  window:set_left_status(wezterm.format {
    { Text = name and (' ' .. name .. ' ') or '' },
  })
end)

return {}
```

See also [window:set_left_status](../window/set_left_status.md).
