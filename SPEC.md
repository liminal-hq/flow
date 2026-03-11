# Liminal Flow — Product Specification

## Overview

Liminal Flow is a terminal-native working-memory sidecar. It tracks what you're working on, supports branching attention across sub-tasks, and provides ambient context awareness — all from the terminal.

The CLI command is `flo`. Running `flo` with no arguments launches the TUI. Subcommands (`flo now`, `flo where`, etc.) operate headlessly for scripting and quick capture.

## Core Concepts

### Threads

A **thread** represents a unit of focused work. Only one thread can be **active** at a time. Other threads may be **paused**, **done**, or **dropped**.

When you start a new thread with `flo now`, any currently active thread is automatically paused.

### Branches

A **branch** is a sub-task beneath a thread. Branches allow you to track tangential work without losing your place. Branches can be **active**, **parked**, **done**, or **dropped**.

Use `flo branch` to start a branch and `flo back` to return to the parent thread (parking all active branches).

### Captures

A **capture** is a piece of raw user input. Every interaction — starting a thread, adding a note, creating a branch — produces a capture that is stored alongside its inferred intent.

### Scopes

A **scope** is structured context automatically attached to threads and branches at creation time. Scopes include:

- **Repo** — the git repository name
- **Git branch** — the current git branch
- **Cwd** — the working directory

### Events

Every state mutation produces an event stored in the events table. Events serve as an audit log and enable the TUI to detect changes made by the CLI in another terminal.

## CLI Commands

| Command | Description |
|---|---|
| `flo` | Launch the TUI |
| `flo now <text>` | Set or replace the current thread |
| `flo branch <text>` | Create a branch beneath the current thread |
| `flo back` | Return to the parent thread, parking active branches |
| `flo note <text>` | Attach a note to the current focus target |
| `flo where` | Print the current thread and its branches |
| `flo pause` | Pause the current thread |
| `flo done` | Mark the current thread done |
| `flo list` | List active and paused threads |
| `flo list -a` | List threads with branches, statuses, and recent notes |

### Title Normalisation

Thread and branch titles are normalised by stripping common prefixes:

- "I'm working on" → stripped
- "I am working on" → stripped
- "I'm" → stripped
- "I am" → stripped
- "working on" → stripped

For example, `flo now "I'm working on the component library"` creates a thread titled `the component library`.

## TUI

The TUI provides a three-pane interface:

```
┌────────────────────────────────┬─────────────────────────────────┐
│ Liminal Flow                   │                        < flo >  │
├────────────────────────────────┼─────────────────────────────────┤
│ > ▼ improving AIDX             │ Current thread: improving AIDX  │
│       answering support        │ 1 active branch                 │
│       reading article          │ Repo: component-library         │
│   ▶ wear os sync  paused      │ Git: feature/aidx               │
│                                │                                 │
│                                │ Notes                           │
│                                │   | need to check auth flow     │
├────────────────────────────────┴─────────────────────────────────┤
│ > Capture                                                        │
└──────────────────────────────────────────────────────────────────┘
```

- **Left pane (30%):** Thread list with branches indented beneath
- **Right pane (70%):** Status display with scope context
- **Bottom (3 lines):** Chat-style input with tui-textarea

### Modes

| Mode | Description |
|---|---|
| Insert | Text input active. Enter submits. Esc switches to Normal. |
| Normal | Keyboard navigation. `j`/`k`/Up/Down to move through threads and branches, `r` to resume selected item, `i` to insert, `?` for help, `a` for about, `q` to quit. |
| Help | Help overlay. Esc or `?` to dismiss. |
| About | About overlay with app info. Esc, `q`, or Enter to dismiss. |

### Thread and Branch Navigation

The thread list supports navigating both threads and their branches:

- **Up/Down** (Insert or Normal mode) moves between all visible items — threads and branches within expanded threads
- **Enter** (on empty input in Insert, or in Normal mode) toggles expand/collapse for the selected thread's branches
- **r** (Normal mode) resumes the selected item:
  - On a paused thread: activates it (pausing the current active thread)
  - On a parked branch: activates it (parking other active branches, and activating the parent thread if needed)

### Command Palette and Hints

- Type `/` on an empty input line to open the **command palette** — a floating popup showing available slash commands. Navigate with Up/Down, select with Enter/Tab, dismiss with Esc.
- Type `?` on an empty input line to show **shortcut hints** — a compact reference bar. Any key dismisses it.

### Slash Commands

The TUI accepts the same commands as the CLI, prefixed with `/`:

`/now`, `/branch`, `/back`, `/note`, `/where`, `/pause`, `/done`

Plain text (without a `/` prefix) is treated as a note attached to the current focus target.

### Polling

The TUI polls the SQLite database every 250ms for changes. This means threads created via `flo now` in another terminal appear in the TUI automatically.

## Persistence

All data is stored in a local SQLite database using WAL mode for safe concurrent access between the CLI and TUI.

**Default paths (Linux):**

- Database: `~/.local/share/liminal-flow/liminal-flow.db`
- Config: `~/.config/liminal-flow/config.toml`

**Default paths (macOS):**

- Database: `~/Library/Application Support/ca.liminalhq.liminal-flow/liminal-flow.db`
- Config: `~/Library/Application Support/ca.liminalhq.liminal-flow/config.toml`

## Five Core Rules

1. **Single active thread** — only one thread can be active at any time
2. **Auto-pause** — starting a new thread pauses the current one
3. **Branches require a thread** — you cannot branch without an active thread
4. **Back parks branches** — returning to parent parks all active branches
5. **Events are immutable** — every state change produces an append-only event

## Configuration

Configuration is loaded from `config.toml` at the platform-appropriate config directory.

```toml
[ui]
show_scopes = true
show_hints = false
compact_mode = false

[context]
shell_helper_enabled = false
git_enrichment = true
ambient_hints = false

[logging]
level = "info"
json = false
```

## Repository Labelling

GitHub issue and PR labelling should follow the broader Liminal HQ style rather than Conventional Commit terms.

- Use primary category labels such as `enhancement`, `bug`, `documentation`, `testing`, `ci`, `build`, and `chore`
- Use shared operational labels such as `infrastructure`, `internal`, `release`, `blocked`, `epic`, and `skip-changelog` when they clarify handling
- Use flow-specific scope labels such as `cli`, `tui`, `core`, `store`, `context`, `inference`, and `model` to describe affected areas

## Licence

MIT — see [LICENSE](LICENSE).
