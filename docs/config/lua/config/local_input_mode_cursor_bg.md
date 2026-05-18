---
tags:
  - appearance
  - input
---
# `local_input_mode_cursor_bg`

Background color used for the cursor while a pane is in
[`Local` input mode](../keyassignment/ToggleInputMode.md). Provides an
unambiguous visual signal that keystrokes are being buffered locally by
the GUI rather than transmitted to the PTY one at a time.

The default is a vivid amber (`#ffa500`) that contrasts with the cursor
colors used by most palettes.

```lua
config.local_input_mode_cursor_bg = '#ffa500'
```

When the pane returns to `Direct` input mode the cursor reverts to the
palette's normal `cursor_bg` setting.
