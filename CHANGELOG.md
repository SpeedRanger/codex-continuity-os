# Changelog

## v0.1.2 - 2026-04-22

### Changed

- Disabled unused `ratatui` default features and enabled only the `crossterm` backend.
- Pruned optional termwiz/phf/rand dependencies from `Cargo.lock`.
- Bumped package version to `0.1.2`.

### Verified

- `cargo fmt --check`.
- `cargo test`: `13 passed`, `0 failed`.
- `cargo tree --target all -i rand` no longer finds `rand` in the dependency graph.

## v0.1.1 - 2026-04-22

### Added

- Added `ccx doctor` for local Codex archive, cache, and current-repo diagnostics.
- Added `scripts/install-windows.ps1` for installing `ccx.exe` into `%USERPROFILE%\.codex-continuity\bin`.
- Added [docs/QUICKSTART.md](./docs/QUICKSTART.md) for install, first-run, daily workflow, dashboard keys, and cache behavior.

### Changed

- Normal commands now validate cache freshness against the local Codex archive before trusting cached sessions.
- Stale caches now auto-refresh on normal commands and report `session_source: auto-refresh`.
- Cache freshness allows a short active-session timestamp grace window so Codex live transcript writes do not trigger rebuild loops.
- Windows release packages now include `QUICKSTART.md`.
- Upgraded `ratatui` to `0.30.0`, which resolves the low-severity transitive `lru` advisory by moving to `lru 0.16.4`.

### Verified

- `cargo test`: `13 passed`, `0 failed`.
- `ccx doctor` reports archive/cache wiring and readiness.
- A stale cache auto-refreshed during `ccx resume --repo D:\saas-workspace\products\roompilot-ai`.
- The temporary Windows install smoke test copied `ccx.exe` and the installed binary ran `doctor` successfully.
- `cargo tree -i lru` resolves to `lru 0.16.4`.

## v0.1.0 - 2026-04-10

Initial public launch.

### Added

- Local Codex session archive scanner.
- Project/repo attribution across indirect workspaces.
- Deterministic continuity digests with summary, verification notes, and next-step hints.
- Cache-backed session index.
- `dashboard`, `sessions`, `projects`, `resume`, `find`, `compare`, `pack`, and `index`.
- Interactive terminal continuity board with first-run help.
- Windows release packaging script.
- Product docs, launch docs, architecture docs, and implementation journal.
