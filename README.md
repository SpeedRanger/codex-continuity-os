# Codex Continuity OS

Local-first continuity layer for Codex.

The product goal is simple: open any repo and instantly understand what happened across Codex chats, why it changed, what files mattered, and what to do next.

## Current State

- Rust CLI app exists and builds locally
- local Codex session archive scanning works on real data
- `ccx sessions`, `ccx projects`, `ccx resume`, and `ccx find` are functional
- indirect repo attribution now recovers downstream repos from template-based sessions
- `compare`, `pack`, indexing, and launch hardening are still in progress

## Key Docs

- `PROJECT_SPEC.md`: compact product definition
- `CONTINUITY.md`: current working state and next steps
- `docs/CODEX_CONTINUITY_OS_MVP.md`: concrete MVP build spec
- `docs/IMPLEMENTATION_JOURNAL.md`: step-by-step build log for this launch push

## Product Shape

- CLI/TUI-first sidecar
- read-only around Codex
- local-first storage and indexing
- no dependence on private Codex internals

## Working Commands

- `ccx sessions`
  - lists historical sessions discovered from `~/.codex/sessions`
  - shows started time, session id, attributed repo, workspace repo, and first user goal
- `ccx projects`
  - groups sessions into project buckets using attributed repo roots
  - shows session counts and latest goal per project
- `ccx resume --repo <path>`
  - finds the best known session for a repo
  - now works even when the original session happened in an indirect workspace like `templates/saas-mvp-template`
- `ccx find "<query>" [--repo <path>] [--limit <n>]`
  - ranks matching sessions using goals, assistant outcomes, repo paths, mentioned repos, and session ids
  - prints why each result matched
  - supports project-scoped search

## Verified Example

`ccx resume --repo D:\saas-workspace\products\roompilot-ai` now resolves to the March 27, 2026 session `019d30b1-1b6f-77a3-8c4b-cfcfe2d10973` even though that work originally ran from `D:\saas-workspace\templates\saas-mvp-template`.

## Current Limit

This is not launch-ready yet. The current build still needs:

- session comparison
- resume-pack generation
- indexing and persistence
- broader automated tests
- final polish and launch docs
