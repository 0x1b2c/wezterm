# CLAUDE.md

A personal fork of WezTerm that carries a stack of patches on `master`; `FORK.md` describes each patch and the branch it lives on.

## Working rules

- Everything committed to this repository is written in English.
- Fork features are documented in `FORK.md` and `docs/fork/`, never in upstream's `docs/changelog.md`.
- Committed files never name local machines, user names, or local paths; machine-specific recipes belong in the uncommitted `justfile.local`.
- Any change to a mux PDU bumps `CODEC_VERSION` in `codec/src/lib.rs`.

## Verification

- Format: `cargo +nightly fmt --all` (CI checks it with `-- --check`).
- Lint: `cargo clippy -p <each crate the change touches> --no-deps`; a change must not add warnings. Upstream code already fails some deny-level lints (hence `--no-deps`), and warnings already present upstream are left alone.
- Test: `cargo nextest run -p <each crate the change touches>`; `make test` runs the whole workspace.
- New behavior is covered by Rust tests named after the behavior they prove.
