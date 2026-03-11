# Liminal Flow — Detailed Implementation Plan

## Purpose

This document turns the current Liminal Flow concept into a practical implementation plan.

It is organised around two delivery stages:

- **v1** — local-first capture and continuity with no required model
- **v1.1** — optional local inference runtime that refines raw captures into structured events

This plan assumes:

- the product name is **Liminal Flow**
- the terminal command is **`flo`**
- the TUI is the primary home of the experience
- the CLI is a companion interface that can write into shared global state

---

## Product framing

### What Liminal Flow is

Liminal Flow is a terminal-native working-memory sidecar.

It helps the user:

- record what they are doing right now
- branch briefly into side activities
- return to the main thread of work
- ask what is currently alive
- recover continuity after walking away

### What Liminal Flow is not

It is not:

- a traditional to-do list
- a project manager
- a calendar
- a full notebook
- a full AI agent

### Core product rule

**The event pipeline is the product.**

Local inference can improve the experience, but the app must remain useful with plain text alone.

---

## Delivery roadmap

## v1 — Capture, persist, resume

### v1 goals

v1 must be good enough to use daily without any model installed.

It should deliver:

- one global current thread
- lightweight branches under that thread
- a TUI for live continuity
- a CLI for fast capture from anywhere
- local persistence
- short replies and summaries using deterministic logic where possible
- optional context hints from repo, cwd, git branch, and shell helpers

### v1 non-goals

Do not include in v1:

- LLM dependency
- voice capture
- embeddings
- cloud model support
- rich automation
- heavy process scraping
- complex plugin systems

## v1.1 — Optional local inference runtime

### v1.1 goals

v1.1 adds a user-opt-in local inference runtime that can:

- classify input intent
- normalise titles
- improve branch detection
- generate terse UI replies
- refresh short summaries

### v1.1 constraints

- the model must be optional
- raw user input remains the source of truth
- every inferred event must be schema-validated before state changes
- fallback to verbatim mode must always exist

---

## System architecture

### High-level architecture

```text
Liminal Flow
├── Shared App Core
│   ├── domain model
│   ├── event pipeline
│   ├── state reducer
│   ├── persistence
│   └── query layer
├── TUI App
│   ├── list pane
│   ├── reply pane
│   ├── chat/input pane
│   └── keyboard navigation
├── CLI App (`flo`)
│   ├── command parser
│   ├── event emitter
│   └── query output
├── Context Layer
│   ├── shell helper ingestion
│   ├── git/repo discovery
│   ├── cwd enrichment
│   └── optional ambient hints
└── Optional Inference Layer (v1.1)
    ├── VerbatimEngine
    ├── LocalInferenceEngine
    ├── model lifecycle
    └── output validation
```

### Core design choice

The TUI and CLI should not have separate logic for state changes.

Both should write events into the same application core.

That means:

- one shared domain model
- one shared reducer / state transition layer
- one shared persistence layer
- one shared query layer

---

## Repository structure

### Recommended workspace layout

```text
liminal-flow/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── liminal-flow-core/
│   ├── liminal-flow-store/
│   ├── liminal-flow-cli/
│   ├── liminal-flow-tui/
│   ├── liminal-flow-context/
│   ├── liminal-flow-infer/
│   ├── liminal-flow-model/
│   └── liminal-flow-shell-helper/
├── docs/
│   ├── architecture/
│   ├── product/
│   ├── schema/
│   └── tui/
├── scripts/
├── fixtures/
└── tests/
```

### Crate responsibilities

#### `liminal-flow-core`
Holds the domain model and app logic.

Contains:

- thread, branch, capture, reply types
- event definitions
- reducer / state transition logic
- app queries
- deterministic interpretation helpers
- validation rules

#### `liminal-flow-store`
Owns persistence and migrations.

Contains:

- SQLite schema
- migration code
- repository traits and implementations
- session log writer
- config file loading and saving

#### `liminal-flow-cli`
Owns the `flo` binary.

Contains:

- `clap` definitions
- command execution
- terminal-friendly output formatting
- event emission into shared core

#### `liminal-flow-tui`
Owns the TUI application.

Contains:

- Ratatui view rendering
- Crossterm integration
- input handling
- async refresh wiring
- pane state and navigation

#### `liminal-flow-context`
Owns workspace enrichment.

Contains:

- cwd discovery
- git branch reading
- repo root detection
- shell helper event ingestion
- optional ambient hint collectors

#### `liminal-flow-infer`
Owns inference abstraction.

Contains:

- inference trait
- `VerbatimEngine`
- inference request/response schemas
- output validation
- prompt versioning support

#### `liminal-flow-model`
Only used for v1.1 and later.

Contains:

- local model lifecycle logic
- model discovery
- install status reporting
- warmup helpers
- model runtime adapters

#### `liminal-flow-shell-helper`
Optional helper binary or scripts.

Contains:

- shell integration snippets
- prompt hooks
- event publishing
- helper install instructions

---

## Rust stack

### Core runtime and application

Use:

- `tokio` for async and background jobs
- `serde` and `serde_json` for serialisation
- `thiserror` or `anyhow` for error handling
- `uuid` or `ulid` for IDs
- `time` or `chrono` for timestamps
- `tracing` and `tracing-subscriber` for structured diagnostics

### TUI stack

Use:

- `ratatui`
- `crossterm`
- `tui-textarea`

### CLI stack

Use:

- `clap`

### Persistence stack

Use:

- `rusqlite`
- a simple migration layer such as `rusqlite_migration` or an equivalent internal migration runner

### Optional context stack

Potentially use:

- `gix` or direct git command execution for repo metadata
- `notify` later for filesystem observation if needed

### Optional v1.1 local inference stack

Keep the interface abstract.

Potential backends can be decided later.

The implementation plan should assume:

- raw text mode first
- optional small local model second
- model backend hidden behind a trait boundary

---

## Domain model

### Primary entities

#### Thread
Represents the main top-level work item currently alive.

Fields:

- `id`
- `title`
- `raw_origin_text`
- `status`
- `short_summary`
- `created_at`
- `updated_at`

#### Branch
Represents a lightweight offshoot of attention within a thread.

Fields:

- `id`
- `thread_id`
- `title`
- `status`
- `short_summary`
- `created_at`
- `updated_at`

#### Capture
Represents raw user input before or alongside refinement.

Fields:

- `id`
- `target_type`
- `target_id`
- `text`
- `source`
- `inferred_intent`
- `created_at`

#### Scope
Represents structured context attached to a thread or branch.

Fields:

- `id`
- `target_type`
- `target_id`
- `kind`
- `value`
- `confidence`
- `observed_at`

#### Hint
Represents lower-confidence observed context.

Fields:

- `id`
- `kind`
- `value`
- `confidence`
- `observed_at`

#### Reply
Represents the short status voice shown in the TUI.

Fields:

- `text`
- `kind`
- `created_at`

### Recommended enums

#### `ThreadStatus`
- `Active`
- `Paused`
- `Done`
- `Dropped`

#### `BranchStatus`
- `Active`
- `Parked`
- `Done`
- `Dropped`

#### `CaptureSource`
- `Keyboard`
- `Cli`
- `Voice`
- `Import`
- `System`

#### `Intent`
- `SetCurrentThread`
- `StartBranch`
- `ReturnToParent`
- `AddNote`
- `QueryCurrent`
- `Pause`
- `Done`
- `Ambiguous`

#### `ScopeKind`
- `Repo`
- `Cwd`
- `GitBranch`
- `Workspace`
- `Host`

#### `HintKind`
- `Process`
- `Command`
- `Tty`
- `Activity`

---

## Event model

### Why an event model

The app should not let the UI mutate state directly.

Every meaningful action should become a domain event.

This keeps:

- TUI behaviour predictable
- CLI behaviour consistent with TUI behaviour
- state transitions testable
- inference optional and replaceable

### Core events

```text
AppEvent
├── CaptureReceived
├── ThreadSetCurrent
├── BranchStarted
├── ReturnedToParent
├── NoteAttached
├── ThreadPaused
├── ThreadMarkedDone
├── ScopeObserved
├── HintObserved
├── ReplyUpdated
└── QueryAnswered
```

### Suggested event payloads

#### `CaptureReceived`
- `capture_id`
- `text`
- `source`
- `created_at`

#### `ThreadSetCurrent`
- `thread_id`
- `title`
- `created_at`

#### `BranchStarted`
- `branch_id`
- `thread_id`
- `title`
- `created_at`

#### `ReturnedToParent`
- `thread_id`
- `parked_branch_ids`
- `created_at`

#### `NoteAttached`
- `target_type`
- `target_id`
- `capture_id`
- `created_at`

### Reducer responsibilities

The reducer should:

- apply events to the in-memory state
- enforce basic rules
- update timestamps
- ensure only one current thread exists
- ensure branch states remain valid

---

## State transition rules

### Core rules

#### Rule 1 — Only one current thread
When a thread becomes current, any previously active thread becomes paused unless explicitly dropped or done.

#### Rule 2 — Branches belong to one thread
A branch can only exist beneath one parent thread.

#### Rule 3 — Returning to parent parks active branches
When the user says `back`, the active branch becomes `parked` and focus returns to the parent thread.

#### Rule 4 — Raw capture is always stored
Even if interpretation fails, the raw capture is persisted.

#### Rule 5 — Queries do not mutate work state unless explicitly designed to
A query like `what am I currently working on?` should produce a reply, not rewire state.

### Ambiguity handling rule

When the app is unsure, prefer:

- `AddNote`
- minimal reply
- no aggressive restructuring

---

## Persistence plan

### Storage responsibilities

Use SQLite as the durable source of truth.
Use a small append-only JSONL session log for recent event replay and diagnostics.

### Main files

#### Linux
- config: `$XDG_CONFIG_HOME/liminal-flow/config.toml` or `~/.config/liminal-flow/config.toml`
- data: `$XDG_DATA_HOME/liminal-flow/data.sqlite3` or `~/.local/share/liminal-flow/data.sqlite3`
- state: `$XDG_STATE_HOME/liminal-flow/session.jsonl` or `~/.local/state/liminal-flow/session.jsonl`
- cache: `$XDG_CACHE_HOME/liminal-flow/` or `~/.cache/liminal-flow/`
- runtime: `$XDG_RUNTIME_DIR/liminal-flow/`

#### macOS
- config and app data: `~/Library/Application Support/Liminal Flow/`
- cache: `~/Library/Caches/Liminal Flow/`
- logs: `~/Library/Logs/Liminal Flow/`
- temp runtime: `$TMPDIR/liminal-flow/`

#### Windows
- config: `%APPDATA%\Liminal Flow\config.toml`
- local data: `%LOCALAPPDATA%\Liminal Flow\data.sqlite3`
- state/logs/cache: `%LOCALAPPDATA%\Liminal Flow\...`
- temp: `%TEMP%\Liminal Flow\`

### Schema plan

Use migrations from the start.

Tables:

- `threads`
- `branches`
- `captures`
- `scopes`
- `hints`
- optionally `events`

### Recommendation on event storage

For v1, either of these is acceptable:

#### Option A — No dedicated `events` table
Use SQLite tables as the main state and JSONL for recent event history.

#### Option B — Add a lightweight `events` table early
Store domain events explicitly for debugging and future replay.

Recommended choice:

Use **Option B** if you want Liminal Flow to evolve cleanly.

Suggested `events` table:

| column | type | notes |
|---|---|---|
| id | text | primary key |
| event_type | text | domain event name |
| payload_json | text | serialised payload |
| created_at | text | ISO timestamp |
| source | text | tui, cli, system, infer |

---

## Config plan

### `config.toml`

Suggested sections:

```toml
[ui]
show_scopes = true
show_hints = false
compact_mode = false

[inference]
enabled = false
engine = "verbatim"
model_name = ""
auto_warm = false

[context]
shell_helper_enabled = false
git_enrichment = true
ambient_hints = false

[logging]
level = "info"
json = false
```

### Config loading rules

- load defaults first
- overlay file config
- overlay environment variables if desired later
- validate before app start

---

## TUI implementation plan

### Primary layout

Use a three-pane layout:

- left: live threads and branches
- upper right: reply/status pane
- bottom: chat/input pane

### TUI states

#### Normal mode
Used for navigation and basic actions.

#### Insert mode
Used when typing into the chat box.

#### Help mode
Used for keybinding and command help.

#### Optional command palette later
Not required for v1.

### Suggested keybindings for v1

#### Global
- `q` — quit
- `?` — help
- `i` — focus input
- `Esc` — leave input mode

#### Navigation
- `j` / `Down` — move down
- `k` / `Up` — move up
- `Enter` — inspect or focus selected item

#### Actions
- `n` — focus input for new thought
- `b` — quick branch prompt
- `p` — pause current thread
- `d` — mark done
- `r` — refresh query / answer current state

### TUI render loop responsibilities

- render state from app core
- display last reply
- show current thread and branches clearly
- show thread statuses with minimal clutter
- remain usable on narrower terminal widths

### TUI background tasks

- poll store changes when the CLI writes events
- refresh reply pane after applied events
- update clock/timestamps if shown later

### Recommended initial rendering constraints

Do not over-decorate.

Keep:

- simple borders
- strong focus indicator
- minimal colour dependence
- readable on dark and light terminals

---

## CLI implementation plan

### Command philosophy

The CLI is a capture and query interface.
It should be quick, stable, and scriptable.

### v1 commands

#### `flo now <text>`
Set or replace the current top-level thread.

#### `flo branch <text>`
Create a branch beneath the current thread.

#### `flo back`
Return attention to the parent thread.

#### `flo note <text>`
Attach a raw note to the current focus target.

#### `flo where`
Print a short answer describing the current thread and branches.

#### `flo pause`
Pause the current thread.

#### `flo done`
Mark the current thread done.

#### `flo list`
Print active threads in a compact list.

### Optional convenience behaviour

Support raw capture as a default action later:

```bash
flo "improving AIDX for the component library"
```

But do not require this for v1.

### Output style

Prefer compact human-readable output by default.

Examples:

```text
Current thread: improving AIDX
Branches: answering question from support, reading article
```

Later you can add:

- `--json`
- `--quiet`
- `--no-color`

### CLI source tagging

All CLI writes should be tagged with source `cli`.

---

## Shared store access plan

### Problem to solve

The TUI and CLI both need safe shared access to the same local store.

### Recommended approach

Use SQLite as the coordination point.

The CLI writes events and exits.
The TUI polls for updates or listens for a light IPC signal.

### v1 implementation choice

Keep it simple:

- SQLite writes for every mutation
- TUI polling at a low interval or on input loop ticks
- optional file/socket nudge later

### Optional later improvement

Use a local runtime socket or file-based notification under:

- Linux: `$XDG_RUNTIME_DIR/liminal-flow/socket`
- macOS: `$TMPDIR/liminal-flow/socket`
- Windows: named pipe or local IPC equivalent

That is not required for v1.

---

## Context layer implementation plan

### v1 context sources

#### 1. Explicit user input
Always highest confidence.

#### 2. Optional shell helper
Can send:

- cwd
- repo root
- git branch
- maybe the last command

#### 3. Direct repo and git discovery
If the user is operating in a known path, derive:

- repo root
- current git branch

#### 4. Ambient hints
Only later or optionally:

- process names
- long-running commands
- active TTY hints

### Shell helper strategy

Keep shell helpers tiny.

They should publish events rather than own logic.

Examples:

- `flo helper publish-context`
- shell prompt hook calls helper with cwd and branch

### Rule for context attachment

Context is enrichment, not authority.

Attach after capture whenever possible.

---

## Deterministic interpretation plan for v1

### Why deterministic interpretation matters

v1 should feel useful without a model.

That means using a small amount of local non-LLM logic to improve behaviour.

### Suggested deterministic rules

#### Rule A — Question detection
If input ends with `?` or matches known phrases like:

- `what am I currently working on`
- `where was I`
- `what is active`

Then treat as `QueryCurrent`.

#### Rule B — Back detection
If input starts with or matches phrases like:

- `back`
- `back to`
- `return to`

Then treat as `ReturnToParent`.

#### Rule C — Branch-friendly fragment detection
If a current thread exists and input is a short verb-led fragment like:

- `reading article`
- `answering support`
- `checking logs`

Then treat as `StartBranch`.

#### Rule D — New thread detection
If no current thread exists, or the input clearly states a main activity like:

- `I'm improving AIDX`
- `working on docs`

Then treat as `SetCurrentThread`.

#### Rule E — Fallback note
If none of the above match safely, store as `AddNote`.

### Deterministic title normalisation

Simple cleanup rules:

- trim whitespace
- lowercase only if stylistically appropriate
- strip leading phrases like `I'm`, `I am`, `working on`
- preserve acronyms like AIDX
- keep titles short and readable

---

## Optional inference plan for v1.1

### Inference boundary

Define a trait like:

```rust
trait InferenceEngine {
    fn interpret(&self, req: InterpretRequest) -> Result<InterpretResult, InterpretError>;
}
```

### `InterpretRequest`

Fields:

- raw input text
- current thread title
- active branch titles
- recent captures
- current scopes
- prompt version

### `InterpretResult`

Fields:

- intent
- canonical_title
- target_type
- target_reference
- reply_text
- optional scope hints
- confidence

### v1.1 engines

#### `VerbatimEngine`
- no model
- deterministic rules only
- always available

#### `LocalInferenceEngine`
- optional model-backed engine
- used only when enabled and healthy
- falls back to `VerbatimEngine` on failure

### Output validation rules

- must parse cleanly
- intent must be one of the allowed enums
- target must be consistent with current state
- confidence may be advisory only
- invalid output becomes `AddNote`

### Prompt versioning

Store a prompt version string with inferred events.

This helps later when behaviour changes between releases.

### Model lifecycle

If enabled:

1. check configured model
2. confirm local availability
3. load runtime
4. optionally warm model
5. interpret request
6. validate structured output
7. apply event or fall back

### Model installation UX

Not required for v1.1 launch, but design around it.

The user should explicitly choose to install a model.

Future CLI support:

- `flo model status`
- `flo model install`
- `flo model remove`

---

## Testing plan

### Unit tests

Test:

- title normalisation
- intent classification heuristics
- reducer transitions
- config validation
- storage repositories

### Integration tests

Test:

- CLI writing into the shared store
- TUI reading updated state
- migration application on new and existing databases
- shell helper event ingestion

### Golden / snapshot tests

Useful for:

- CLI output
- TUI small-screen layouts
- reply formatting

### Event sequence tests

Build scenario tests like:

1. `now improving AIDX`
2. `branch answering support`
3. `branch reading article`
4. `back`
5. `where`

Assert final state and reply output.

### v1.1 inference tests

Test:

- valid model result gets applied
- invalid model result falls back safely
- ambiguous result becomes note
- prompt version recorded correctly

---

## Observability plan

### Logging

Use `tracing` everywhere.

Key spans and events:

- app start
- config load
- migration run
- capture received
- event emitted
- reducer applied
- reply updated
- CLI command executed
- context attached
- inference requested
- inference returned
- inference validation failed

### Debugging modes

Later CLI helpers can include:

- `flo trace`
- `flo doctor`
- `flo infer test`

### Session log usage

The JSONL session log should help inspect:

- recent actions
- crash recovery clues
- surprising event sequences

---

## Security and privacy plan

### Local-first policy

The default app should not require network access.

### User input handling

- store only local user-entered data
- keep model usage local when enabled
- make any future networked features explicitly opt-in

### Sensitive context handling

Because context may include repo paths or command hints:

- keep everything local by default
- avoid excessive process scraping in v1
- let users disable shell helpers and context enrichment

---

## Implementation sequence

## Phase 0 — Bootstrap

Deliver:

- Cargo workspace
- core crate scaffold
- store crate scaffold
- CLI crate scaffold
- TUI crate scaffold
- config loading
- storage path resolution by platform
- SQLite initial migration

## Phase 1 — Core state and events

Deliver:

- domain entities
- event definitions
- reducer
- store repositories
- `events` table or JSONL session log
- basic unit tests

## Phase 2 — CLI first path

Deliver:

- `flo now`
- `flo branch`
- `flo back`
- `flo note`
- `flo where`
- `flo pause`
- `flo done`
- `flo list`

This gives fast feedback before the TUI is finished.

## Phase 3 — TUI shell

Deliver:

- three-pane layout
- input box
- thread list rendering
- reply pane rendering
- keyboard navigation
- shared store reads

## Phase 4 — Deterministic interpretation

Deliver:

- question detection
- back detection
- simple branch detection
- fallback note logic
- title cleanup
- short reply generation

At the end of this phase, v1 should be usable.

## Phase 5 — Context enrichment

Deliver:

- cwd and repo detection
- git branch attachment
- optional shell helper ingestion
- lightweight scope display in TUI

## Phase 6 — v1 hardening

Deliver:

- migration tests
- integration tests
- snapshot tests
- logging polish
- error messages
- packaging preparation

## Phase 7 — v1.1 inference adapter

Deliver:

- inference trait
- `VerbatimEngine`
- request/response schema
- safe integration with reducer
- config flags for inference

## Phase 8 — optional local model runtime

Deliver:

- local model discovery
- runtime load and warmup
- structured output validation
- fallback handling
- `flo model status`
- `flo infer test`

---

## Packaging and distribution plan

### v1 distribution targets

Primary targets:

- Linux
- macOS
- Windows

### Binary outputs

At minimum:

- `flo` CLI binary
- `liminal-flow` TUI binary or a single binary with mode switching

### Recommended packaging decision

Prefer one binary first if it simplifies shipping.

Possible mode shapes:

- `flo tui`
- `flo now ...`
- `flo where`

Alternative:

- `flo` for CLI
- `liminal-flow` for full TUI

Make the final naming choice based on ergonomics near packaging time.

---

## Open design questions

These do not block implementation, but should be answered before locking v1.

### 1. Single binary or two binaries?
Should the TUI and CLI ship as one executable or two?

### 2. Event table or JSONL only?
Do you want full event persistence in SQLite from day one?

### 3. How aggressive should deterministic branch inference be?
Too aggressive will feel clever but untrustworthy.

### 4. How much context should be shown visibly?
Repo and branch are useful, but too much context becomes clutter.

### 5. Should `flo` support freeform raw capture at launch?
Example:

```bash
flo "improving AIDX for the component library"
```

### 6. How should TUI refresh work when CLI mutates state?
Polling is simplest; IPC is cleaner later.

---

## Recommended first coding target

Build this slice first:

1. storage path resolution
2. SQLite setup
3. event model
4. reducer
5. `flo now`
6. `flo where`
7. tiny TUI showing current thread, reply, and input box

That gives a complete vertical slice of the product with very little wasted work.

---

## One-sentence implementation summary

**Build Liminal Flow as a shared local event-driven core with a TUI for continuity, a CLI for capture, deterministic interpretation for v1, and an optional model-backed inference adapter layered on top in v1.1.**

