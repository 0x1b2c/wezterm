---
title: wezterm.fork_version
tags:
 - utility
 - version
---
# `wezterm.fork_version`

{{since('nightly')}}

This constant is set to this fork's own version string, the one that
`wezterm -V` reports before the protocol revision, for example `1b2c-1.4.0`.
A local build reports `1b2c-dev+abcd1234`, with the short hash of the commit
it was built from.

The fork version is meant for people to read.  To compare versions in a
configuration, use [wezterm.version](version.md), which keeps upstream's
date-based format.

```lua
local wezterm = require 'wezterm'
wezterm.log_info('Running ' .. wezterm.fork_version)
```
