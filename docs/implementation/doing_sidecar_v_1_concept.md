# Doing Sidecar — v1 Concept

## Core idea

**Liminal Flow** is a local-first, terminal-native working-memory sidecar for the shell.

`flo` is the short everyday terminal command.

The app can still be thought of internally as a working-memory sidecar, but the product identity for this concept is now **Liminal Flow**.

It is **not** a to-do list, a project manager, or a full agent system.

Its purpose is much smaller and more immediate:

- capture what the user says they are doing right now
- keep a visible history of what is still alive
- allow short branches of attention beneath the current thread
- answer brief natural-language questions like **"what am I currently working on"**
- help the user resume context after stepping away

The key loop is:

1. The user says a plain-language thing.
2. The app stores it immediately.
3. The app infers a little structure.
4. The app shows the current working state back to the user.
5. The user leaves and later returns.
6. The app helps them re-enter the active thread.

---

## Product identity

### Product name

**Liminal Flow**

### Terminal command

`flo`

### Naming rationale

The fuller name carries the calmer, more brandable product identity, while the short command keeps daily terminal use friction low.

Examples:

- `flo now "improving AIDX for the component library"`
- `flo branch "answering question from support"`
- `flo branch "reading article"`
- `flo back`
- `flo where`
- `flo note "article may explain the support issue"`
- `flo pause`
- `flo done`

### Positioning note

The product should present itself in the UI as **Liminal Flow**, while the shell entrypoint stays short and practical as `flo`.

## Product thesis

A normal task app manages future obligations.

This app manages **present-tense continuity**.

It should feel like:

- a calm terminal workspace
- a working-memory console
- a sidecar for the shell
- a place where active effort stays coherent

It should **not** feel like:

- a guilt machine
- a giant second brain
- a heavy knowledge-management system
- a context-taxing form to fill out

---

## Design principles

### 1. Capture first, infer later

The user should be able to type raw thought with almost no friction.

Example:

> I'm improving AIDX for the component library

That should become a live thread immediately, even before the app has cleaned it up.

### 2. The user owns the truth

Machine context is helpful, but the user’s statement is the source of truth.

If the app sees a repo or running process, that is only supporting context.

### 3. One global "now"

The app should always be able to answer one simple question:

> What am I currently working on?

That answer comes from a **global current thread**, not from whichever repo or terminal happens to be active.

### 4. Branches are lightweight

Real work forks constantly.

A branch is not a full project. It is a temporary offshoot of attention.

Example:

- improving AIDX
  - answering question from support
  - reading article
- back to AIDX

### 5. Replies stay short

This is not a chat transcript app.

The reply panel should act like a terse status voice.

### 6. Local-first by default

The app should work fully offline and store its own state locally.

Optional local model inference can make it feel more alive, but the app must still be useful without it.

---

## v1 scope

### In scope

- one global current thread
- lightweight branches beneath the current thread
- plain-language capture
- brief natural-language queries
- local persistence
- optional context enrichment from path/repo/branch/process hints
- short inferred replies

### Out of scope for v1

- long-form journalling
- project planning
- calendars and deadlines
- autonomous agents
- multi-user sync
- cloud dependency
- heavy embeddings pipeline

---

## Core concepts

### Thread

The main thing the user is doing right now.

Examples:

- improving AIDX
- debugging Wear OS sync issue
- rewriting board survey

### Branch

A temporary offshoot of the current thread.

Examples:

- answering question from support
- reading article
- checking logs

### Capture

A raw user input entered into the bottom chat box.

Examples:

- I'm improving AIDX for the component library
- answering question from support
- back to AIDX
- what am I currently working on?

### Reply

A short response from the app.

Examples:

- Current thread: improving AIDX
- Added branch: reading article
- Returned to parent thread: improving AIDX

### Scope

Optional machine or workspace context attached to a thread.

Examples:

- repo path
- git branch
- cwd
- host OS
- shell session hint

### Hint

Low-confidence machine-observed context.

Examples:

- running `pnpm test`
- open shell in component library repo
- `cargo` process active in another terminal

Hints should never override explicit user intent.

---

## State model

This is the minimum state model for v1.

### App state

```text
AppState
├── current_thread_id: ThreadId?
├── threads: Map<ThreadId, Thread>
├── ui: UiState
├── storage: StorageState
└── ambient_context: AmbientContext
```

### Thread state

```text
Thread
├── id
├── title
├── raw_origin_text
├── status
├── created_at
├── updated_at
├── branches: [Branch]
├── captures: [Capture]
├── scopes: [Scope]
├── hints: [Hint]
└── short_summary
```

### Branch state

```text
Branch
├── id
├── parent_thread_id
├── title
├── status
├── created_at
├── updated_at
├── captures: [Capture]
└── short_summary
```

### Capture state

```text
Capture
├── id
├── text
├── created_at
├── source
├── inferred_intent
└── attached_to
```

### Scope state

```text
Scope
├── kind: repo | cwd | git_branch | workspace | host
├── value
├── confidence
└── observed_at
```

### Hint state

```text
Hint
├── kind: process | command | tty | activity
├── value
├── confidence
└── observed_at
```

### UI state

```text
UiState
├── selected_list_item
├── input_buffer
├── reply_text
├── mode: normal | insert | command | help
└── last_error
```

### Recommended status values

Keep v1 simple.

#### Thread status

- `active` — the current live thread
- `paused` — still alive, but not current
- `done` — intentionally finished
- `dropped` — intentionally abandoned

#### Branch status

- `active` — currently inside this branch
- `parked` — branch exists, but attention returned to parent
- `done` — resolved branch
- `dropped` — no longer relevant

For the very first v1, thread status alone may be enough.

---

## Interaction model

The app should accept plain language and infer simple intent.

### Input examples

#### Start a thread

> I'm improving AIDX for the component library

Result:

- create or update current thread
- set current thread to `improving AIDX`
- attach inferred scope if available

#### Create a branch

> answering question from support

Result:

- add a branch under the current thread
- set that branch active

#### Return to parent thread

> back to AIDX

Result:

- mark active branch parked
- set parent thread as active focus

#### Query current state

> what am I currently working on?

Result:

- reply briefly with current thread and active branches

#### Add context note

> article might explain the support issue

Result:

- attach as a capture to the active branch or thread
- refresh summary

---

## Inference model for v1

Inference should stay narrow and cheap.

The app should try to infer only a few things from each capture:

- is this a new thread, a branch, a return action, a note, or a query?
- what is the clean title?
- should this attach to the current thread or current branch?
- what short reply should be shown?

### Example

Input:

> I'm improving AIDX for the component library

Possible inferred structure:

- intent: `start_thread`
- title: `improving AIDX`
- scope: `component library`
- reply: `Current thread: improving AIDX`

Input:

> answering question from support

Possible inferred structure:

- intent: `start_branch`
- title: `answering question from support`
- parent: `improving AIDX`
- reply: `Added branch: answering question from support`

Input:

> what am I currently working on?

Possible inferred structure:

- intent: `query_current`
- reply: `Current thread: improving AIDX. Active branches: answering question from support, reading article.`

---

## Context model

The app needs both **global state** and **scoped context**.

### Global state

Global state answers the human question:

> what is my current work?

This should not depend on a repo or folder.

### Scoped context

Scoped context enriches the thread.

Examples:

- current repo path
- current git branch
- current cwd
- last observed shell path
- process hints from the OS

### Confidence hierarchy

Use this order of trust:

1. explicit user statement
2. explicit shell helper context
3. repo and git discovery
4. process scraping and ambient hints

This prevents the app from confusing machine activity with human intent.

---

## Terminal workspace concept

The sidecar lives in its own terminal window.

The user flips between that window and their editor, browser, shell, or other terminals.

That makes the app a **stable memory console** rather than an embedded prompt helper.

The sidecar should feel like:

- a nearby control room for current work
- a calm place to re-anchor context
- a terminal-native continuity surface

It does not replace the shell. It remembers the meaning of what the user was doing across shells, repos, and desktop windows.

---

## Storage layout

This app should follow platform conventions and keep storage predictable.

Use a stable application identifier such as:

- `ca.liminalhq.liminal-flow`
- or a shorter user-facing folder name like `liminal-flow`

### Linux (XDG)

Respect XDG environment variables first.

#### Config

- `$XDG_CONFIG_HOME/liminal-flow/config.toml`
- fallback: `~/.config/liminal-flow/config.toml`

#### Persistent data

- `$XDG_DATA_HOME/liminal-flow/data.sqlite3`
- fallback: `~/.local/share/liminal-flow/data.sqlite3`

#### State

- `$XDG_STATE_HOME/liminal-flow/session.jsonl`
- fallback: `~/.local/state/liminal-flow/session.jsonl`

#### Cache

- `$XDG_CACHE_HOME/liminal-flow/`
- fallback: `~/.cache/liminal-flow/`

#### Runtime socket / lock / IPC

- `$XDG_RUNTIME_DIR/liminal-flow/`
- example: `$XDG_RUNTIME_DIR/liminal-flow/socket`

### macOS

Use standard per-user Library directories.

#### Config and persistent data

- `~/Library/Application Support/Liminal Flow/`
- examples:
  - `~/Library/Application Support/Doing Sidecar/config.toml`
  - `~/Library/Application Support/Doing Sidecar/data.sqlite3`
  - `~/Library/Application Support/Doing Sidecar/session.jsonl`

#### Cache

- `~/Library/Caches/Liminal Flow/`

#### Logs

- `~/Library/Logs/Liminal Flow/`

#### Temporary runtime files

- `$TMPDIR/liminal-flow/`

### Windows

Use AppData locations.

#### Config and roaming-friendly small settings

- `%APPDATA%\Liminal Flow\config.toml`

#### Persistent local data

- `%LOCALAPPDATA%\Liminal Flow\data.sqlite3`
- `%LOCALAPPDATA%\Liminal Flow\session.jsonl`

#### Cache

- `%LOCALAPPDATA%\Liminal Flow\Cache\`

#### Logs

- `%LOCALAPPDATA%\Liminal Flow\Logs\`

#### Temporary runtime files

- `%TEMP%\Liminal Flow\`

### Suggested file roles

#### `config.toml`

User-visible settings:

- keybindings
- model settings
- inference on/off
- shell helper settings
- UI preferences

#### `data.sqlite3`

Source of truth for:

- threads
- branches
- captures
- scopes
- summaries

#### `session.jsonl`

Append-only recent state history:

- window open events
- current thread changes
- branch changes
- crash-safe write-ahead style activity record

#### Cache directory

For:

- rendered summaries
- transient local model outputs
- temporary snapshots

---

## Suggested database model

A very small SQLite schema is enough for v1.

### `threads`

| column            | type | notes                         |
| ----------------- | ---- | ----------------------------- |
| id                | text | primary key                   |
| title             | text | canonical display title       |
| raw\_origin\_text | text | first raw capture             |
| status            | text | active, paused, done, dropped |
| short\_summary    | text | brief inferred summary        |
| created\_at       | text | ISO timestamp                 |
| updated\_at       | text | ISO timestamp                 |

### `branches`

| column         | type | notes                         |
| -------------- | ---- | ----------------------------- |
| id             | text | primary key                   |
| thread\_id     | text | parent thread                 |
| title          | text | branch title                  |
| status         | text | active, parked, done, dropped |
| short\_summary | text | brief inferred summary        |
| created\_at    | text | ISO timestamp                 |
| updated\_at    | text | ISO timestamp                 |

### `captures`

| column           | type | notes                                             |
| ---------------- | ---- | ------------------------------------------------- |
| id               | text | primary key                                       |
| target\_type     | text | thread or branch                                  |
| target\_id       | text | linked entity                                     |
| text             | text | raw user text                                     |
| source           | text | keyboard, voice, import                           |
| inferred\_intent | text | start\_thread, start\_branch, note, query, return |
| created\_at      | text | ISO timestamp                                     |

### `scopes`

| column       | type | notes                                   |
| ------------ | ---- | --------------------------------------- |
| id           | text | primary key                             |
| target\_type | text | thread or branch                        |
| target\_id   | text | linked entity                           |
| kind         | text | repo, cwd, git\_branch, workspace, host |
| value        | text | string value                            |
| confidence   | real | 0.0 to 1.0                              |
| observed\_at | text | ISO timestamp                           |

### `hints`

| column       | type | notes                           |
| ------------ | ---- | ------------------------------- |
| id           | text | primary key                     |
| kind         | text | process, command, tty, activity |
| value        | text | serialized hint                 |
| confidence   | real | 0.0 to 1.0                      |
| observed\_at | text | ISO timestamp                   |

---

## Sample TUI layout

### Main layout

```text
┌───────────────────────────────┬──────────────────────────────────────┐
│ Doing Now                     │ Reply                                │
│────────────────────────────── │──────────────────────────────────────│
│ > improving AIDX              │ Current thread: improving AIDX       │
│     answering support         │ 2 active branches                    │
│     reading article           │ Last movement: reading article       │
│                               │ Repo: component-library              │
│   wear os sync issue          │ Git: feature/aidx-improvements       │
│   board survey rewrite        │                                      │
├───────────────────────────────┴──────────────────────────────────────┤
│ Chat                                                                 │
│──────────────────────────────────────────────────────────────────────│
│ > what am I currently working on?                                    │
└──────────────────────────────────────────────────────────────────────┘
```

### Behaviour of each panel

#### Left panel

The visible history of what is still alive.

Shows:

- current thread
- branches under the current thread
- other live threads

#### Reply panel

A short status voice.

Shows:

- current interpretation
- active thread summary
- active branch count
- recently observed scope

#### Chat panel

The universal input box.

Used for:

- capture
- notes
- branch creation
- returns
- questions

---

## Sample TUI screens

### Screen 1 — Fresh capture

```text
Doing Now
────────────────────────────────────────
> improving AIDX

Reply
────────────────────────────────────────
Current thread: improving AIDX

Chat
────────────────────────────────────────
> I'm improving AIDX for the component library
```

### Screen 2 — Branching attention

```text
Doing Now
────────────────────────────────────────
> improving AIDX
    answering question from support

Reply
────────────────────────────────────────
Added branch: answering question from support

Chat
────────────────────────────────────────
> answering question from support
```

### Screen 3 — Another branch

```text
Doing Now
────────────────────────────────────────
> improving AIDX
    answering question from support
    reading article

Reply
────────────────────────────────────────
Added branch: reading article

Chat
────────────────────────────────────────
> reading article
```

### Screen 4 — Return to parent

```text
Doing Now
────────────────────────────────────────
> improving AIDX
    answering question from support   parked
    reading article                   parked

Reply
────────────────────────────────────────
Returned to parent thread: improving AIDX

Chat
────────────────────────────────────────
> back to AIDX
```

### Screen 5 — Resume after stepping away

```text
Doing Now
────────────────────────────────────────
> improving AIDX
    answering question from support   parked
    reading article                   parked

Reply
────────────────────────────────────────
Current thread: improving AIDX
Recent branches: answering question from support, reading article
Last note: article may explain the support issue

Chat
────────────────────────────────────────
> what am I currently working on?
```

### Screen 6 — Empty but ready

```text
Doing Now
────────────────────────────────────────
(no active thread)

Reply
────────────────────────────────────────
Start by telling me what you're doing.

Chat
────────────────────────────────────────
>
```

---

## Sample user flows

### Flow A — Start work

1. Open sidecar window.
2. Type: `I'm improving AIDX for the component library`
3. App creates current thread.
4. App replies: `Current thread: improving AIDX`

### Flow B — Branch mid-work

1. Type: `answering question from support`
2. App creates branch under current thread.
3. App replies: `Added branch: answering question from support`

### Flow C — Return to main work

1. Type: `back to AIDX`
2. App parks the current branch.
3. App replies: `Returned to parent thread: improving AIDX`

### Flow D — Resume later

1. Leave the app open or return later.
2. Type: `what am I currently working on?`
3. App replies with current thread, branches, and latest note.

---

## Context ingestion strategy

The app should avoid making capture depend on machine introspection.

### First-class input

- keyboard text in the chat box
- later: voice transcription

### Optional context enrichment

- current host OS
- manually linked repo or cwd
- shell helper events
- git branch discovery
- process observation

### Important rule

Context should be attached **after** capture whenever possible.

The user should never need to pause and carefully classify their thought before saving it.

---

## Cross-platform context hints

Context discovery should be layered.

### Best source

A small optional shell helper that reports:

- cwd
- repo root
- git branch
- maybe the last command

### Good source

Repo discovery from known paths and recent scopes.

### Weak source

OS-level process inspection.

This is useful for hints such as:

- active shell windows
- long-running build or test commands
- likely repo locations

But it should never be treated as the source of truth for what the user is doing.

---

## Minimal command language

The app should support plain language first, but a compact CLI command surface makes the system scriptable, easier to integrate, and easier to re-enter from other shell contexts.

### Core design rule

The TUI remains the primary home of the experience. The CLI command surface is a companion interface for:

- fast capture from any shell
- scripting and automation
- shell aliasing and integration
- adding context without switching fully into the TUI

### Proposed binary

`flo`

### Initial command surface direction

#### Set the current thread

- `flo now "improving AIDX"`
- alias possibility: `flo set "improving AIDX"`

#### Create a branch under the current thread

- `flo branch "answering question from support"`
- `flo branch "reading article"`

#### Return to the parent thread

- `flo back`

#### Ask for current state

- `flo where`

#### Add a note to the current thread or branch

- `flo note "article may explain the support issue"`

#### Pause active work

- `flo pause`

#### Mark current work done

- `flo done`

#### Show active items

- `flo list`

### Suggested meaning of key subcommands

#### `flo now`

Sets or replaces the current top-level thread.

#### `flo branch`

Creates a lightweight branch beneath the current thread.

#### `flo back`

Returns attention to the parent thread and parks the active branch.

#### `flo where`

Prints the current thread, active branches, and recent context in a compact terminal-friendly format.

#### `flo note`

Appends raw thought to the current focus target.

### Example CLI session

```bash
flo now "improving AIDX for the component library"
flo branch "answering question from support"
flo branch "reading article"
flo note "article may explain the support issue"
flo back
flo where
```

### Relationship between CLI and TUI

The same state model should back both interfaces.

- The CLI is terse and action-oriented.
- The TUI is reflective and continuous.
- Both should read and write the same local store.

### Plain-language input remains valid

Natural language inside the TUI should still be the most forgiving path:

- `I'm improving AIDX`
- `reading article`
- `back to AIDX`
- `what am I currently working on?`

The CLI should complement this, not replace it.

## v1.1 — Optional local inference runtime

v1 remains useful with plain text input stored verbatim.

v1.1 adds an **optional local inference runtime** that refines raw captures into cleaner thread, branch, note, return, and query events.

This should not change the core contract of the app. The source of truth remains:

- raw capture first
- structured event second
- UI state updated after validation

### Why add this in v1.1

The app already works without a model. A local model should be an enhancement the user explicitly opts into by downloading a model.

Benefits of the optional inference layer:

- cleaner canonical thread titles
- more reliable branch detection
- short reply generation in the middle pane
- lightweight summary refresh
- future voice-to-text integration through the same event pipeline

### Architectural shape to borrow

Rather than treating inference as a helper function, Liminal Flow should treat it as a small subsystem.

The most useful architectural pattern to borrow is this shape:

- **core app state** — threads, branches, captures, scopes, replies
- **inference adapter layer** — verbatim mode or local model mode
- **CLI ops surface** — inspection, testing, model status, and future tooling
- **future bindings or integrations** — only if needed later

This keeps the application model separate from whichever inference backend is active.

### What to keep from this pattern

#### 1. A strict app core

Liminal Flow should own its own nouns and state transitions:

- capture
- thread
- branch
- reply
- inferred event

The inference backend should never become the source of truth.

#### 2. A backend adapter boundary

The app should support at least two modes behind one interface:

- **VerbatimEngine** — no model, raw text stored and used directly
- **LocalInferenceEngine** — local model refines text into structured events

Both engines should feed the same state machine.

#### 3. A model lifecycle

If local inference is enabled, the runtime should follow a simple lifecycle:

- discover installed model
- load model
- optionally warm model
- run inference
- validate output
- apply event

That lifecycle should stay explicit and observable.

#### 4. Platform-aware backend presets

The app-level contract should stay stable while the backend implementation can vary by platform.

Examples:

- desktop may support a small local LLM first
- voice features may arrive later per platform
- fallback verbatim mode should always exist

#### 5. Operational tooling as a first-class concern

A local inference feature needs boring, inspectable operations around it.

Future CLI ideas:

- `flo model status`
- `flo model install`
- `flo infer test`
- `flo trace`
- `flo doctor`

### What not to copy into Liminal Flow

Liminal Flow does **not** need to inherit a large hybrid routing or cloud orchestration worldview.

For this app, avoid in v1.1:

- policy-driven cloud routing
- multi-provider orchestration
- complex pipeline DSLs
- high-level abstraction layers that hide state transitions

The app should stay centred on local-first capture and refinement.

### Suggested v1.1 runtime layering

```text
Liminal Flow
├── App Core
│   ├── Thread / Branch / Capture / Reply state
│   ├── SQLite store
│   └── TUI + CLI surfaces
├── Inference Adapter
│   ├── VerbatimEngine
│   └── LocalInferenceEngine
├── Model Runtime
│   ├── local model loader
│   ├── warmup
│   ├── inference execution
│   └── output validation
└── Optional future inputs
    ├── voice transcription
    └── richer context ingestion
```

### How v1.1 would be used inside Liminal Flow

#### Without a model

Input:

> I'm improving AIDX for the component library

Behaviour:

- save raw text immediately
- optionally apply small deterministic heuristics
- show a minimal reply

#### With a local model

Input:

> I'm improving AIDX for the component library

Possible refinement:

- intent: set current thread
- canonical title: improving AIDX
- scope hint: component library
- reply: Current thread: improving AIDX

The same pattern applies for branch creation, return-to-parent actions, note capture, and current-state questions.

### Trust model for inference

Inference should remain:

- optional
- local-first
- reversible
- schema-validated
- conservative when uncertain

When the model is unsure, it should prefer adding a note rather than inventing structure.

### Product rule

The model is not the product. The event pipeline is the product.

## Local-first architecture sketch

### Core components

- Rust TUI app
- SQLite persistence
- small append-only session log
- optional local inference runtime
- optional shell helper

### Suggested responsibilities

#### App core

- routing inputs
- state transitions
- persistence
- rendering TUI

#### Inference layer

- intent classification
- title cleanup
- short reply generation
- lightweight summary refresh
- verbatim mode fallback when no model is installed

#### Context layer

- repo detection
- git branch reading
- OS/environment path discovery
- optional process observation

---

## Why this idea matters

The app solves a very ordinary but important problem:

People leave work mid-thought and lose continuity.

A to-do list does not solve that. A full notebook often adds too much friction. A sidecar can solve it by being:

- immediate
- local
- short-form
- terminal-native
- present-tense

It becomes a tiny continuity engine for active work.

---

## One-sentence summary

**Liminal Flow is a local-first terminal workspace sidecar that captures what the user is doing right now, tracks lightweight branches of attention, and answers brief natural-language questions to restore continuity when they come back.**

