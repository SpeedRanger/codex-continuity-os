# Product Requirements Document

Last updated: 2026-04-10

## Product

Codex Continuity OS

## Product Thesis

Heavy Codex users lose more time to orientation failure than to raw coding speed.

The product wins if it can answer, quickly and reliably:

- what were we doing in this repo?
- which chat actually matters?
- what changed recently?
- what should the next session know?

This is not a session search utility. It is a continuity layer over Codex history.

## Problem

Codex can reopen a session id, but that is not the same as restoring project context.

When users switch across many repos, templates, branches, and parallel chats, they lose:

- repo-level orientation
- decision memory
- file-level continuity
- next-step clarity

The current default history surfaces are too shallow for that workflow.

## Target User

### Primary

- solo founders and operators using Codex heavily
- developers switching across many repos and chats every day
- AI-heavy builders who return after breaks and cannot reconstruct context quickly

### Secondary

- consultants juggling many client repos
- teams experimenting with multiple coding agents

## Jobs To Be Done

### Functional

- recover the right repo context after interruption
- find the chat where a topic was discussed
- compare two chats in the same project arc
- build a compact resume handoff for the next session

### Emotional

- reduce the panic of "I know we solved this somewhere"
- reduce the feeling that agent work disappears into a black box
- increase confidence before making the next change

## Product Shape

Current recommended shape:

- local-first
- read-only around Codex
- Rust core
- CLI/TUI-first

Current front door:

- `ccx dashboard`

Later expansion path:

- optional local web UI on top of the same engine

## Core User Promise

Open any repo and, within 30 seconds, understand:

- the most relevant recent session
- the project goal
- the latest meaningful outcome
- the files that mattered
- the best next action

## Current Scope

### Shipped

- archive scanning over `~/.codex/sessions`
- repo attribution across indirect workspaces
- cache-backed session index
- `projects`
- `resume`
- `find`
- `compare`
- `pack`
- interactive dashboard TUI

### In Scope Next

- better session summarization across the whole transcript
- stronger product polish in the dashboard
- canonical product ops docs and task management
- packaged distribution

### Explicit Non-Goals

- patching or mutating Codex internals
- cloud sync
- team collaboration
- browser-first IDE shell
- becoming a Codex replacement

## UX Principles

- the default entrypoint should feel like one product, not many disconnected commands
- the best answer matters more than the longest list
- every inference should stay inspectable
- terminal output must be readable quickly under interruption stress
- the product should feel fast after indexing

## Key Workflows

### Workflow 1: Resume This Repo

User opens:

```powershell
ccx dashboard --repo <path>
```

Success means the user can identify:

- which session is the best checkpoint
- what the goal was
- what happened last
- what to do next

### Workflow 2: Find The Topic

User searches:

```powershell
ccx find "prompt profiles" --repo <path>
```

Success means the user finds the correct historical session without remembering its id.

### Workflow 3: Compare Two Chats

User compares:

```powershell
ccx compare <a> <b>
```

Success means the user understands whether the two sessions are part of the same continuity chain and what changed between them.

### Workflow 4: Create Resume Pack

User runs:

```powershell
ccx pack --repo <path>
```

Success means the next Codex session can start with a compact, useful context block.

## Functional Requirements

### Session Understanding

- parse session metadata reliably
- extract meaningful user goal
- extract meaningful assistant outcome
- identify repo mentions and file mentions
- attribute sessions to the correct downstream project when possible

### Query Layer

- list project clusters
- list relevant sessions per project
- support repo-scoped search
- support pairwise comparison
- support resume-pack generation

### Interface Layer

- provide one preferred entrypoint
- support keyboard-first dashboard navigation
- surface key context without forcing users to run many separate commands

## Quality Bar

### Performance

- hot-path commands should feel near-instant after cache exists
- cold indexing can be slower, but must remain trustworthy and bounded

### Reliability

- no mutation of Codex session/state files
- graceful handling of missing or malformed session files
- deterministic results from the same archive input

### Product Quality

- product must be understandable without reverse-engineering the source
- docs must support shipping, planning, and execution
- the dashboard must feel like a coherent front door, not just a screenful of text

## Known Weak Points

- summarization is currently heuristic, not true whole-chat summarization
- file extraction is heuristic
- repo attribution is heuristic
- packaging/install remains rough
- current dashboard is strong first cut, not final polish

## Success Metrics

### Primary

- user can recover the correct repo context and next step within 30 seconds

### Secondary

- user can find the correct historical session by topic without manual archive hunting
- user prefers the dashboard over running multiple raw commands

## Launch Criteria

For the next meaningful public push, the product should have:

- reliable dashboard-first workflow
- stronger summarization quality
- canonical product docs
- cleaner install/build path
- a product demo that makes the value obvious in under one minute

## Open Questions

- should the next interface leap be a richer TUI or a local web UI?
- how much deterministic summarization can be improved before introducing optional model-assisted summarization?
- should the task tracker live only in-repo or also mirror to GitHub issues?
