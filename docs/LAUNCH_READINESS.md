# Launch Readiness

Last updated: 2026-04-04

## Current Status

Codex Continuity OS is launchable as a source-first CLI product.

The current build has:

- cache-backed local session indexing
- project grouping
- repo resume
- ranked session search
- side-by-side compare
- resume-pack generation
- milestone git history
- implementation journal
- unit tests covering the core heuristics

## Build

Use the intact Rust toolchain directly:

```powershell
$toolchain='C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin'
$env:PATH="$toolchain;" + $env:PATH
cargo.exe build --bin ccx
```

## First Run

```powershell
target\debug\ccx.exe index
```

This creates the local cache at:

- `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`

## Demo Commands

```powershell
target\debug\ccx.exe projects
target\debug\ccx.exe resume --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe find "prompt profiles" --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe compare 019d1f8d-698d-70d1-b07d-f099066d4d34 019d30b1-1b6f-77a3-8c4b-cfcfe2d10973
target\debug\ccx.exe pack --repo D:\saas-workspace\products\roompilot-ai
```

## Verification Evidence

Manual verification completed against the live Codex archive:

- `projects` returned `30` grouped roots
- `resume --repo roompilot-ai` recovered the correct March 27, 2026 session from a template workspace
- `find "prompt profiles" --repo roompilot-ai` returned the expected March 24, 2026 session
- `compare` correctly identified the March 27 session as the later continuation of the March 24 session
- `pack --repo roompilot-ai` produced a usable resume block with the latest checkpoint and richer context anchor

Automated verification:

- `9` unit tests passed
- `0` unit test failures

## Known Limits

- cache refresh is manual through `ccx index`
- file extraction is heuristic, not AST- or git-diff-backed
- there is no installer/package yet; current launch shape is repo + build instructions

## Recommended Launch Framing

Position it as:

- local-first continuity layer for Codex
- fast recovery for multi-chat, multi-repo work
- search, compare, and resume packs for historical agent sessions

Do not position it yet as:

- perfect automatic project intelligence
- real-time live sync across active Codex sessions
- a polished desktop app
