# Command Targeting and Completion Design

Date: March 13, 2026

## Purpose

Capture design decisions behind the current bug-review work before implementation begins.

This document is the implementation-facing snapshot for:

- [Issue #25](https://github.com/liminal-hq/flow/issues/25): local-time note rendering in the TUI
- [Issue #26](https://github.com/liminal-hq/flow/issues/26): consistent slash-command targeting in the TUI
- [Issue #28](https://github.com/liminal-hq/flow/issues/28): cascading branch completion when a thread is marked done

## Scope

### TUI-only

- Local-time rendering for note timestamps in the Status pane
- Selection-aware slash-command targeting in Insert mode

### TUI and CLI

- Thread completion should also mark non-archived child branches done
- Related docs should describe the same lifecycle behaviour across both interfaces

## Problem Summary

The current implementation mixes several targeting models:

- Normal mode actions mostly operate on the selected item
- Insert-mode slash commands are split between selected-item and active-item behaviour
- Capture still follows active focus
- `/back` is neither a pure selected-item command nor a simple active-item mutation

That makes the command model harder to predict than it needs to be.

At the same time, thread completion is currently incomplete from a lifecycle point of view: marking a thread done can leave parked child branches behind in a dangling state.

## Design Goals

- Make command targeting predictable in the TUI
- Keep selection, active focus, and capture targeting conceptually distinct
- Align Insert mode with Normal mode where commands act on existing items
- Preserve active-context creation flows such as starting a thread or branch
- Keep CLI lifecycle rules aligned with TUI lifecycle rules where the concepts overlap
- Fix the note timestamp bug without changing the stored UTC model

## Targeting Model

The TUI should use three command-targeting categories.

### 1. Selected-item commands

These commands operate on an existing thread or branch chosen in the thread list.

- `/resume [note]`
- `/pause [note]`
- `/park [note]`
- `/done [note]`
- `/archive [note]`
- `/note <note>`

Rules:

- They should target the currently selected thread or branch
- They should not require the selected item to already be active
- `/note <note>` should attach to the selected item without changing active focus
- Where a trailing note is supported, it should attach to the same selected target as the command action

### 2. Active-context commands

These commands operate on the current active working context rather than the selected row.

- `/now <thread>`
- `/branch <branch>`
- plain-text capture
- `/where`

Rules:

- They should use the current active thread or active branch context
- They should keep behaving consistently with the CLI mental model
- Plain text remains shorthand for “note on the current active capture target”

### 3. Active-focus-stack commands

These commands are defined by the active navigation stack rather than simple selected-item targeting.

- `/back`
- `/back [note]`

Rules:

- `/back` should mean “return from the current active branch context to the parent thread”
- It should continue to operate on the current active focus stack, not on the selected row
- Any trailing note should attach to the same context affected by the `/back` transition

## Command Matrix

| Command | Current TUI behaviour | Proposed rule |
|---|---|---|
| `/resume [note]` | Selected item, optional trailing note | Selected item, optional trailing note |
| `/park [note]` | Active branch, optional trailing note | Selected item, optional trailing note |
| `/done [note]` | Active item, optional trailing note | Selected item, optional trailing note |
| `/archive [note]` | Active item, optional trailing note | Selected item, optional trailing note |
| `/pause [note]` | Active thread, optional trailing note | Selected item, optional trailing note |
| `/note <note>` | Active capture target | Selected item, attaching trailing text without changing active context |
| `/now <thread>` | Active-context command | Active-context command |
| `/branch <branch>` | Active thread | Active-context command |
| `/back` | Returns from the current active branch context to the parent thread | Active-focus-stack command |
| `/back [note]` | Returns from the current active branch context to the parent thread, with optional trailing note | Active-focus-stack command |
| Plain text | Active capture target | Active-context command |
| `/where` | Active thread query | Active-context command |

## Lifecycle Decisions

### Note timestamps

- Note timestamps should continue to be stored in UTC
- The TUI should render them in the local time of the current machine
- This change is presentation-only

### Thread completion

- Marking a thread done should also mark all of its non-archived child branches done
- This rule should apply in both TUI and CLI flows
- The goal is to avoid dangling branches beneath a completed parent thread

### Note attachment

- `/note <note>` becomes selection-aware in the TUI
- Plain text remains active-context capture
- This preserves a fast “note on what I am working on now” flow while still allowing notes on another visible item without activating it

## Implementation Guidance

### TUI dispatch

The current TUI slash-command flow is split between:

- special handling in `crates/liminal-flow-tui/src/app.rs`
- generic intent execution in `crates/liminal-flow-tui/src/input.rs`

Implementation should move toward one shared target-resolution layer so command semantics are defined once.

Desired outcome:

- Insert-mode slash commands and Normal-mode item actions follow the same targeting rules where they overlap
- Selected-item commands resolve a selected target first
- Active-context commands resolve the active context
- Active-focus-stack commands resolve against the active branch/thread stack

### Docs

Before or alongside code changes, update:

- `SPEC.md`
- `README.md`
- TUI help text
- command palette copy

The docs should explain the distinction between:

- selected-item commands
- active-context commands
- active-focus-stack commands

## Surface Impact

- [Issue #25](https://github.com/liminal-hq/flow/issues/25) is TUI-only
- [Issue #26](https://github.com/liminal-hq/flow/issues/26) is TUI-only
- [Issue #28](https://github.com/liminal-hq/flow/issues/28) affects both TUI and CLI

## Implementation Order

1. Update `SPEC.md` and `README.md` so the command-targeting model is defined clearly.
2. Refactor TUI slash-command dispatch around shared target resolution.
3. Make `/resume`, `/pause`, `/park`, `/done`, `/archive`, and `/note <note>` selection-aware in Insert mode.
4. Keep `/now`, `/branch`, plain text, and `/where` as active-context commands.
5. Keep `/back` as an active-focus-stack command.
6. Fix TUI note timestamp rendering to use local time.
7. Update thread completion behaviour in both TUI and CLI so thread completion cascades to non-archived branches.
8. Add or update tests for command targeting, lifecycle transitions, completion cascading, and local-time rendering.
9. Run verification:
   - `cargo fmt --check`
   - `cargo build`
   - `cargo clippy --workspace -- -D warnings`
   - `cargo test`

## Source

Derived from issue triage and decision capture for [Issue #25](https://github.com/liminal-hq/flow/issues/25), [Issue #26](https://github.com/liminal-hq/flow/issues/26), and [Issue #28](https://github.com/liminal-hq/flow/issues/28) on March 12-13, 2026.
