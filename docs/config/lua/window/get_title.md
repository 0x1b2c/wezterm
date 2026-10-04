# `window:get_title()`

{{since('nightly')}}

Returns the title most recently handed to the operating system for this GUI
window. This is the text shown in the window's title bar, including the
`[<preset name>] ` prefix of a [window preset](../../../fork/window-presets.md)
window, for example `[control_center] [2/5] ✳ Claude Code`.

This differs from [mux_window:get_title()](../mux-window/get_title.md), which
returns the title stored on the mux window (set through `OSC 0`, `OSC 2` or
`set_title()`) and knows nothing about the formatting applied by
[format-window-title](../window-events/format-window-title.md) or about the
preset prefix.

```lua
wezterm.on('update-status', function(window, pane)
  window:set_right_status(window:get_title())
end)
```
