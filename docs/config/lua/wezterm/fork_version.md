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
A local build made after a release reports how far it is from that release,
for example `1b2c-1.4.0-dev.3+abcd1234` (three commits later, built from
commit `abcd1234`); a build with no release among its ancestors, such as one
made after an upstream sync and before the next release, reports
`1b2c-dev+abcd1234`.

The fork version is meant for people to read.  To compare versions in a
configuration, use [wezterm.version](version.md), which keeps upstream's
date-based format.

```lua
local wezterm = require 'wezterm'
wezterm.log_info('Running ' .. wezterm.fork_version)
```
