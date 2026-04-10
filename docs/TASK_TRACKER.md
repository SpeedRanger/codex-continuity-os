# Task Tracker

Last updated: 2026-04-10

This is the canonical project task list for Codex Continuity OS.

Status key:

- `now`: active focus
- `next`: queued immediately after current work
- `later`: important, but not current
- `done`: completed and materially verified

## Now

- `[now]` Improve session summarization beyond first-user / last-assistant heuristics
  - Why: this is the biggest current product weakness
  - Success: summary fields capture objective, key decisions, relevant files, verification, and next step from the full conversation

- `[now]` Refine the dashboard UX from “usable” to “obvious”
  - Why: the new TUI is the front door, so it must carry more of the product story by itself
  - Success: better hierarchy, clearer selection states, stronger empty states, more legible context pane

- `[now]` Maintain a proper product operating system in-repo
  - Why: planning is currently under-documented relative to the product ambition
  - Success: PRD, tracker, user flows, and launch docs stay aligned

## Next

- `[next]` Add a session summary model richer than `first_user_goal` and `last_assistant_outcome`
- `[next]` Show explicit “why this session was selected” reasoning in the dashboard detail pane
- `[next]` Add a proper first-run onboarding view for `dashboard`
- `[next]` Add installation/distribution docs for a cleaner launch story
- `[next]` Decide whether the next interface leap is richer TUI or local web UI

## Later

- `[later]` Improve file extraction quality with stronger filtering and evidence ranking
- `[later]` Add richer compare output and continuity deltas
- `[later]` Add structured JSON output for more commands
- `[later]` Add packaged binaries / installer
- `[later]` Add small local web companion if the TUI proves the workflow
- `[later]` Add optional GitHub issue / PR linking

## Done

- `[done]` Dedicated product repo created outside `~/.codex`
- `[done]` Real archive scanner implemented
- `[done]` Repo attribution for indirect workspaces implemented
- `[done]` `resume`, `find`, `compare`, and `pack` implemented
- `[done]` Cache-backed session index implemented
- `[done]` Launch docs and implementation journal added
- `[done]` Public GitHub repo created and secured
- `[done]` Visual live demo walkthrough added
- `[done]` First continuity dashboard TUI added and verified on real archive data

## Current Recommendation

The best next product move is:

1. strengthen summarization
2. tighten dashboard polish around that stronger summary layer
3. only then decide whether to leap to a local web UI

That order matters because better interface polish on weak summaries still produces a shallow product.
