# Launch Readiness

Last updated: 2026-04-09

## Current Status

Codex Continuity OS is now launchable as a source-first CLI/TUI product.

Public repository:

- `https://github.com/SpeedRanger/codex-continuity-os`

The current build has:

- cache-backed local session indexing
- interactive continuity dashboard
- project grouping
- repo resume
- ranked session search
- side-by-side compare
- resume-pack generation
- GitHub Actions CI
- CodeQL workflow
- Dependabot configuration
- security policy
- milestone git history
- implementation journal
- unit tests covering the core heuristics

## Build

Clone the repo:

```powershell
git clone https://github.com/SpeedRanger/codex-continuity-os.git
cd codex-continuity-os
```

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

Preferred interactive entrypoint:

```powershell
target\debug\ccx.exe dashboard
```

## Demo Commands

```powershell
target\debug\ccx.exe dashboard --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe projects
target\debug\ccx.exe resume --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe find "prompt profiles" --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe compare 019d1f8d-698d-70d1-b07d-f099066d4d34 019d30b1-1b6f-77a3-8c4b-cfcfe2d10973
target\debug\ccx.exe pack --repo D:\saas-workspace\products\roompilot-ai
```

Repeatable scripted demo:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\live-demo.ps1
```

## Verification Evidence

Manual verification completed against the live Codex archive:

- `dashboard --repo roompilot-ai` opened a real continuity board with the repo preselected
- dashboard search mode filtered the session pane to the `prompt profiles` result
- `projects` returned `30` grouped roots
- `resume --repo roompilot-ai` recovered the correct March 27, 2026 session from a template workspace
- `find "prompt profiles" --repo roompilot-ai` returned the expected March 24, 2026 session
- `compare` correctly identified the March 27 session as the later continuation of the March 24 session
- `pack --repo roompilot-ai` produced a usable resume block with the latest checkpoint and richer context anchor

Scripted verification completed on 2026-04-06 with the isolated demo binary:

- fresh build completed successfully
- `index` rebuilt the cache and indexed `322` sessions
- scripted full demo completed successfully end to end
- observed timings on the scripted run:
  - `index`: `57352ms`
  - `projects`: `103ms`
  - `resume`: `142ms`
  - `find`: `120ms`
  - `compare`: `144ms`
  - `pack`: `132ms`

Automated verification:

- `9` unit tests passed
- `0` unit test failures

## Known Limits

- cache refresh is manual through `ccx index`
- file extraction is heuristic, not AST- or git-diff-backed
- the dashboard currently optimizes for keyboard-driven operators, not first-time casual users
- there is no installer/package yet; current launch shape is repo + build instructions

## Recommended Launch Framing

Position it as:

- local-first continuity layer for Codex
- fast recovery for multi-chat, multi-repo work
- a continuity board plus search, compare, and resume packs for historical agent sessions

Do not position it yet as:

- perfect automatic project intelligence
- real-time live sync across active Codex sessions
- a polished desktop app
