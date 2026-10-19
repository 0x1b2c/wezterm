---
tags:
  - keys
---
# `enable_kitty_keyboard = false`

{{since('20220624-141144-bd1b7c5d')}}

When set to `true`, wezterm will honor kitty keyboard protocol escape
sequences that modify the [keyboard encoding](../../key-encoding.md).

In a pane of a multiplexer domain, the mux server answers the program's
kitty keyboard requests while the GUI encodes the keys, so the protocol
takes effect only when this option is `true` in the configuration of both
the GUI and the mux server. With it on in the mux server alone, programs
are told that the protocol is on, but the GUI keeps sending the legacy
encoding.
