# Project Spec

Last updated: 2026-04-03

## Product

Codex Continuity OS

## One-Line Promise

Open any repo and instantly reconstruct what happened across Codex chats, why it changed, what files mattered, and what to do next.

## Primary User

- heavy Codex users
- solo builders switching across many repos and chats
- AI-heavy developers who lose context after interruptions

## Problem

Codex session history is good enough to resume a raw session id, but weak at answering:

- what were we doing in this repo?
- which chat actually matters?
- what changed across chats or forks?
- what should the next Codex session know?

## Recommended Product Shape

- CLI/TUI-first sidecar
- local-first
- read-only around Codex

## Non-Goals

- do not replace Codex
- do not patch Codex internals
- do not build cloud sync or team features in v1
- do not start with a browser-first workspace shell

## V1 Must-Haves

- `ccx resume`
- `ccx find`
- `ccx compare`
- `ccx pack`

## Success Metric

A heavy Codex user should be able to return after a break and recover the correct repo context and next step in under 30 seconds.
