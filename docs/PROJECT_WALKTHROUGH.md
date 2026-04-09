# Project Walkthrough

Last updated: 2026-04-09

## Purpose

This document explains the entire product in operator terms:

- what Codex Continuity OS is
- how it works internally
- what each command does
- how to use it in a real repo recovery workflow

This is the fastest "read one file and understand the product" document in the repo.

## What The Product Is

Codex Continuity OS is a local-first continuity layer for Codex.

It does not replace Codex and it does not modify Codex internals. It reads your local Codex session archive, builds a project-aware index, and helps you answer questions that `codex resume` alone does not answer well:

- what were we doing in this repo?
- which historical chat actually matters?
- where did we discuss this feature?
- how are these two chats related?
- what context should the next session start with?

The product exists because session ids are not enough. Real continuity requires project attribution, search, comparison, and a compact way to resume work after a break.

## The Core Mental Model

Think of the app as:

- session parser
- repo attribution layer
- local cache
- continuity commands over that normalized history

Or more simply:

- git-aware memory for Codex chats

The product takes raw rollout archives from `~/.codex/sessions`, converts them into structured session summaries, groups them by real project, and then exposes a few high-value commands over that dataset.

## How The System Works

The app has three main modules:

- [src/model.rs](/D:/saas-workspace/products/codex-continuity-os/src/model.rs)
- [src/scanner.rs](/D:/saas-workspace/products/codex-continuity-os/src/scanner.rs)
- [src/main.rs](/D:/saas-workspace/products/codex-continuity-os/src/main.rs)

### 1. Session ingestion

The scanner reads rollout archives from:

- `C:\Users\AKR\.codex\sessions`

Each JSONL file is parsed into a normalized session record. The parser extracts:

- session id
- session start timestamp
- working directory
- detected workspace repo root
- first meaningful user goal
- last meaningful assistant outcome
- mentioned repo roots
- mentioned file paths

The parser is line-based and string-driven rather than serde-heavy because that was the most stable path in the current Windows environment.

### 2. Repo attribution

This is the most important behavior in the product.

There are two different repo concepts:

- `repo_root`
  - where the session actually ran
- `attributed_repo_root`
  - which project repo the session was really about

This matters because many Codex chats happen from:

- template workspaces
- worktrees
- `.codex`-style directories
- intermediate staging repos

If the transcript clearly points to a downstream product repo, the app attributes the session to that downstream repo instead of blindly trusting `cwd`.

That is why sessions from:

- `D:\saas-workspace\templates\saas-mvp-template`

can still be recovered correctly under:

- `D:\saas-workspace\products\roompilot-ai`

### 3. Cache

Normalized sessions are cached at:

- `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`

This keeps repeat commands fast. The refresh model is explicit:

- `ccx index` rebuilds the cache
- normal commands prefer cache reads

That keeps the system small and predictable.

### 4. Command layer

The CLI then serves operator commands over that normalized session list:

- `dashboard`
- `sessions`
- `projects`
- `resume`
- `find`
- `compare`
- `pack`
- `index`

## The Data Model

Each normalized session record currently includes:

- `id`
- `started_at`
- `cwd`
- `repo_root`
- `attributed_repo_root`
- `mentioned_repo_roots`
- `mentioned_files`
- `first_user_goal`
- `last_assistant_outcome`

That is enough structure to support:

- project grouping
- repo resume
- ranked search
- side-by-side comparison
- resume-pack generation

## What Each Command Does

### `ccx dashboard [--repo <path>]`

Opens the interactive continuity board.

This is now the preferred front door for the product.

It gives you:

- project list
- session list for the selected project
- rich detail pane for the selected session
- live search inside the selected project's sessions
- a next-step hint panel

Keyboard flow:

- `Tab` switches between projects and sessions
- `/` opens search mode
- `Esc` clears search
- `i` rebuilds the cache/index
- `q` quits

### `ccx sessions`

Lists recent normalized sessions.

Use this when you want the raw historical view: which sessions exist, when they happened, and which project each one was attributed to.

### `ccx projects`

Groups sessions by attributed repo root.

Use this when you want the project-level view instead of the chat-level view.

### `ccx resume --repo <path>`

Finds the best known session for a repo.

Use this first when coming back to a project after a break.

It is the main answer to:

- what were we doing here?

### `ccx find "<query>" [--repo <path>]`

Runs ranked search across normalized session history.

Use this when you remember the topic but not the session id.

The scoring looks at:

- goal text
- assistant outcome text
- repo evidence
- session id

### `ccx compare <session-a> <session-b>`

Compares two sessions side by side.

Use this when you need to understand how two chats relate, what files overlap, and whether one looks like the later continuation of the other.

### `ccx pack [--repo <path>] [--session <id>]`

Generates a compact resume artifact for the next Codex session.

Use this when you are about to start a fresh chat and want to carry forward the best context with minimal noise.

### `ccx index`

Rebuilds the local cache.

Use this after new sessions accumulate or when you want a clean refresh from the live archive.

## Recommended Operator Workflow

When you come back to a repo after a break, the best order is:

1. `ccx dashboard --repo <path>`
2. use the dashboard to inspect the latest session and related sessions
3. use `/` search inside that repo if you need a specific discussion
4. use `ccx compare <a> <b>` if two chats still look similar
5. use `ccx pack --repo <path>` before starting the next session

That workflow goes from broad recovery to precise recovery to handoff.

## Real Walkthrough: `roompilot-ai`

The examples below were verified against a fresh build of the current source using:

- `C:\Users\AKR\.codex\tmp\ccx-target-live\debug\ccx.exe`

That separate target path was used because `target\debug\ccx.exe` can be stale or locked on this Windows machine during active iteration.

### Step 1: Recover the repo

Command:

```powershell
ccx resume --repo D:\saas-workspace\products\roompilot-ai
```

What happened in the verified run:

- it matched `roompilot-ai` even though the session actually ran from `D:\saas-workspace\templates\saas-mvp-template`
- it selected session `019d30b1-1b6f-77a3-8c4b-cfcfe2d10973`
- it reported that there are `2` related sessions in that repo

Why that matters:

- this is the product's core value
- the continuity layer recovered the real project despite the indirect workspace

### Step 2: Find a specific topic

Command:

```powershell
ccx find "prompt profiles" --repo D:\saas-workspace\products\roompilot-ai
```

Verified result:

- it returned one result
- the winning session was `019d1f8d-698d-70d1-b07d-f099066d4d34`
- the match reason was that the assistant outcome matched both query terms

Why that matters:

- this is how you recover a discussion by topic instead of by session id

### Step 3: Compare two related sessions

Command:

```powershell
ccx compare 019d1f8d-698d-70d1-b07d-f099066d4d34 019d30b1-1b6f-77a3-8c4b-cfcfe2d10973
```

Verified result:

- relation: `same attributed repo; session_b looks like the later continuation`
- `same_attributed_repo: true`
- `same_workspace_repo: true`
- common repo mention: `D:\saas-workspace\products\roompilot-ai`
- overlapping files included:
  - `D:/saas-workspace/products/roompilot-ai/AGENTS.md`
  - `D:/saas-workspace/products/roompilot-ai/ARCHITECTURE.md`
  - `D:/saas-workspace/products/roompilot-ai/App.tsx`
  - `D:/saas-workspace/products/roompilot-ai/CONTINUITY.md`

Why that matters:

- compare does not just show two ids
- it tells you whether they are really part of the same continuity chain

### Step 4: Generate a resume pack

Command:

```powershell
ccx pack --repo D:\saas-workspace\products\roompilot-ai
```

Verified result:

- latest session: `019d30b1-1b6f-77a3-8c4b-cfcfe2d10973`
- context anchor session: `019d1f8d-698d-70d1-b07d-f099066d4d34`
- related sessions: `2`
- generated a `BEGIN_CCX_RESUME_PACK` block
- included concrete files that mattered, such as:
  - `D:/saas-workspace/products/roompilot-ai/backend/app/api/routes/ai.py`
  - `D:/saas-workspace/products/roompilot-ai/backend/app/core/config.py`
  - `D:/saas-workspace/products/roompilot-ai/backend/app/services/gemini_service.py`
  - `D:/saas-workspace/products/roompilot-ai/backend/app/services/prompts.py`
  - `D:/saas-workspace/products/roompilot-ai/frontend/index.html`

Why that matters:

- this is the fastest bridge from old chats to the next productive session

## What Is Finished Today

The current product includes:

- real archive scanning
- normalized session summaries
- heuristic repo attribution
- project grouping
- repo-aware resume
- ranked search
- side-by-side compare
- resume-pack generation
- cache-backed reads
- unit tests for core heuristics
- public GitHub repo and launch docs
- CI, CodeQL, Dependabot, `SECURITY.md`, and `CODEOWNERS`

## Current Limits

The honest limitations are:

- cache refresh is explicit through `ccx index`
- repo attribution is heuristic, not a full identity graph
- file extraction is heuristic, not git-diff-backed
- there is no packaged installer yet
- there is no TUI or GUI yet

So the correct launch framing is:

- strong source-first CLI continuity product

Not:

- perfect automatic project intelligence
- polished desktop product
- real-time multi-session live sync

## The Fastest Way To Think About It

Codex Continuity OS is a project-oriented memory layer over local Codex session history.

It turns:

- raw chats

into:

- repo-aware operational context

And it gives you five useful actions over that context:

- list
- group
- resume
- search
- compare
- hand off
