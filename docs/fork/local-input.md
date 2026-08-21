# Local input mode

A per-pane input mode that buffers keystrokes in a client-side line editor
and sends the whole line to the PTY only when submitted. Over a high-latency
mux connection this removes the per-keystroke network round trip: typing
feels local, and only the submit pays the network cost.

Local input mode is a convenience tool aimed at a minimally usable
environment when the network is bad. It is not, and does not aim to be, a
perfect line editor.

This page is the usage guide. Reference pages:
[`ToggleInputMode`](../config/lua/keyassignment/ToggleInputMode.md),
[`local_input_mode_cursor_bg`](../config/lua/config/local_input_mode_cursor_bg.md).

## Quick start

```lua
-- There is no default binding; opt in with one.
config.keys = config.keys or {}
table.insert(config.keys, {
  key = 'i',
  mods = 'CTRL|SHIFT',
  action = wezterm.action.ToggleInputMode,
})

-- Optional: the cursor color used while a pane is in Local mode.
-- Defaults to amber (#ffa500).
config.local_input_mode_cursor_bg = '#ffa500'
```

Press the binding to switch the active pane between `Direct` (the default,
send every keystroke immediately) and `Local`. While in Local mode:

* Typed characters are echoed locally at the pane's cursor position; nothing
  is sent to the PTY yet.
* The cursor turns amber (or your configured color) as the mode indicator.
* `Enter` submits the whole buffer to the PTY and clears it; the pane stays
  in Local mode for the next line.
* `Shift-Enter` inserts a newline into the buffer for multi-line input.
* Readline-style editing works locally: `Ctrl-A`/`Ctrl-E`, `Ctrl-W`,
  `Ctrl-U`, `Ctrl-K`, word motion, and arrow keys. The full key table is in
  the [`ToggleInputMode`](../config/lua/keyassignment/ToggleInputMode.md)
  reference.

The mode is sticky and per-pane, with the mux server as the authoritative
source: it survives client reconnects, and all clients attached to the same
server see the same mode for a pane.

Toggling back to Direct with a non-empty buffer does not discard it; the
buffered text is drained through the normal direct input path, as if typed.

## Interaction with key bindings

WezTerm's global key bindings run before the line editor. A key bound to a
global action never reaches the editor while in Local mode. This is the
escape hatch for keys that must bypass buffering, but it also means a
binding can shadow an editor key: for example, binding `Shift-Enter` to
`SendString` globally makes multi-line input unavailable in Local mode.

## When to use it, and known limitations

The sweet spot is long-form text into chat-style prompts over a slow link:
messages to a TUI assistant, commit messages, long command lines. It gets
especially valuable when the mux server runs on a machine that has dozed
off and every round trip stretches to seconds.

Known limitations, accepted by design or still open:

* **Active TUIs redraw over the local echo.** Programs that continuously
  repaint their input area (Claude Code and similar) overwrite the locally
  rendered buffer; the amber cursor remains visible, but the text can
  flicker or vanish until repainted. Static prompts (plain shells, REPLs
  awaiting input) are unaffected.
* **Multi-line submits into a plain shell execute line by line.** The submit
  path writes raw text, so a shell treats every newline as "run this line".
* **Readline's inline features are unavailable while buffering.** Tab
  completion, history search, and syntax highlighting react only to what the
  PTY receives, which in Local mode is nothing until submit.
* **Paste bypasses the buffer.** `Cmd+V` currently goes straight to the
  remote PTY instead of into the local editor.
