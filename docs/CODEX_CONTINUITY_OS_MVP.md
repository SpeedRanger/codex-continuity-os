# Codex Continuity OS MVP

Last updated: 2026-04-03

## One-Line Promise

Open any repo and instantly reconstruct what happened across Codex chats, why it changed, what files mattered, and what to do next.

## Positioning

- Category: local-first developer continuity layer for coding agents
- Initial wedge: Codex power users who juggle many chats, repos, branches, and forks
- Primary job: orientation after interruption
- Anti-goal: do not become a Codex replacement, IDE, or agent runner

## Why This Exists

Current session menus are too shallow. They help you pick a session id, but they do not answer the questions that matter in practice:

- what were we doing in this repo?
- what changed in the last few chats?
- which chat mattered versus which chat was noise?
- what is different between this fork and that chat?
- what is the best prompt/context pack to resume work now?

Search alone is not enough. The product must provide narrative continuity, project grouping, and next-step recovery.

## Product Shape

Recommendation: CLI/TUI-first sidecar.

Why this shape wins:

- matches existing Codex workflow instead of fighting it
- stays fast and lightweight
- works offline
- avoids deep UI coupling to Codex internals
- can later grow a web UI without changing the core

Rejected initial shape: desktop or browser-first workspace shell.

Reason for rejection:

- more surface area
- slower to ship
- easier to turn into a bloated alternate IDE
- higher break risk as Codex evolves

## ICP

### Primary

- solo developers and founders using Codex heavily
- developers switching between many repos and branches daily
- AI-heavy operators who lose context after breaks

### Secondary

- teams using multiple coding agents
- consultants managing many client repos

## Core Workflows

### Workflow 1: Resume This Repo

User opens a repo and runs:

```bash
ccx resume
```

The tool should show:

- active and recent Codex chats tied to this repo
- best current resume candidate
- last meaningful goal
- last meaningful change
- key files mentioned or touched
- open blocker or pending decision
- next recommended action
- optional generated resume pack for the next Codex prompt

### Workflow 2: Find The Chat

User runs:

```bash
ccx find "prompt profile"
ccx find "FastAPI migration"
```

The tool should return:

- matching chats
- repo and branch
- time
- short explanation of why the result matched
- top files and decisions from that chat

### Workflow 3: Compare Two Chats

User runs:

```bash
ccx compare <session-a> <session-b>
```

The tool should compare:

- repo and cwd
- branch and git SHA
- overlapping and differing file sets
- user intent
- major decisions
- likely fork point if inferable
- outcome summary

### Workflow 4: Build A Resume Pack

User runs:

```bash
ccx pack
ccx pack --session <id>
```

The tool should produce a compact context block for the next Codex session:

- current goal
- what is true now
- what changed recently
- files that matter
- next step
- unresolved questions

## V1 Commands

### Must Have

- `ccx resume`
- `ccx find <query>`
- `ccx compare <session-a> <session-b>`
- `ccx pack`
- `ccx sessions`
- `ccx projects`
- `ccx index`

### Useful Flags

- `--json`
- `--repo <path>`
- `--since <date>`
- `--limit <n>`
- `--session <id>`
- `--branch <name>`
- `--open`

### Explicit Non-Goals For V1

- no cloud sync
- no auth
- no team collaboration
- no write access to Codex session/state files
- no dependence on Codex private APIs
- no browser UI as primary surface

## UX Rules

- default output must be readable in a terminal in under 10 seconds
- every command should degrade to plain text cleanly
- no noisy dashboards by default
- prefer one best answer over huge lists
- always show the repo path and session id clearly
- make uncertain inferences explicit

## Data Sources

### Required

- `~/.codex/sessions/**/*.jsonl`
- `~/.codex/history.jsonl`
- local git metadata for relevant repos

### Optional

- `CONTINUITY.md`
- `STATE_OF_PLAY.md`
- `SESSION_MAP.md`
- `docs/DELIVERY_REPORT.md`
- `~/.codex/memories/SESSION_LEDGER.md`
- `~/.codex/memories/CODEX_MEMORY_LOG.md`

The tool must remain useful even when optional files do not exist.

## Core Data Model

### Session

```text
id
started_at
last_updated_at
cwd
repo_root
branch
head_sha
title_guess
user_goal
assistant_outcome
status
message_count
summary
next_step
confidence
```

### Project

```text
project_id
repo_root
name
active_session_ids
recent_session_ids
current_branch
last_head_sha
current_summary
current_next_step
last_meaningful_change
```

### Event

```text
session_id
timestamp
kind
text
role
```

### Artifact Link

```text
session_id
artifact_type
path
reason
confidence
```

### Comparison Record

```text
session_a
session_b
same_repo
same_branch
common_files
only_in_a
only_in_b
shared_topics
key_differences
```

## Storage Layout

Use a dedicated local store owned by the tool.

```text
~/.codex-continuity/
  index.db
  cache/
    sessions/
    projects/
    packs/
  logs/
  config.toml
  adapters/
    codex/
      versions/
```

Why not store in `~/.codex`:

- cleaner ownership boundary
- avoids accidental interference with Codex internals
- easier to delete or rebuild
- safer against future Codex changes

## Architecture

### Principle

Read-only around Codex. Never patch or mutate Codex state.

### Components

1. `scanner`
   Walks session files and history files incrementally.
2. `parser`
   Converts raw Codex artifacts into normalized events.
3. `enricher`
   Adds repo, branch, SHA, file mentions, and artifact references.
4. `summarizer`
   Produces deterministic compact summaries and next-step guesses.
5. `indexer`
   Stores normalized rows plus full-text search material.
6. `query engine`
   Powers resume, find, compare, and pack.
7. `renderer`
   Outputs terminal text, TUI views, and optional JSON.

## Compatibility Strategy

This is the load-bearing constraint.

- do not depend on undocumented Codex RPCs
- do not inject into Codex UI
- do not rewrite `sessions/`, `history.jsonl`, or `state_5.sqlite`
- treat Codex artifacts as external inputs
- version the parser adapters
- tolerate unknown fields
- fail open when partial parsing still supports a reduced answer
- keep a fixture suite of real archived sessions from multiple dates

The product should still work after Codex updates unless the on-disk formats change materially. If that happens, only the adapter layer should need updates.

## Performance Targets

- cold `ccx resume` in a known repo: under 2 seconds
- warm query latency: under 150 milliseconds
- incremental index refresh after one finished session: under 500 milliseconds
- memory footprint for typical usage: under 150 MB

## Reliability Targets

- deterministic indexing from the same input set
- resumable scans
- no data loss if indexing is interrupted
- corrupted session file should degrade one session, not the whole index
- every user-visible summary should carry a confidence hint when partly inferred

## Technical Stack

### Recommendation

- language: Rust
- local database: SQLite
- search: SQLite FTS5 first
- TUI: Ratatui
- config: TOML

### Why

- fast startup
- low memory
- single static binary distribution
- good fit for terminal-heavy users
- safer long-term than a large Electron shell

### Acceptable Alternative

- Go instead of Rust

Why Rust still wins:

- stronger parsing and state-machine ergonomics
- better odds of building a fast single-binary TUI without runtime drag

## 7-Day MVP

### Day 1

- set up CLI binary
- define normalized schema
- parse session list and basic metadata

### Day 2

- map sessions to repo roots
- attach git branch and head SHA
- implement `ccx sessions` and `ccx projects`

### Day 3

- add summary extraction
- identify user goal, assistant outcome, next-step guess
- implement `ccx resume`

### Day 4

- add FTS indexing
- implement `ccx find`

### Day 5

- implement `ccx compare`
- compare file mentions, cwd, branch, and summary deltas

### Day 6

- implement `ccx pack`
- tighten terminal rendering
- test on real session archives

### Day 7

- ship one polished TUI screen for `resume`
- write install docs
- create a demo using real archived sessions

## V1 Success Metric

The product is successful if a heavy Codex user can return after a break and, within 30 seconds, recover the correct repo context and next step without manually hunting through raw session ids.

## Distribution

### Initial

- GitHub repo
- single binary release
- short terminal demo GIF
- launch into Codex-heavy communities first

### Messaging

Do not sell "session search."

Sell:

- resume any repo instantly
- recover context after interruptions
- compare forks and chats cleanly
- stop losing work inside agent history

## Pricing Hypothesis

### Start Free

- local CLI and TUI free

### Paid Later

- shared team memory
- sync across machines
- cloud backup
- GitHub issue and PR linkage
- Slack or Linear integrations

## Risks

### Risk 1: Codex format drift

Mitigation:

- adapter boundary
- parser fixtures
- fail-open design

### Risk 2: summaries hallucinate

Mitigation:

- bias toward extraction over generation
- show confidence
- anchor summaries to explicit evidence

### Risk 3: too broad too early

Mitigation:

- keep V1 focused on repo resume
- no cloud
- no teams
- no browser-first UI

## What Makes This Hard To Copy

- repo-aware clustering
- reliable resume-pack generation
- useful comparisons between sessions and forks
- extraction quality from messy real session logs

The moat is not just indexing text. The moat is converting fragmented agent history into operational continuity.

## Technical Implementation Plan

1. Build a read-only session scanner and normalized schema.
2. Get repo grouping and git enrichment working.
3. Ship `resume` before anything flashy.
4. Add search only after the summary layer is useful.
5. Add comparison once session metadata is trustworthy.
6. Add a small TUI only after the plain-text UX is already strong.

## Intuition-Building Plan

Think of this as `git blame` plus `ripgrep` plus `session memory`, but for agent work.

- `scanner` answers: what raw evidence exists?
- `summarizer` answers: what happened here?
- `project view` answers: what matters right now?
- `pack` answers: what should I tell the next chat?

If V1 nails those four answers, the product has real pull.

## Recommended Next Step

Start implementation with one command and one promise:

```bash
ccx resume
```

If `ccx resume` becomes genuinely useful, the rest of the product will follow naturally.
