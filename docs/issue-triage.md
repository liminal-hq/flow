# Issue Triage

_Last updated 2026-03-17._

This document groups the current open issues into working triage buckets so we can review related work together and leave notes in one place.

## TUI bugs and input handling

Issues that look like current behaviour defects or interaction regressions in the terminal UI.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| ~~#33~~ | `bug`, `tui` | Command parsing doesn't stop after command is entered | **Closed** — fixed in PR #40, tech debt cleaned up in PR #35 |
| ~~#32~~ | `bug`, `tui` | Terminal suspend doesn't work in the TUI | **Closed** — fixed in PR #40, refined in PR #35 |
| #34 | `enhancement`, `tui` | Text isn't selectable for copying in the Notes section or the input box | Polish next |
| #37 | `notes` | Navigate back to past comments in the Capture input | Polish next |

### Notes

- #32 and #33 are resolved. PR #40 shipped the initial fixes; PR #35 rebased onto that, added a comprehensive tech debt cleanup (data-driven command parsing, unified key handling, palette correction workflow, review fixes), and closed both issues.
- #34 and #37 are the next polish items — neither is a broken-path blocker but both improve daily usability.

## CI and infrastructure

Issues related to build pipeline, branch protection, and developer workflow.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #41 | `ci`, `infrastructure` | Set up branch protection rules on main | Do next |

### Notes

- The CI pipeline landed in PR #42 (format, clippy, test, PR summary jobs). The pipeline spec is documented at `docs/ci.md`.
- #41 is the follow-up: enable branch protection on `main` requiring the `format`, `clippy`, and `test` status checks to pass before merge. This is a manual GitHub settings change with a step-by-step walkthrough in the issue.
- Once #41 is done, the basic CI loop is complete. Future additions (`cargo audit`/`cargo deny`, coverage) are tracked in the CI spec as deferred items.

## Notes workflow and richer attachments

Issues focused on note management, richer note capture, and content attached to notes.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #27 | `tui`, `notes` | Add ability to manage notes | Defer until core TUI fixes land |
| #38 | `enhancement`, `tui`, `notes` | Save pictures to notes | Later feature |

### Notes

- #27 looks like foundational notes workflow polish, while #38 is a richer-capability follow-on that probably depends on the management model being clear first.
- Suggested sequencing: define how notes are reviewed, edited, and organised before expanding note payloads to include pictures.

## TUI workflow enhancements

Interaction improvements in the TUI that are not obvious bugs.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #39 | `enhancement`, `tui` | Resize panels in TUI with mouse | Nice to have |

### Notes

- #39 feels like late-stage workflow polish: high user-visible value, but probably best after the current interaction model is stable.

## Context curation and inference planning

The main planning cluster for inference-related context selection and assembly. This includes the umbrella issue and the design work that appears to feed it.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #1 | `epic` | v1.1 — Optional local inference runtime | Defer |
| #21 | `enhancement`, `core`, `store`, `cli`, `tui`, `inference` | Add context curation for inference inputs | Defer |
| #22 | `enhancement`, `core`, `store`, `inference` | Design persistence for context-curation state | Defer |
| #23 | `enhancement`, `core`, `context`, `inference` | Design inference context-assembly rules | Defer |
| #24 | `enhancement`, `cli`, `tui`, `inference` | Design CLI and TUI controls for context curation | Defer |

### Notes

- The org polish plan explicitly says to defer the entire inference epic while the product is still addressing TUI behaviour.
- #21 is the umbrella delivery issue; #22 through #24 are prerequisite design work.
- This category remains deferred during current release-focused work.

## Inference engine and runtime implementation

Issues for the core inference abstractions, adapter pipeline, crates, and runtime plumbing.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #2 | `core`, `inference` | Define InferenceEngine trait and request/response types | Defer |
| #3 | `core`, `inference` | Implement VerbatimEngine | Defer |
| #5 | `core`, `inference` | Integrate inference adapter into event pipeline | Defer |
| #6 | `core`, `inference` | Add output validation for inference results | Defer |
| #7 | `core`, `inference` | Add prompt versioning support | Defer |
| #8 | `inference` | Create liminal-flow-infer crate | Defer |
| #9 | `model` | Create liminal-flow-model crate (local model runtime) | Defer |
| #10 | `cli`, `model` | Add flo model status CLI command | Defer |
| #11 | `cli`, `inference` | Add flo infer test CLI command | Defer |
| #4 | `store` | Add inference config flags to config.toml | Defer |

### Notes

- The entire inference/runtime track remains deferred per the org polish plan.
- Suggested sequencing when it resumes: trait/contracts (#2), minimal engine path (#3, #8), pipeline integration (#5), then validation/versioning/config/status tooling (#4, #6, #7, #10, #11).

## Workflow model and lifecycle design

Issues that affect Flow's core user model and lifecycle semantics beyond the inference feature set.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #14 | `enhancement`, `core`, `cli`, `tui` | Reassess whether branches need a neutral state | Defer |
| #15 | `enhancement`, `core`, `cli`, `tui` | Design undo and redo for Flow lifecycle actions | Defer |

### Notes

- #14 is a prerequisite semantics question; #15 depends on those lifecycle rules being clear enough to reverse safely.
- Both sit below the immediate bug-fix and polish tracks.

## Test coverage

Issues specifically about adding or improving tests.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #12 | `testing`, `inference` | Add inference-specific tests | Defer with inference stream |

### Notes

- #12 becomes urgent once inference contracts and validation rules start landing.
- The workspace currently has 95 tests across all crates (up from ~85 before the tech debt cleanup).

## Untriaged or unclear

Issues that need clarification before they can be placed confidently into a delivery stream.

| Issue | Labels | Summary | Triage |
|---|---|---|---|
| #36 | none | Remindes? | Clarify first |

### Notes

- #36 likely needs a rewrite or expansion before it can be prioritised.

## Action summary

| When | Issue | Area | Summary | Status |
|---|---|---|---|---|
| ~~Now~~ | ~~#32~~ | ~~TUI bugs~~ | ~~Terminal suspend doesn't work in the TUI~~ | **Closed** (PR #40, #35) |
| ~~Now~~ | ~~#33~~ | ~~TUI bugs~~ | ~~Command parsing doesn't stop after command is entered~~ | **Closed** (PR #40, #35) |
| Now | #41 | CI and infrastructure | Set up branch protection rules on main | Open |
| Next | #34 | TUI polish | Text isn't selectable for copying | Open |
| Next | #37 | TUI polish | Navigate back to past comments in Capture input | Open |
| Next | #27 | Notes workflow | Add ability to manage notes | Open |
| Next | #39 | TUI enhancements | Resize panels in TUI with mouse | Open |
| Later | #38 | Notes workflow | Save pictures to notes | Open |
| Later | #14 | Lifecycle design | Reassess whether branches need a neutral state | Open |
| Later | #15 | Lifecycle design | Design undo and redo for lifecycle actions | Open |
| Clarify | #36 | Untriaged | Remindes? | Open |
| Defer | #1, #2–#12, #21–#24 | Inference and context curation | Inference epic, design, runtime, and test work | Open |
