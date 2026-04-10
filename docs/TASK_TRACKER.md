# Task Tracker

Last updated: 2026-04-10

This is the canonical project task list for Codex Continuity OS.

Status key:

- `now`: active focus
- `next`: queued immediately after current work
- `later`: important, but not current
- `done`: completed and materially verified

## Now

- `[now]` Refine the dashboard UX from usable to obvious
  - Why: the new TUI is the front door, so it must carry more of the product story by itself
  - Success: better hierarchy, clearer selection states, stronger empty states, clearer first-run flow, and a more compact high-signal layout

- `[now]` Maintain a proper product operating system in-repo
  - Why: planning is currently under-documented relative to the product ambition
  - Success: PRD, tracker, user flows, and launch docs stay aligned

## Next

- `[next]` Publish the packaged Windows release artifact on GitHub
- `[next]` Decide whether packaged binaries are enough or whether the next interface leap should be a local web UI
- `[next]` Tighten dashboard empty states and low-width rendering further
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
- `[done]` Deterministic session digest added with summary, verification notes, and next-step extraction
- `[done]` Dashboard detail pane upgraded to expose continuity summary, verification, and extracted next step
- `[done]` Dashboard now explains why the selected session matters with explicit selection reasoning
- `[done]` Dashboard now includes a first-run onboarding/help overlay and persisted dismissal state
- `[done]` Repeatable Windows release packaging path added and verified locally

## Current Recommendation

The best next product move is:

1. tighten packaging and distribution so launch friction drops
2. keep improving dashboard layout and empty-state clarity
3. only then decide whether to leap to a local web UI

That order matters because the core continuity workflow now works end to end. The highest remaining leverage is making the launch path and default experience feel simpler and more premium.
