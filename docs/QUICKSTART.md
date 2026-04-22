# Quickstart

Last updated: 2026-04-22

This is the shortest path from "I have Codex history on this machine" to "I can recover project context."

## Install From Source On Windows

Build the binary:

```powershell
$toolchain='C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin'
$env:PATH="$toolchain;" + $env:PATH
cargo.exe build --release --bin ccx
```

Install it into the local continuity home:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-windows.ps1 -AddToUserPath
```

Open a new terminal after changing PATH.

## First Run

Check that Codex history and the CCX cache are wired correctly:

```powershell
ccx doctor
```

Open the dashboard:

```powershell
ccx dashboard
```

If you are already inside a repo, this is usually enough. To focus a specific repo:

```powershell
ccx dashboard --repo D:\path\to\your\repo
```

## Daily Workflow

Use this when returning to a project:

```powershell
ccx dashboard --repo D:\path\to\your\repo
```

Use this when starting a fresh Codex session:

```powershell
ccx pack --repo D:\path\to\your\repo
```

Use this when you remember a topic, not a chat id:

```powershell
ccx find "prompt profiles" --repo D:\path\to\your\repo
```

## Dashboard Keys

- `?` opens help
- `Tab` switches between projects and sessions
- `j` / `k` moves selection
- `/` searches inside the selected project
- `Esc` clears search or closes overlays
- `i` manually rebuilds the index
- `q` quits

## Cache Behavior

Normal commands now check whether the Codex archive changed. If the local cache is missing or stale, CCX refreshes it automatically.

CCX allows a short active-session timestamp grace window so Codex writing to the current transcript does not make every command rebuild the cache repeatedly.

You can still rebuild explicitly:

```powershell
ccx index
```

Use `ccx doctor` any time you want to inspect the cache path, freshness, archive count, and suggested dashboard command.
