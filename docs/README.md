# Liminal Flow — Documentation

## Contents

- [Implementation Plan](implementation/liminal_flow_implementation_plan.md) — Detailed technical plan for the v1 implementation
- [Concept Document](implementation/doing_sidecar_v_1_concept.md) — Original design concept for the working-memory sidecar
- [Distribution Strategy](release/distribution-strategy.md) — Linux-first release and packaging plan modelled on the SMDU flow
- [2026-03-13 Command Targeting and Completion Design](ux/2026-03-13-command-targeting-and-completion-design.md) — Dated design snapshot for command targeting, note rendering, and thread completion updates

## Quick Reference

- **CLI binary**: `flo`
- **Product spec**: [SPEC.md](../SPEC.md)
- **Coding standards**: [AGENTS.md](../AGENTS.md)
- **Licence**: [LICENSE](../LICENSE) (MIT)

## Architecture Overview

Liminal Flow is structured as a Rust Cargo workspace with five crates:

| Crate | Role |
|---|---|
| `liminal-flow-core` | Domain types, event system, state reducer, deterministic rules |
| `liminal-flow-store` | SQLite persistence layer — database setup, migrations, repositories |
| `liminal-flow-cli` | Binary entrypoint, clap-based CLI, command handlers |
| `liminal-flow-tui` | Three-pane terminal UI built with ratatui and crossterm |
| `liminal-flow-context` | Git and working-directory context discovery |

## Data Model

- **Threads** — main units of focused work (one active at a time)
- **Branches** — sub-tasks beneath a thread
- **Captures** — raw user input with inferred intent
- **Scopes** — structured context (git repo, branch, cwd) attached to threads/branches
- **Hints** — ambient signals from the environment
- **Events** — append-only audit log of all state mutations
