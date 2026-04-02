# CLAW.md

This file provides guidance to Claw Code when working with code in this repository.

## Detected stack

- Languages: Rust (primary), Python (support/tooling).
- Frameworks: none detected from the supported starter markers.

## Verification

- Run Rust verification from root: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- Build release: `cargo build --release`
- `python/` contains Python support scripts; run tests with `python3 -m unittest discover -s python/tests -v`

## Repository shape

- `crates/` contains the Rust workspace with all CLI/runtime implementation.
- `python/` contains Python support/tooling scripts (secondary).
- `tests/` contains Python validation surfaces.
- `docs/` contains documentation and release notes.

## Working agreement

- Prefer small, reviewable changes and keep generated bootstrap files aligned with actual repo workflows.
- Keep shared defaults in `.claw.json`; reserve `.claw/settings.local.json` for machine-local overrides.
- Do not overwrite existing `CLAW.md` content automatically; update it intentionally when repo workflows change.
- Rust is the primary implementation language; Python is for tooling and prototyping only.
