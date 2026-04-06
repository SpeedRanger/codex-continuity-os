# Codex Continuity OS

Local-first continuity layer for Codex.

Codex Continuity OS helps you recover context across chats, repos, branches, and indirect workspaces without modifying Codex itself. It reads your local Codex session archive, builds a project-aware index, and gives you fast commands to resume work, search old sessions, compare two chats, and generate a resume pack for the next session.

Public repo:

- `https://github.com/SpeedRanger/codex-continuity-os`

## Why This Exists

`codex resume` is good at reopening a session id.

It is much weaker at answering the questions that matter after a break:

- what were we doing in this repo?
- which chat actually mattered?
- which session talked about this feature?
- what changed between those two chats?
- what context should the next Codex session start with?

This project exists to solve that gap.

## What It Does

- reads local Codex rollout archives from `~/.codex/sessions`
- attributes sessions to the real project repo, even when the work happened from a template/worktree
- groups sessions into project-level views
- ranks historical chats by query
- compares two sessions side by side
- generates a pasteable resume block for the next Codex chat
- caches the normalized index locally for faster repeat usage

## Command Surface

- `ccx sessions`
  - list recent known sessions with attributed repo and first user goal
- `ccx projects`
  - group sessions by project/repo and show the latest known context
- `ccx resume --repo <path>`
  - recover the best known session for a repo
- `ccx find "<query>" [--repo <path>] [--limit <n>]`
  - ranked search across historical sessions
- `ccx compare <session-a> <session-b>`
  - side-by-side session continuity view
- `ccx pack [--repo <path>] [--session <id>]`
  - generate a compact resume pack for the next Codex chat
- `ccx index`
  - rebuild the local cache

## Why It Is Useful

The key behavior is not raw session listing. The key behavior is **project attribution**.

Example: a lot of real work happens from a template workspace like `D:\saas-workspace\templates\saas-mvp-template`, but the actual product being discussed is `D:\saas-workspace\products\roompilot-ai`.

Codex Continuity OS can recover that relationship from the transcript and attribute the chat back to `roompilot-ai`, which makes `resume`, `find`, `compare`, and `pack` actually useful.

## Build

This repo currently ships as a source-first CLI.

Clone it:

```powershell
git clone https://github.com/SpeedRanger/codex-continuity-os.git
cd codex-continuity-os
```

On this machine, the default `cargo` shim for the active stable toolchain is broken, so use the intact toolchain directly:

```powershell
$toolchain='C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin'
$env:PATH="$toolchain;" + $env:PATH
cargo.exe build --bin ccx
```

## First Run

Build the local cache:

```powershell
target\debug\ccx.exe index
```

This creates:

- `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`

Normal commands then prefer that cache for speed.

## Quick Demo

```powershell
target\debug\ccx.exe projects
target\debug\ccx.exe resume --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe find "prompt profiles" --repo D:\saas-workspace\products\roompilot-ai
target\debug\ccx.exe compare 019d1f8d-698d-70d1-b07d-f099066d4d34 019d30b1-1b6f-77a3-8c4b-cfcfe2d10973
target\debug\ccx.exe pack --repo D:\saas-workspace\products\roompilot-ai
```

## How It Works

At a high level:

1. scan local Codex rollout files
2. extract normalized session summaries
3. infer the real project repo from workspace + transcript mentions
4. cache the normalized index
5. expose repo-aware commands over that index

Core implementation files:

- [src/main.rs](./src/main.rs): CLI command dispatch and output shaping
- [src/scanner.rs](./src/scanner.rs): archive scanning, parsing, attribution, search, and cache logic
- [src/model.rs](./src/model.rs): normalized session/project/search data model

For a deeper internal walkthrough, see [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md).

## Verification Status

Verified against the real local Codex archive:

- `projects` returned grouped project roots
- `resume --repo roompilot-ai` recovered a template-based `roompilot-ai` session correctly
- `find "prompt profiles" --repo roompilot-ai` recovered the expected historical session
- `compare` correctly inferred later continuation between two `roompilot-ai` chats
- `pack --repo roompilot-ai` produced a usable resume block

Automated checks currently included:

- `9` unit tests passed
- `0` failures

## Current Limits

- cache refresh is explicit via `ccx index`
- file extraction is heuristic, not git-diff-backed
- repo attribution is heuristic, not a full project identity graph
- there is no installer/package yet; current launch shape is source repo + build instructions

## Roadmap

Near-term improvements:

- smarter cache refresh
- better file extraction quality
- stronger resume ranking
- packaged binaries / installer
- richer TUI layer on top of the same core index

## Docs

- [PROJECT_SPEC.md](./PROJECT_SPEC.md): compact product definition
- [CONTINUITY.md](./CONTINUITY.md): current project state
- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md): implementation walkthrough
- [docs/LAUNCH_READINESS.md](./docs/LAUNCH_READINESS.md): launch-facing verification and known limits
- [docs/CODEX_CONTINUITY_OS_MVP.md](./docs/CODEX_CONTINUITY_OS_MVP.md): original MVP spec
- [docs/IMPLEMENTATION_JOURNAL.md](./docs/IMPLEMENTATION_JOURNAL.md): step-by-step build log

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

MIT. See [LICENSE](./LICENSE).
