#!/bin/bash
# Wrapper around macOS ar that drops the noisy "has no symbols" warnings
# emitted when conditionally compiled C sources (e.g. libpng NEON, pixman
# timing, several cairo backends) produce empty object files for the
# non-active architecture. Apple's `ar -s` symbol-index step delegates to the
# ranlib code path, which is why the warnings are tagged "ranlib:" even
# though no separate ranlib invocation occurs.
exec /usr/bin/ar "$@" 2> >(grep -v 'has no symbols' 1>&2)
