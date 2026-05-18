# `ToggleInputMode`

Toggles the active pane between `Direct` and `Local` input mode.

In `Direct` mode (the default), every keystroke is sent to the pane's PTY
one at a time. For a remote pane reached through a mux connection, that
means a network round-trip per keystroke, which is the dominant source of
perceived latency when typing long-form text into chat-style TUI apps over
a high-latency link.

In `Local` mode, keystrokes are buffered by a client-side line editor and
echoed locally onto the pane's display. The buffered text is only
transmitted to the PTY when the user presses `Enter`, eliminating the
per-keystroke network cost and giving near-native input responsiveness for
the duration of the local edit.

The mode is sticky and per-pane; it lives on the mux server side so it
survives client reconnects and is consistent across multiple connected
clients of the same mux server. The cursor is rendered in a configurable
color (see [`local_input_mode_cursor_bg`](../config/local_input_mode_cursor_bg.md))
while in `Local` mode so the user can see at a glance which mode is active.

```lua
config.keys = {
  {
    key = 'i',
    mods = 'CTRL|SHIFT',
    action = wezterm.action.ToggleInputMode,
  },
}
```

There is no default key binding; users must opt in by defining one.

## Local-mode key bindings

While the pane is in `Local` mode, the line editor consumes the following
keys (anything else falls through to the wezterm key-binding system, which
takes precedence over the editor and provides the orthogonal escape hatch
for keys that need to bypass the editor — bind those to `SendKey` or
`SendString`):

| Key                       | Action                                    |
| ------------------------- | ----------------------------------------- |
| `Enter`                   | Submit buffer to PTY, exit edit           |
| `Shift-Enter`             | Insert newline into buffer (multi-line)   |
| `Backspace`               | Delete character before cursor            |
| `Delete`                  | Delete character at cursor                |
| `Ctrl-A` / `Home`         | Move cursor to start of line              |
| `Ctrl-E` / `End`          | Move cursor to end of line                |
| `Ctrl-B` / `Left`         | Move cursor one character left            |
| `Ctrl-F` / `Right`        | Move cursor one character right           |
| `Ctrl-Shift-B`            | Move cursor one word left                 |
| `Ctrl-Shift-F`            | Move cursor one word right                |
| `Ctrl-P` / `Up`           | Move cursor up one line in the buffer     |
| `Ctrl-N` / `Down`         | Move cursor down one line in the buffer   |
| `Ctrl-W`                  | Delete word before cursor                 |
| `Ctrl-U`                  | Delete from cursor to start of line       |
| `Ctrl-K`                  | Delete from cursor to end of line         |

When toggling out of `Local` mode while a buffer is non-empty, the buffer
is drained through the same writer path that direct-mode keystrokes use,
so no typed characters are lost.
