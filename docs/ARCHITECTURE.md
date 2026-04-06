# Architecture

## Purpose

This document explains how Codex Continuity OS works internally.

The system is intentionally simple:

- one Rust CLI
- one normalized session model
- one scanner/indexing layer
- one cache file
- several operator commands over the same in-memory session list

## Top-Level Flow

1. Read local Codex session archives from `~/.codex/sessions`
2. Parse each rollout JSONL into a normalized `SessionSummary`
3. Infer the real project repo from workspace and transcript mentions
4. Cache the normalized result set to `~/.codex-continuity/cache/session_index.tsv`
5. Serve `sessions`, `projects`, `resume`, `find`, `compare`, and `pack` from that index

## Main Modules

### `src/model.rs`

Defines the in-memory shapes:

- `SessionSummary`
- `ProjectSummary`
- `SearchHit`

### `src/scanner.rs`

Contains the archive and indexing logic:

- session scanning
- line-based JSONL parsing
- repo attribution
- transcript file extraction
- search scoring
- cache read/write

### `src/main.rs`

Contains:

- CLI command definitions
- command dispatch
- output formatting
- compare and pack helpers

## Session Parsing

The parser walks each JSONL file line by line and extracts:

- session id
- session timestamp
- cwd
- first meaningful user message
- last meaningful assistant message
- mentioned repo roots
- mentioned files

The parser is line-based and string-driven rather than serde-heavy because that was the most stable path in the current Windows environment.

## Repo Attribution

This is the most important heuristic in the product.

There are two repo concepts:

- `repo_root`
  - the detected git/workspace root where the session actually ran
- `attributed_repo_root`
  - the repo the session was really about

If the workspace looks indirect, such as:

- `.codex`
- `.agents`
- `.claude`
- `templates`
- `worktrees`

and the transcript mentions a known downstream product repo, the session is attributed to that downstream repo.

This is why sessions run from template workspaces can still resolve correctly to product repos like `roompilot-ai`.

## Search

Search is ranking-based, not exact-id lookup only.

It scores:

- goal text
- assistant outcome text
- attributed repo
- workspace repo
- mentioned repos
- session id

Full-query matches score higher than partial term matches.

The command first tries stricter all-term matching, then falls back to weaker matches if nothing qualifies.

## Compare

`compare` loads two normalized sessions and derives:

- same attributed repo or not
- same workspace repo or not
- a simple relationship guess
- overlapping files
- files unique to each session
- overlapping repo mentions

To keep output useful, compare prefers repo-relevant files and filters obvious global skill/memory noise.

## Pack

`pack` creates a resume artifact for the next Codex session.

It separates:

- latest checkpoint session
- richer context-anchor session

The latest checkpoint is the freshest session for that repo.

The context anchor is chosen from nearby related sessions using a simple usefulness heuristic, so the resume pack does not blindly depend on the latest assistant reply if that reply was procedural noise.

## Cache

The cache lives at:

- `C:\Users\AKR\.codex-continuity\cache\session_index.tsv`

Behavior:

- `ccx index` explicitly rebuilds the cache
- normal commands prefer cache reads
- if no cache exists yet, a command scans the archive and writes a cache

The refresh policy is intentionally explicit for now.

## Design Constraints

- local-first
- read-only around Codex
- no mutation of Codex session/state files
- CLI-first
- small dependency surface

## Known Weak Points

- file extraction is heuristic
- repo attribution is heuristic
- cache freshness is manual
- there is no packaged installer yet
