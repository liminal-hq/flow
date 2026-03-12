# Navigation Review

## Summary

The TUI currently has two competing focus models:

- the left-pane cursor selects an item
- the chat input, note target, and status pane follow the active thread or active branch instead

That split makes navigation feel inconsistent. The cursor looks important, but most actions ignore it.

The current visual language already does useful work:

- green highlights the active item
- bold blue highlights the focused selection

That distinction should stay. The tighter model should build on it rather than add more status copy to the layout.

## Current behaviour

### Left pane

- Up and Down move a visible selection cursor through threads and branches
- Enter toggles expand and collapse on the selected thread
- `r` resumes the selected thread or branch

Keeping Enter for expand and collapse makes sense. It allows the user to inspect a thread group without activating it.

### Input and notes

- plain text input is treated as a note
- notes attach to the active branch if one exists on the active thread
- otherwise notes attach to the active thread
- the selected row does not affect where the note goes

This is the part that feels most at odds with the selection model.

The cleaner direction is:

- capture stays targeted at the active item
- if the user wants to add notes to something else, they should activate it first
- the capture area should make the active note target obvious

### Status pane

- the right pane always shows the active thread
- it does not show details for the selected row
- recent notes are collected from the active thread and any active branch beneath it

### Parking and pause

- `/back` parks all active branches on the active thread and returns to the parent thread
- there is no explicit `park` action in the TUI
- pausing a thread pauses only the thread
- pausing does not park active branches beneath that thread

In this application, `park` should mean:

- the item remains live and visible
- it is no longer the current active focus
- it is intentionally set aside so the user can return to it later

For a branch, parked feels like "not finished, not dropped, not active right now".

### Resume

- resuming a branch activates that branch and its parent thread
- resuming a thread activates only the thread
- resuming a thread does not restore the most recently parked branch

This means park and resume are only partly defined as a pair in the current behaviour.

## Main inconsistencies

### 1. Cursor and action target diverge

The left pane suggests that the selected item is the current focus target, but notes and the status pane still operate on the active item instead.

This means the user can:

- move the cursor to a parked branch
- see that branch highlighted
- type a note
- have the note attach somewhere else

That is the biggest source of confusion in the current model.

### 2. Parking is hidden behind `/back`

The app supports the concept of parked branches, but it does not expose parking as a direct, explicit action in the TUI.

That leaves the user asking:

- how do I park this thing
- does pause park it
- does resume undo parking

The current answer is too indirect.

### 3. Pause leaves branch state ambiguous

A paused thread can still have an active branch in the current implementation.

That can look ambiguous at first because:

- the parent thread says paused
- a child branch may still say active

The more coherent interpretation is:

- paused means the whole thread stack is inactive
- child branches do not need a separate parked state change just because the parent paused
- branch state can remain as-is and still be treated as dormant while the parent thread is paused

That keeps pause lightweight and avoids extra status churn.

### 4. Status pane follows activity, not focus

The right pane acts like a status dashboard for the active thread rather than a detail view for the selected item.

That can be valid, but it conflicts with the presence of a strong selection cursor in the left pane.

The visual distinction between active and selected is already present in the screenshot and should remain colour-based rather than adding extra status lines that consume space.

### 5. Resume is not a full return-to-context action

Resuming a branch brings you back to that branch. Resuming a thread brings you back only to the thread.

If the user thinks of resume as "take me back to where I left off", the current thread resume behaviour feels incomplete.

## Current mental model in the code

Today the app behaves like this:

- selection is for navigation
- active state is for mutations and status
- `r` turns the selected item into the active item
- `/back` parks active branches on the active thread
- `pause` pauses the thread only

This model is internally understandable in code, but it is weaker in the interface because the user has to track both selection and active state at once.

## Recommended direction

The cleanest fix is to centre the experience on one rule:

**The selected item should guide inspection, while active state remains the source of truth for capture and current work.**

## Proposed tighter model

### Selection and focus

- the selected row is the current focus target
- the right pane should follow the selected item for inspection
- the input area should clearly state the active item that notes will attach to before submission

Examples:

- `Note on: improving AIDX`
- `Note on: answering question from support`

This preserves a more CLI-like flow:

- inspect one thing with the cursor
- activate it when you want to work on it
- capture naturally lands on the active thing

### Activation

- `r` resumes the selected parked thread or branch
- activating a branch also activates its parent thread
- activating a paused thread should restore the thread from where it left off
- the main open question is which branch should become active after resuming a paused thread

Enter should continue to expand and collapse groups rather than activate them.

The thread-resume rule should be deliberate because it defines what "resume" means in practice:

- if resuming a thread restores its most recently active branch, resume behaves like a true return-to-context action
- if the user resumes a specific branch inside a paused thread, that more explicit action should win

### Parking

- add an explicit `park` action in the TUI, such as `p`
- parking a selected branch should park just that branch
- pausing a selected thread should imply that the whole stack is inactive without rewriting child branch state
- `/back` can remain the higher-level "return to parent" command

An explicit `park` key is the right next step either way.

This suggests a cleaner meaning split:

- `park` is mainly for branches when stepping back to the parent thread
- `pause` is for putting the whole thread stack aside

### Status visibility

- keep the current visual distinction where active and selected are styled differently
- avoid adding extra status copy that increases space pressure in the layout
- let the right pane content and capture target do the explanatory work instead

## Recommended implementation order

1. Make the right pane follow the selected item and show the capture target clearly.
2. Add an explicit park action for branches.
3. Keep capture attached to the active item and make that target explicit in the input area.
4. Treat paused threads as making the whole stack inactive without changing stored branch state.
5. Resume paused threads from where they left off, restoring the most recently active branch by default unless the user explicitly resumes a different branch.

## Recommendation

This should be treated as a behaviour and interaction redesign rather than a small wording fix.

The strongest improvement is to unify:

- what is selected for inspection
- what the right pane describes
- what the capture input says it will target as the active item

That keeps the TUI closer to the CLI model, where capture feels more natural because it is implicitly targeted at the current active context.

It also gives the interaction model a cleaner split:

- selection is for looking
- activation is for working
- capture follows activity
- pause suspends the whole stack
- park is a branch-level "set this aside for now" action

## Implementation checklist

### Status and inspection

- [ ] Make the right pane follow the selected item rather than always showing the active thread
- [ ] Keep the current active-versus-selected visual distinction in the thread list
- [ ] Ensure the right pane still makes the active thread context easy to understand when inspecting a non-active item

### Capture targeting

- [ ] Add clear capture-target copy to the input area, based on the active item
- [ ] Keep plain text capture and `/note` targeted at the active item
- [ ] Verify that inspecting a non-active item does not change the note target until the user explicitly activates it

### Activation and resume

- [ ] Keep `Enter` for expand and collapse only
- [ ] Keep `r` as the explicit resume and activate action for the selected item
- [ ] When resuming a branch, activate its parent thread and restore that branch as active
- [ ] When resuming a paused thread, restore the thread from where it left off
- [ ] Restore the most recently active branch by default when resuming a paused thread, unless the user explicitly resumes a different branch

### Parking and pause

- [ ] Add an explicit park action for branches in the TUI, likely on `p`
- [ ] Define branch parking as "still live, not active, easy to return to"
- [ ] Keep `/back` as the higher-level return-to-parent action
- [ ] Treat paused threads as making the whole stack inactive without rewriting stored child branch state
- [ ] Ensure paused-thread rendering makes that implicit inactivity understandable in the UI

### State and persistence

- [ ] Store enough state to identify the most recently active branch for a thread
- [ ] Confirm whether existing timestamps are sufficient to restore branch context on resume
- [ ] Update reducer and persistence rules so TUI and CLI resume behaviour stay aligned

### CLI and TUI parity

- [ ] Review CLI semantics for `/back`, `pause`, and resume so the TUI does not drift from the command model
- [ ] Decide whether explicit branch parking should also exist as a CLI command
- [ ] Keep active-context capture behaviour consistent between the CLI and the TUI

### Documentation and verification

- [ ] Update `SPEC.md` with the revised navigation, pause, park, and resume semantics
- [ ] Update implementation docs if the navigation model materially changes the planned behaviour
- [ ] Add or update tests covering:
- [ ] selected-versus-active inspection behaviour
- [ ] capture targeting on the active item
- [ ] explicit branch parking
- [ ] paused-thread resume restoring prior context
- [ ] explicit branch resume overriding default thread-resume behaviour
