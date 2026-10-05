---
title: wezterm.version
tags:
 - utility
 - version
---
# `wezterm.version`

This constant is set to the `wezterm` version string in upstream's
date-based format.  This can potentially be used to adjust configuration
according to the installed version.

In this fork, `wezterm -V` reports the fork's own version instead; see
[wezterm.fork_version](fork_version.md).  `wezterm.version` keeps upstream's
format so that comparisons written against upstream versions keep working.

The version string looks like `20200406-151651-5b700e4`.  You can compare the
strings lexicographically if you wish to test whether a given version is newer
than another; the first component is the date on which the release was made,
the second component is the time and the final component is a git hash.

```lua
local wezterm = require 'wezterm'
wezterm.log_error('Version ' .. wezterm.version)
```


