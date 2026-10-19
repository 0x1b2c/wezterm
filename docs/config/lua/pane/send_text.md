# `pane:send_text(text)`

{{since('20220624-141144-bd1b7c5d')}}

Sends text to the pane as-is.

In a pane of a multiplexer domain, the text is queued behind the keys and
pastes already sent to the pane, and the method returns before the text
reaches the mux server, so it does not report a failure to deliver it.
