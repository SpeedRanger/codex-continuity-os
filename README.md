# Codex Continuity OS

Local-first continuity layer for Codex.

The product goal is simple: open any repo and instantly understand what happened across Codex chats, why it changed, what files mattered, and what to do next.

## Current State

- Rust CLI app exists and builds locally
- local Codex session archive scanning works on real data
- `ccx sessions`, `ccx projects`, `ccx resume`, `ccx find`, `ccx compare`, and `ccx pack` are functional
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
- `ccx compare <session-a> <session-b>`
  - compares two sessions side-by-side
  - shows repo/context continuity, goals, outcomes, overlapping files, and unique files per session
- `ccx pack [--repo <path>] [--session <id>]`
  - generates a compact resume block for the next Codex chat
  - chooses both a latest checkpoint and a richer context anchor when possible

## Quickstart

Build with the intact local Rust toolchain:

```powershell
$toolchain='C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin'
$env:PATH="$toolchain;" + $env:PATH
cargo.exe build --bin ccx
```

Prime or refresh the local continuity cache:

```powershell
target\debug\ccx.exe index
```

Typical workflow:

```powershell
target\debug\ccx.exe projects
target\debug\ccx.exe resume --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe find "prompt profiles" --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe compare 019d1f8d-698d-70d1-b07d-f099066d4d34 019d30b1-1b6f-77a3-8c4b-cfcfe2d10973
target\debug\ccx.exe pack --repo D:\saas-workspace\products\roompilot-ai
```

## Cache Behavior

- index home: `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`
- `ccx index` explicitly rebuilds the cache
- normal commands prefer the last built cache for speed
- if no cache exists yet, the app scans the archive and writes one

## Verified Example

`ccx resume --repo D:\saas-workspace\products\roompilot-ai` now resolves to the March 27, 2026 session `019d30b1-1b6f-77a3-8c4b-cfcfe2d10973` even though that work originally ran from `D:\saas-workspace\templates\saas-mvp-template`.

## Current Limit

Current known limitations:

- the cache is intentionally manual-refresh via `ccx index`; it does not auto-reconcile every live Codex change yet
- file extraction is heuristic, so compare output can still include some non-project noise
- there is no packaged installer yet; current launch shape is repo + binary
