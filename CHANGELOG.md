# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog and Semantic Versioning conventions where applicable.

## [Unreleased]

### Added

- Added server runtime plugin: `src/plugins/server.rs`.
- Added operations runbook: `docs/ops-runbook.md`.
- Added documentation completeness audit: `docs/documentation-completeness-audit-2026-02-23.md`.
- Added upgrade status SSOT: `docs/2026-upgrade-status.md`.
- Added architecture task board: `docs/2026-architecture-upgrade-tasks.md`.

### Changed

- Upgraded `redis` dependency from `0.23.3` to `0.32.7` to eliminate future-incompat warnings.
- Added `rust-version = "1.87"` to `Cargo.toml`.
- Added `[future-incompat-report] frequency = "always"` in `.cargo/config.toml`.
- Refactored `src/bin/server.rs` into network bootstrap + plugin-wired ECS runtime.
- Moved server game loop systems to `FixedUpdate` with explicit `60Hz` fixed timestep.
- Added client auto-reconnect and heartbeat ping systems in `src/systems/network.rs`.
- Reworked server broadcast path to per-client writer channels.
- Upgraded CI workflow in `.github/workflows/rust-ci.yml`:
  - Added `cargo check` (default features).
  - Added `cargo check --all-features --future-incompat-report`.
  - Switched clippy to strict mode: `cargo clippy --lib --all-features -- -D warnings`.
  - Switched tests to all-features mode: `cargo test --lib --all-features`.
  - Added workflow concurrency cancellation and manual trigger (`workflow_dispatch`).

### Documentation

- Reconciled `README.md` with current Bevy baseline and plugin architecture.
- Reworked `G-ENGINE-SETUP.md` to reflect implemented vs planned capabilities.
- Reframed `IMPLEMENTATION-SUMMARY.md` as an archival summary aligned with current code.
- Added verification checklist section to `SCENE_ENHANCEMENT.md`.
- Merged completed docs into `docs/2026-upgrade-status.md` and removed duplicated completed docs.
- Removed completed docs `IMPLEMENTATION-SUMMARY.md` and `docs/documentation-completeness-audit-2026-02-23.md` from active index.

## [2026-02-23]

### Verified

- `cargo fmt --check` passed.
- `cargo check` passed.
- `cargo check --all-features` passed.
- `cargo check --all-features --future-incompat-report` passed with zero future-incompat warnings.
- `cargo clippy --lib --all-features` passed.
- `cargo clippy --lib --all-features -- -D warnings` passed.
- `cargo test --lib` passed (85 passed, 0 failed).
- `cargo test --lib --all-features` passed (85 passed, 0 failed).
