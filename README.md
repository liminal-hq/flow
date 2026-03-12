# Liminal Flow

<p align="center">
  <img src="assets/hero.svg" alt="Liminal Flow — Terminal-native working memory for developers" width="100%">
</p>

A terminal-native working-memory sidecar. Track what you're working on, branch your attention across sub-tasks, and maintain ambient context awareness — all from the terminal.

## Quick Start

```bash
# Build from source
cargo build --release

# The binary is `flo`
./target/release/flo              # Launch TUI
./target/release/flo now "improving AIDX"   # Set current thread
./target/release/flo branch "debugging auth"  # Branch off
./target/release/flo where        # Show current state
./target/release/flo back         # Return to parent
./target/release/flo done         # Mark thread done
```

## What It Does

Liminal Flow keeps track of your working context so you don't have to. When you switch between tasks, branch into sub-problems, or need to remember what you were doing — `flo` has your back.

**Threads** are your main units of work. Only one is active at a time.

**Branches** let you track tangential sub-tasks without losing your place.

**Scopes** automatically capture your git repo, branch, and working directory.

**Events** log every state change, so the TUI can detect CLI changes in real time.

## CLI Commands

| Command | Description |
|---|---|
| `flo` | Launch the TUI |
| `flo now <text>` | Set or replace the current thread |
| `flo branch <text>` | Create a branch beneath the current thread |
| `flo back` | Return to the parent thread |
| `flo note <text>` | Attach a note to the current focus target |
| `flo where` | Print current thread and branches |
| `flo pause` | Pause the current thread |
| `flo done` | Mark the current thread done |
| `flo list` | List active and paused threads |

## TUI

Run `flo` with no arguments to launch the terminal UI:

```
┌────────────────────────────────────────────────────────────┐
│ Liminal Flow                                    <flo>      │
├──────────────────┬─────────────────────────────────────────┤
│ Threads          │ Status                                  │
│ > ▼ improving    │ Branch: debugging auth                  │
│     debugging    │ Thread: improving AIDX                  │
│   ▶ wear os sync │ Repo: component-library                 │
│                  │ Git: feature/aidx                        │
│                  │                                          │
│                  │ Notes                                    │
│                  │   need to check the auth token flow      │
│                  │   waiting on API response                │
├──────────────────┴─────────────────────────────────────────┤
│ > Capture (branch: debugging auth)                         │
└────────────────────────────────────────────────────────────┘
```

The TUI starts in **Insert mode**:

- Type slash commands (`/now`, `/branch`, `/back`, `/note`, `/where`, `/pause`, `/done`) or plain text (treated as a note)
- Type `/` on an empty line to open the **command palette** — navigate with arrow keys, select with Enter
- Type `?` on an empty line to see **shortcut hints**
- **Up/Down** arrows navigate the thread list; **Enter** on empty input expands/collapses branches
- The **Status** pane follows the selected thread or branch for inspection
- The **Capture** pane shows the active note target explicitly
- Selected-item notes in the **Status** pane show compact timestamps and separators for readability
- Press `Esc` for **Normal mode** where `j`/`k` navigate, `Enter` expands or collapses the selected thread, `r` resumes a selected item to make it active, `p` parks a selected branch, `?` opens help, `a` shows about, and `q` quits

The TUI polls the database every 250ms, so changes made via `flo` CLI in another terminal appear automatically.

## Architecture

Liminal Flow is a Rust workspace with five crates:

| Crate | Purpose |
|---|---|
| `liminal-flow-core` | Domain model, events, reducer, deterministic rules |
| `liminal-flow-store` | SQLite persistence, migrations, repositories |
| `liminal-flow-cli` | CLI entrypoint and command handlers |
| `liminal-flow-tui` | Terminal UI (ratatui + crossterm) |
| `liminal-flow-context` | Git and workspace context discovery |

### Key Design Decisions

- **Single binary**: `flo` with no args → TUI, subcommands → headless CLI
- **Local-first**: All data in SQLite with WAL mode for concurrent CLI + TUI access
- **Events table**: Append-only audit log enables TUI polling via watermark
- **Pure reducer**: State transitions are deterministic and tested
- **No forced background**: TUI uses the terminal's default background colour

## Building

```bash
cargo build                           # debug build
cargo build --release                 # release build
cargo test                            # run all tests
cargo clippy --workspace -- -D warnings  # lint
cargo fmt --check                     # check formatting
```

## Configuration

Config lives at the platform-appropriate config directory (e.g., `~/.config/liminal-flow/config.toml` on Linux):

```toml
[ui]
show_scopes = true
show_hints = false
compact_mode = false

[context]
git_enrichment = true

[logging]
level = "info"
```

## Persistence

SQLite database at the platform data directory (e.g., `~/.local/share/liminal-flow/liminal-flow.db` on Linux). WAL mode is enabled for safe concurrent access.

## Licence

MIT — see [LICENSE](LICENSE).

Copyright 2026 Liminal HQ, Scott Morris.
