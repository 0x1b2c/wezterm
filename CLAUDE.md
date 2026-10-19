# CLAUDE.md

A personal fork of WezTerm that carries a stack of patches on `master`; `FORK.md` describes each patch and the branch it lives on.

## Working rules

- Everything committed to this repository is written in English.
- Fork features are documented in `FORK.md` and `docs/fork/`, never in upstream's `docs/changelog.md`.
- Committed files never name local machines, user names, or local paths; machine-specific recipes belong in the uncommitted `justfile.local`.
- A change to a mux PDU leaves the codec version alone in its own commit. `CODEC_VERSION` (`0x1b2c_UUNN`) combines upstream's own codec version, `UPSTREAM_CODEC_VERSION`, which the commit at the base of the stack keeps and which takes upstream's new number whenever an upstream sync conflicts on it, with the fork's protocol revision, `FORK_PROTOCOL_REVISION`. Between releases, each protocol change adds a `fixup!` commit for the stack's revision commit that raises the fork's revision by one; a release moves the revision commit to the top of the stack, squashes its fixups into it, and sets the revision to the number of patches that change the protocol. The comments on these constants in `codec/src/lib.rs` explain why.
- Release version numbers and the history rules that keep release tags meaningful are described in the Versioning section of `FORK.md`.

## Verification

- Format: `cargo +nightly fmt --all` (CI checks it with `-- --check`).
- Lint: `cargo clippy -p <each crate the change touches> --no-deps`; a change must not add warnings. Upstream code already fails some deny-level lints (hence `--no-deps`), and warnings already present upstream are left alone.
- Test: `cargo nextest run -p <each crate the change touches>`; `make test` runs the whole workspace.
- New behavior is covered by Rust tests named after the behavior they prove.
