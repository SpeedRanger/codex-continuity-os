# Continuity

Last updated: 2026-04-10

## Current Goal

Ship the current CLI/TUI product as a launch-ready local-first continuity tool for Codex, with a real product-operating-system layer: PRD, task tracker, user flows, clear docs, cache-backed performance, and enough tests to trust the core heuristics.

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
- sessions now carry a deterministic continuity digest with summary, verification notes, and next-step hints
- commands now use a cache under `C:/Users/AKR/.codex-continuity/cache/session_index.tsv`
- `ccx index` explicitly rebuilds the cache and normal commands prefer cache reads
- `ccx dashboard` now acts as the interactive front door for the product
- dashboard detail now exposes summary plus verification and an extracted next-step panel
- dashboard session selection now includes explicit why-this-session reasoning
- dashboard now includes a persisted first-run onboarding/help overlay via `?`
- repeatable Windows release packaging now exists via `scripts/package-release.ps1`
- automated unit tests now cover attribution, search, cache serialization, file detection, file filtering, and pack prioritization
- public GitHub repo now exists at `https://github.com/SpeedRanger/codex-continuity-os`
- canonical product docs now exist for PRD, task tracking, and user flows
- verified recovery exists for `D:/saas-workspace/products/roompilot-ai`
- current grouped project count is `28`
- current scanned session count is `325`

## Last Change

Added a repeatable Windows release packaging path and produced the first versioned zip/checksum artifacts under `dist/`.

## Next Actions

1. Publish the packaged Windows release artifact on GitHub.
2. Tighten the dashboard UX so the front door feels more polished and more obvious.
3. Decide whether the next interface leap is a richer TUI pass or a local web UI.

## Blockers / Decisions

- Rust is chosen; Go is no longer under consideration for v1.
- The default `cargo` shim on this machine is broken for the active stable toolchain. Direct builds currently need an explicit intact toolchain on `PATH`, using:
  - `C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin`
- On Windows, rebuilding `target\debug\ccx.exe` while a prior `ccx` process is still running can fail with `Access is denied (os error 5)`. A dedicated `CARGO_TARGET_DIR` is the safest verification path during active iteration.
- Repo attribution is now stronger than pure `cwd` matching, but still heuristic. It relies on known product roots and transcript mentions, not a formal project identity graph yet.
- Cache freshness is explicit, not magical. `ccx index` is the refresh control.
- Product docs are now materially better organized, but they still need to stay synchronized with implementation as the dashboard and summary model evolve.
- The new summary layer is better than first-user / last-assistant extraction, but it is still deterministic heuristic extraction rather than true semantic summarization.
- The dashboard onboarding state is stored under the product’s own continuity home, not in Codex state, which is the right boundary, but it is still minimal and local-only.

## Canonical Workspace

- Repo path: `D:/saas-workspace/products/codex-continuity-os`
- Canonical global memory remains in `C:/Users/AKR/.codex/memories/`
- This folder is now a git repository with milestone commits.
