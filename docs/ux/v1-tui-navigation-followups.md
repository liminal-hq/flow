# Navigation Follow-Up

## Confirmed bugs

- Active-branch targeting looks inconsistent. `r` on a branch can report "already active" while capture still targets a different branch.
  - **Scott:** There should only be one active branch or thread at any time, this keeps the TUI and the CLI in sync and follows the mental model of working on one thing at a time.
  - ***Decision:*** Enforce a single active item invariant across thread and branch targeting.

- We likely have a broken active-branch invariant. More than one branch may still be stored as `active`, and note targeting is probably picking whichever active branch `find_active_for_thread()` returns first.
  - **Scott:** See above.
  - ***Decision:*** Treat multiple active branches as invalid state and normalize them automatically.

- Notes appearing under the wrong branch, or seeming to belong to the whole thread, is likely a symptom of the same targeting bug rather than only a rendering issue.
  - **Scott:** See above.
  - ***Decision:*** Fix targeting first, then re-check rendering with clean state.

- Active-branch highlighting is still not reliable enough if the underlying branch state is inconsistent.
  - **Scott:** See above.
  - ***Decision:*** Make highlighting depend on normalized single-active-item state.

## Recommended fixes

- Enforce exactly one active branch per thread at the data and transition layer.
  - **Scott:** Yes!
  - ***Decision:*** Do this first.

- Add a repair or normalization step on load or resume so older bad state is corrected automatically.
  - **Scott:** Yup
  - ***Decision:*** Add a lightweight normalization pass rather than leaving existing state broken.

- Make note targeting use the normalized active branch consistently in both TUI and CLI flows.
  - **Scott:** 100%
  - ***Decision:*** Capture must always follow the one true active item.

- Improve branch-note rendering in the Status pane with separators and timestamps.
  - **Scott:** <3
  - ***Decision:*** Add visible note boundaries and compact timestamps.

- Add scrolling to the Status pane so longer note histories are actually readable.
  - **Scott:** How will this look?
  - ***Decision:*** Add keyboard scrolling with a small offset-based viewport. Keep the current layout and scroll content vertically when it overflows.
  - **Scott:** Any way to detect the mouse's current position and scroll the pane if the mouse wheel is used while hovering over it?
  - ***Decision:*** If mouse support is practical in the current terminal stack, pane-aware mouse-wheel scrolling is desirable. The hovered pane should scroll instead of applying wheel input globally.

- Add scrolling to the Threads pane when the thread list exceeds the available height.
  - ***Decision:*** Threads should support the same pane-aware scrolling model as other scrollable panes.
  - ***Decision:*** Keyboard and mouse-wheel behaviour should feel consistent across Threads, Status, and Help.

- Add scrolling to the Help pane for smaller terminal sizes.
  - **Scott:** Yes, this is a must-have for smaller screens.
  - ***Decision:*** Add Help scrolling in the same pass as Status scrolling.
  - **Scott:** Same question about mouse wheel support here. If the mouse is hovering over the Help pane, it should scroll that pane instead other panes.
  - ***Decision:*** Use the same pane-aware mouse-wheel rule for Help if mouse events are available and reliable.

## Branch state model

- `parked` currently feels like a more intentional, longer-lived state.
- A `neutral` branch state could mean "not active, but not explicitly parked".
- Adding `neutral` would complicate the CLI model immediately, because `back`, `branch`, `resume`, and `where` would all need clear behaviour for `neutral` versus `parked`.
- My current recommendation is to avoid adding `neutral` in this first pass. First make `active` and `parked` actually consistent, then revisit whether a third branch state is still needed.
  - **Scott:** Ok, you're on to something here. I think we can get away with just `active` and `parked` for now.
  - ***Decision:*** Defer `neutral`. Keep the model to `active` and `parked` for now.

## Keybindings and workflow

- `d` for marking the selected item done feels like a good Normal-mode shortcut.
- `u` and `U` for undo and redo are appealing, but they imply a much larger event-reversal model rather than a small keybinding addition.
  - **Scott:** Probably :)
  - ***Decision:*** `d` is a good short-term UI improvement. Undo and redo need explicit design work.

- My current recommendation is to treat `d` as near-term, and `u` / `U` as a separate feature after the core navigation model is stable.
  - **Scott:** What do we have to think through? Let's tread deleted as tombstones and a simple undo stack for now (maybe a note for undo only works on delete), and we can open an issue to track a more robust undo/redo model after we have the basics working.
  - ***Decision:*** Open a detailed issue for undo and redo design early, but do not implement it in this navigation fix batch.
  - **Scott:** If you're implementing `d` for marking items done then we should use tombstones.  However, is done deleteing them or leaving the branches around so it kinda funtions as a "done" list? I think it would be nice to have the option to review done items, but we can also just delete them and add a "recently done" list in the future if we want that functionality. then repoping them with `r` makes sense adn we don't need undo/redo for now.
  - **Scott:** Let's keep the done items in the list in a different colour and marked done.  there can be a key to hide/show done items if the list gets too cluttered. but it is nice to see where you've been! `r` can mark them as active again and bring them back to life if you need to re-open something.
  - ***Decision:*** Do not make `done` the long-term list-pruning mechanism. Keep `done` as a completion marker, and introduce `archive` as the way to remove finished items from the main working surface.

## Archive model

- The thread and branch lists will eventually fill up if `done` items stay visible forever.
  - **Scott:** one other issue, the list will fill up! haha so even done threads should go away.
  - ***Decision:*** Add an archive concept to clear completed items out of the main working view.

- Archiving feels like a cleaner solution than overloading `done`.
  - **Scott:** Archiving! that's one solution, manual archive a thread/branch, or archive all.
  - ***Decision:*** Split completion from cleanup:
    - `done` means finished
    - `archive` means remove from the main working surface

- Archiving should work at both the item level and the batch level.
  - ***Decision:*** Support manual archive for a selected thread or branch, plus bulk archive actions such as archive-all-done later.

- `done` items can remain visible briefly or by default until archived, but archive becomes the real pressure-release mechanism for list growth.
  - ***Decision:*** Keep archive as the main long-term answer for list clutter rather than making `done` disappear immediately.

## CLI adjustments

- The CLI should stay aligned with the same lifecycle model as the TUI.
  - ***Decision:*** Any archive behaviour added to the TUI should have a corresponding CLI command shape.

- If we add archive support, the CLI likely needs explicit archive commands.
  - ***Decision:*** Plan for commands such as:
    - `flo archive`
    - `flo archive --all-done`
    - possibly a branch-specific archive shape if branch targeting needs to be explicit

- `flo where` and `flo list` will need clear rules for whether done items, archived items, or both are shown.
  - ***Decision:*** Archived items should be excluded from the default working views. If we later expose archived history, that should be opt-in.

- `r` reviving done items in the TUI has a CLI equivalent question.
  - ***Decision:*** If done items remain restorable, the CLI should support reopening or resuming a done item without requiring manual database edits.
  - **Scott:** Let's add this now so it is complete and inline with the TUI.
  - ***Decision:*** Include CLI support for reviving done items in the same lifecycle pass as the TUI.

## Suggested order

- First fix the active-branch invariant and note-targeting bug.
- Then add note separators and timestamps.
- Then add pane scrolling for Threads, Status, and Help.
- Then polish the TUI chrome with rounded corners.
- Then add done and archive lifecycle support in both TUI and CLI.
- Leave `neutral` and undo/redo until after the core branch model feels solid.
- **Scott:** As noted, let's skip `neutral` for now. I think the undo/redo model is pretty important to get right early on, so I would make sure there's a clear, detailed issue opened with the possibilities and design decisions to be made.
  - ***Decision:*** Keep undo and redo out of this implementation pass, but capture them in a dedicated design issue immediately after the navigation fixes land.

## Working plan

### Phase 1: Fix active targeting and state repair

- [x] Enforce one active branch per thread in TUI and CLI branch activation flows
- [x] Add a normalization pass when loading or resuming thread state
- [x] Make capture targeting read from the normalized active branch or active thread only
- [x] Verify active highlighting and capture target titles against the repaired state
- [x] Re-check branch-note ownership in the Status pane; notes are still appearing under sibling branches instead of only the branch they were added to

### Phase 2: Improve note readability

- [x] Show branch notes only on the branch they belong to
- [x] Add compact timestamps to notes in the Status pane
- [x] Add clear separators or spacing between notes so they are easy to scan

### Phase 3: Add pane scrolling

- [x] Add vertical scrolling to the Status pane
- [x] Add keybindings for scrolling while preserving existing navigation semantics
- [x] Add vertical scrolling to the Help pane for smaller terminals
- [x] Ensure the Help pane exposes scroll affordance clearly enough
- [x] Investigate pane-aware mouse-wheel scrolling for Threads, Status and Help
  - ***Tracked in:*** #16

### Phase 4: TUI chrome polish

- [x] Update the main TUI boxes to use rounded corners consistently
- [x] Check overlays and popups so rounded borders look intentional across Threads, Status, Capture, Help, and command palette surfaces

### Phase 5: Done and archive lifecycle

- [x] Add `d` as a Normal-mode shortcut for marking the selected item done
- [x] Define archive lifecycle rules for threads and branches
- [x] Add archive actions in the TUI
- [x] Design corresponding CLI archive commands
- [x] Allow `r` to revive a done item and make it active again
- [x] Add matching CLI support for reopening or resuming done items
- [x] Add a slash resume command so active focus can be changed from Insert mode without switching to Normal mode

### Phase 6: Undo and redo design

- [x] Open a detailed issue for undo and redo design
  - ***Tracked in:*** #14

### Phase 7: Model reassessment

- [x] Revisit whether a third branch state is still needed after the two-state model settles (open a detailed issue for the concept of `neutral`.)
  - ***Tracked in:*** #15
