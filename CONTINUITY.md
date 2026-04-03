# Continuity

Last updated: 2026-04-04

## Current Goal

Push Codex Continuity OS from early prototype into a launchable CLI product with real resume, search, compare, and pack workflows backed by local Codex history.

## State

- project home created under `D:/saas-workspace/products/codex-continuity-os`
- MVP spec moved out of `C:/Users/AKR/.codex`
- agreed product shape is a read-only CLI/TUI sidecar for Codex
- Rust selected for implementation because the toolchain is present locally and fits the speed/size goals
- cargo project scaffold created in-place
- `ccx` binary name is wired and the initial command shell boots cleanly
- first real session scanner works against `~/.codex/sessions`
- `ccx sessions` and `ccx projects` return real archive data
- `ccx resume` now supports indirect repo attribution and can recover downstream repos mentioned from template or worktree sessions
- `ccx find` now returns ranked matches with repo filtering and match reasons
- `ccx compare` now provides side-by-side continuity views with relation inference and repo-focused file overlap
- `ccx pack` now generates a pasteable resume pack with a latest checkpoint, context anchor, related sessions, and prioritized files
- verified recovery exists for `D:/saas-workspace/products/roompilot-ai`
- current grouped project count is `30`
- current scanned session count is `319`

## Last Change

Implemented `ccx pack` as a real resume-pack generator and verified it against `roompilot-ai` using both `--repo` and `--session`.

## Next Actions

1. Add index persistence so the app stops rescanning the archive on every command.
2. Add automated tests for parsing, attribution, search, compare helpers, and pack helpers.
3. Do a final launch verification sweep and document exact usage.

## Blockers / Decisions

- Rust is chosen; Go is no longer under consideration for v1.
- The default `cargo` shim on this machine is broken for the active stable toolchain. Direct builds currently need an explicit intact toolchain on `PATH`, using:
  - `C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin`
- On Windows, rebuilding `target\debug\ccx.exe` while a prior `ccx` process is still running can fail with `Access is denied (os error 5)`. A dedicated `CARGO_TARGET_DIR` is the safest verification path during active iteration.
- Repo attribution is now stronger than pure `cwd` matching, but still heuristic. It relies on known product roots and transcript mentions, not a formal project identity graph yet.

## Canonical Workspace

- Repo path: `D:/saas-workspace/products/codex-continuity-os`
- Canonical global memory remains in `C:/Users/AKR/.codex/memories/`
- This folder is currently not a git repository yet.
