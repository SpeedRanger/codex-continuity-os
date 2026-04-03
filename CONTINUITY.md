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
- verified recovery exists for `D:/saas-workspace/products/roompilot-ai`
- current grouped project count is `30`
- current scanned session count is `319`

## Last Change

Added secondary repo attribution based on known product roots mentioned inside session transcripts, then verified that `ccx resume --repo D:/saas-workspace/products/roompilot-ai` resolves the correct March 27, 2026 session even though the work ran from the template workspace.

## Next Actions

1. Implement `ccx find` with genuinely useful search over historical goals and assistant outcomes.
2. Implement `ccx compare` for side-by-side session context.
3. Implement `ccx pack` to generate a compact resume prompt from a chosen or inferred session.
4. Add index persistence and automated tests.
5. Do a final launch verification sweep and document exact usage.

## Blockers / Decisions

- Rust is chosen; Go is no longer under consideration for v1.
- The default `cargo` shim on this machine is broken for the active stable toolchain. Direct builds currently need an explicit intact toolchain on `PATH`, using:
  - `C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin`
- Repo attribution is now stronger than pure `cwd` matching, but still heuristic. It relies on known product roots and transcript mentions, not a formal project identity graph yet.

## Canonical Workspace

- Repo path: `D:/saas-workspace/products/codex-continuity-os`
- Canonical global memory remains in `C:/Users/AKR/.codex/memories/`
- This folder is currently not a git repository yet.
