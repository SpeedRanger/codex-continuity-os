# Security Policy

## Supported Scope

This project is currently supported as the latest `main` branch on GitHub:

- `https://github.com/SpeedRanger/codex-continuity-os`

## Reporting A Vulnerability

If you find a security issue:

1. Do not open a public issue with exploit details.
2. Report it privately through GitHub private vulnerability reporting.
3. If private reporting is unavailable, contact the maintainer directly before disclosing details publicly.

Please include:

- a short description of the issue
- impact
- affected file(s) or command(s)
- reproduction steps
- suggested mitigation if you have one

## Security Posture

This project is intentionally:

- local-first
- read-only around Codex session/state files
- small in dependency surface

It should not:

- mutate Codex rollout/session/state artifacts
- require cloud credentials to function
- store secrets in the repository

## Current Priorities

Security-sensitive areas in this repo include:

- transcript parsing
- cache serialization
- file/path inference
- GitHub Actions workflows

Current focus is on:

- avoiding secret leakage
- keeping the repo dependency surface small
- keeping automation minimal and auditable

## Repository Controls

The public repository is maintained with:

- protected `main`
- required CI and CodeQL checks
- code-owner review requirement
- stale review dismissal
- required conversation resolution
- secret scanning and push protection
- private vulnerability reporting
- Dependabot security updates

For the full public repo posture, see [docs/REPO_SECURITY.md](./docs/REPO_SECURITY.md).
