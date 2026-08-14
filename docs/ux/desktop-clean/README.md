# Liminal Flow — Clean Desktop Concepts (Series 3)

Third series of the desktop exploration. Series 1
([`desktop-mockups/`](../desktop-mockups/)) translated the existing TUI to
Tauri; series 2 ([`desktop-rethinks/`](../desktop-rethinks/)) explored
expressive metaphors. This series starts from the ground up again but stays
**serious**: conventional, legible UI structures — panes, columns, tables —
executed with restraint. Any of these could ship to a professional audience
without explanation.

Each mock-up is a self-contained annotated HTML file — open in any browser.

| # | Concept | Structure | One-line thesis |
|---|---------|-----------|-----------------|
| 1 | [FLOW / QUEUE](01-queue.html) | Nav · list · inspector (3-pane) | An operator's console: a fixed Now block, an explicitly ordered queue, single-key actions |
| 2 | [FLOW / DAY](02-day.html) | Time column · context panel | A calendar that fills itself in — attention recorded as blocks, gaps kept honest |
| 3 | [FLOW / BOARD](03-board.html) | Four fixed kanban columns | Personal kanban with an enforced WIP limit of one; swaps are explicit and fair |
| 4 | [FLOW / REVIEW](04-review.html) | Stat tiles · heat grid · table | A read-only weekly close: four numbers, the sessions ledger, loose ends demanding decisions |

## Shared design language

- **Surfaces, not glows.** Flat panels (`#0e1320` on `#0a0d14`), 1 px hairline
  borders, one radius scale. Color appears only as meaning: green = active,
  amber = held/attention, violet = waiting/blocked, blue = interactive.
- **Type does the hierarchy.** UI text 13–14 px; metadata in a mono face at
  10–11 px; exactly one large number or title per view. No gradients inside
  the working UI (the brand gradient stays in marketing).
- **Keyboard-first, mouse-complete.** Every action shows its key; every key
  has a visible button equivalent. Selection drives detail panes; nothing
  opens a modal.
- **One capture input per view,** always plain text, always bottom of the
  frame, with deterministic placement rules (note → current focus; new item →
  queue/Next).

## The four, as a system

These aren't competitors so much as **rooms of one house**, and they map onto
the same store:

- **Queue** is the working view (threads → queue items, branches → sub-tasks,
  ordering is new but trivial to persist).
- **Day** is the log view (a direct rendering of session events).
- **Board** is Queue re-projected for people who think in columns — same
  data, a WIP-1 constraint the core Five Rules already imply.
- **Review** is the weekly read-only aggregation of the events table.

A credible v1 desktop app is Queue + Day as two tabs, with Review generated
weekly, and Board as an optional projection.

## Data-viz notes (Review)

- Sessions-table category marks use a categorical palette validated for the
  dark surface `#0e1320` (lightness band, chroma floor, CVD ΔE, contrast):
  `#4a8ee8` / `#cb7a1c` / `#9678ea` / `#f43f5e`, always paired with the text
  label — never color alone.
- The focus-by-hour grid is a single-hue sequential ramp (blue, four steps)
  with a legend; magnitude bars in the table use one hue for all rows.
- Stat tiles carry plain-language deltas instead of sparkline decoration.
