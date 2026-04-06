# Contributing

## Scope

Codex Continuity OS is intentionally small.

Contributions should preserve the core product shape:

- local-first
- read-only around Codex
- fast CLI-first workflow
- no patching of Codex internals

## Setup

Use the intact Rust toolchain directly on this machine:

```powershell
$toolchain='C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin'
$env:PATH="$toolchain;" + $env:PATH
cargo.exe build --bin ccx
```

## Test

```powershell
cargo.exe test
```

## Local Usage

Rebuild the session cache:

```powershell
target\debug\ccx.exe index
```

Then exercise the main commands:

```powershell
target\debug\ccx.exe projects
target\debug\ccx.exe resume --repo <path>
target\debug\ccx.exe find "<query>"
target\debug\ccx.exe compare <session-a> <session-b>
target\debug\ccx.exe pack --repo <path>
```

## Contribution Priorities

High-value improvements:

- better repo attribution
- faster indexing and refresh behavior
- cleaner file extraction
- stronger resume ranking
- packaging and installation improvements
- better terminal UX without bloating the core

Avoid introducing:

- write access to Codex session/state files
- cloud sync in the core path
- dependence on private Codex APIs
- large framework overhead for simple CLI concerns
