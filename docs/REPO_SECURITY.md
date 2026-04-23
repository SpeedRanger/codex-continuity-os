# Public Repo Security

Last updated: 2026-04-23

This document records the public GitHub repo posture for Codex Continuity OS.

Public repo:

- `https://github.com/SpeedRanger/codex-continuity-os`

## Goal

Maintain the repository as a credible public product showcase:

- safe for public browsing
- clear for contributors
- protected against accidental unsafe merges
- monitored for dependency and code-scanning issues
- minimal in automation and dependency surface

## Current GitHub Controls

`main` is protected with:

- required status checks
- strict branch freshness before merge
- required checks:
  - `test`
  - `Analyze (actions)`
  - `Analyze (rust)`
- one approving review required
- code-owner review required
- stale reviews dismissed on new pushes
- conversation resolution required
- admins enforced
- force-push disabled
- branch deletion disabled

Repository security features:

- secret scanning enabled
- push protection enabled
- private vulnerability reporting enabled
- Dependabot security updates enabled
- CodeQL enabled for GitHub Actions and Rust
- Dependabot weekly checks for Cargo and GitHub Actions

Repository hygiene:

- `CODEOWNERS` owns the full repository
- `SECURITY.md` defines private vulnerability reporting expectations
- issue templates guide public bug and feature reports
- PR template includes verification and security checklist items
- release artifacts include a checksum file

## Current Release Posture

Latest release:

- `v0.1.2`

Release assets:

- `ccx-windows-x86_64-v0.1.2.zip`
- `ccx-windows-x86_64-v0.1.2.sha256.txt`

Security-relevant release notes:

- `lru` advisory fixed by upgrading through `ratatui 0.30.0`
- unused `ratatui` default features disabled
- optional `rand` dependency removed from the lockfile dependency graph

## Maintenance Checklist

Before merging product changes:

- CI must pass
- CodeQL must pass
- public docs must stay accurate
- release notes must mention security-relevant dependency changes
- new dependencies must be justified
- no generated archives, secrets, local Codex archives, or build artifacts should be committed

Before publishing a release:

- run `cargo fmt --check`
- run `cargo test`
- run `cargo tree --target all -i rand` when dependency alerts mention `rand`
- run `cargo tree -i <dependency>` for any dependency alert
- build release package with `scripts/package-release.ps1`
- expand the zip and run `ccx.exe --version`
- upload zip plus SHA256 checksum

## Known Non-Maximal Controls

These are intentional current tradeoffs:

- commit signing is not required yet
- GitHub Actions are version-tagged, not SHA-pinned
- there is no cross-platform installer yet

These are acceptable for the current single-maintainer public launch, but should be revisited if the repo gets external contributors.
