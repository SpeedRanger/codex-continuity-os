# Launch Readiness

Last updated: 2026-04-22

## Current Status

Codex Continuity OS is now launchable as a CLI/TUI product with a repeatable Windows packaging path.

Public repository:

- `https://github.com/SpeedRanger/codex-continuity-os`

The current build has:

- cache-backed local session indexing
- automatic cache refresh when the Codex archive changes
- deterministic session digest extraction
- interactive continuity dashboard
- first-run onboarding/help overlay in the dashboard
- `doctor` diagnostics for archive/cache/current-repo wiring
- Windows install helper script
- project grouping
- repo resume
- ranked session search
- side-by-side compare
- resume-pack generation
- GitHub Actions CI
- CodeQL workflow
- Dependabot configuration
- patched dependency floor for the previously reported low-severity transitive `lru` advisory
- unused `ratatui` default features disabled so optional termwiz/phf/rand dependencies are not carried in the lockfile
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

## Windows Release Packaging

To build a Windows release archive locally:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\package-release.ps1 -Version v0.1.2
```

This creates:

- `dist\ccx-windows-x86_64-v0.1.2.zip`
- `dist\ccx-windows-x86_64-v0.1.2.sha256.txt`

Verified on 2026-04-10:

- local package build succeeded
- zip artifact created successfully
- SHA256 checksum file created successfully

Verified on 2026-04-22 for `v0.1.1`:

- package build succeeded
- zip artifact created successfully
- SHA256 checksum file created successfully
- release zip expanded successfully
- expanded package contained `ccx.exe`, `README.md`, `LICENSE`, `QUICKSTART.md`, and `QUICKSTART.txt`
- expanded `ccx.exe doctor` ran successfully
- expanded `ccx.exe --version` returned `ccx 0.1.1`

Verified on 2026-04-22 for `v0.1.2`:

- package build succeeded
- zip artifact created successfully
- SHA256 checksum file created successfully
- release zip expanded successfully
- expanded package contained `ccx.exe`, `README.md`, `LICENSE`, `QUICKSTART.md`, and `QUICKSTART.txt`
- expanded `ccx.exe --version` returned `ccx 0.1.2`
- `cargo tree --target all -i rand` confirmed `rand` is absent from the dependency graph

## First Run

```powershell
target\debug\ccx.exe doctor
```

This verifies the Codex archive path, cache path, cache freshness, and suggested dashboard command.

Normal commands now refresh the local cache automatically when `~/.codex/sessions` changes. The cache lives at:

- `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`

Preferred interactive entrypoint:

```powershell
target\debug\ccx.exe dashboard
```

## Demo Commands

```powershell
target\debug\ccx.exe dashboard --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe doctor
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
- forced-first-run dashboard opened the onboarding overlay and persisted dismissal correctly
- dashboard detail pane showed extracted summary, verification notes, and next-step hint for `roompilot-ai`
- dashboard snapshot now includes explicit why-this-session reasoning for the selected row
- dashboard search mode filtered the session pane to the `prompt profiles` result
- `doctor` reports archive path, cache file path, freshness, archive file count, and suggested dashboard command
- cache loading now rejects stale cache headers and refreshes automatically on the next normal command
- cache freshness includes a short active-session timestamp grace window so Codex logging does not cause repeated rebuilds during live use
- temporary installer smoke test copied `ccx.exe` into `C:\Users\AKR\.codex\tmp\ccx-install-smoke` without modifying PATH
- installed smoke-test binary ran `doctor` successfully
- `cargo tree -i lru` confirmed `lru 0.16.4`
- `cargo tree --target all -i rand` confirmed `rand` is no longer in the dependency graph
- `projects` returned `28` grouped roots
- `resume --repo roompilot-ai` recovered the correct March 27, 2026 session from a template workspace
- `resume --repo roompilot-ai` now exposes summary, verification, and next-step fields directly
- `find "prompt profiles" --repo roompilot-ai` returned the expected March 24, 2026 session
- `compare` correctly identified the March 27 session as the later continuation of the March 24 session
- `pack --repo roompilot-ai` produced a usable resume block with the latest checkpoint, richer continuity summary, verification notes, and next-step hint

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

- `12` unit tests passed
- `0` unit test failures

## Known Limits

- automatic cache refresh uses archive file count and newest modified timestamp with a short active-session grace window, not a per-file content hash
- file extraction is heuristic, not AST- or git-diff-backed
- session summarization is deterministic heuristic extraction, not full semantic LLM summarization
- the dashboard currently optimizes for keyboard-driven operators, not first-time casual users
- onboarding exists, but the product is still terminal-first and assumes a builder/operator audience
- there is no cross-platform installer yet; current distribution path is source repo plus Windows release packaging

## Recommended Launch Framing

Position it as:

- local-first continuity layer for Codex
- fast recovery for multi-chat, multi-repo work
- a continuity board plus search, compare, and resume packs for historical agent sessions

Do not position it yet as:

- perfect automatic project intelligence
- real-time live sync across active Codex sessions
- a polished desktop app
