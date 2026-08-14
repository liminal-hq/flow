# Liminal Flow — Ground-up Desktop Rethinks

Part of the desktop exploration. Where [`1-desktop-app/`](../1-desktop-app/) — the
chosen design — translated the existing CLI/TUI faithfully to a Tauri app, this set ignores
the current design entirely and asks: *if a working-memory tool were born on
the desktop, what could it be?*

Four deliberately divergent product concepts. Each is a self-contained,
annotated HTML mock-up — open in any browser.

| # | Concept | Core metaphor | One-line thesis |
|---|---------|---------------|-----------------|
| A | [FLOW / DESK](01-desk.html) | An infinite desk under lamplight | Focus is a *place*, not a status flag — spatial memory replaces every list |
| B | [FLOW / ONE](02-one.html) | A teleprompter for intention | Show exactly one thing, huge; three gesture verbs; lists are visited, never lived in |
| C | [FLOW / DIALOGUE](03-dialogue.html) | A colleague you narrate your day to | Plain language in, quiet structure out — the transcript *is* the database |
| D | [FLOW / GARDEN](04-garden.html) | A living night garden | Attention is water; neglect wilts kindly; done work blossoms into the meadow |

## What the four disagree about (on purpose)

- **Where structure lives.** Desk: in space. One: hidden, one card at a time.
  Dialogue: inferred from language. Garden: in organic form (stems, branches,
  roots).
- **How neglect is shown.** Desk: things cool and drift off the edge. One:
  nothing is shown at all. Dialogue: Flow mentions it, once. Garden: plants
  visibly wilt.
- **What "done" feels like.** Desk: a card slides off the horizon. One: a
  satisfying upward flick. Dialogue: a sentence ("done with that"). Garden: a
  blossom and a petal-fall.
- **How much the tool talks back.** From silent (One) to fully conversational
  (Dialogue).

## What all four agree on

1. **One active focus** — every concept enforces a single point of attention,
   each through its own physics (the lamp, the single card, "I'll hold your
   place", one bloom at a time).
2. **Capture is one gesture** — a whisper bar, a thought line, a composer, a
   watering can. Plain text, no dialogs, no forms.
3. **Interruptions bank their context** — the parent card peeks at the edge,
   the clip rides along, the colleague holds your place, the branch stays on
   the stem.
4. **No guilt mechanics** — nothing is overdue, scored, or streaked. Neglect
   decays gracefully into an archive (the edge, the tray, the transcript, the
   compost) instead of accusing you.
5. **The record is free** — everything renders from the same append-only
   event history; the user never does bookkeeping to get the picture.

## Tauri feasibility notes

- All four are single-webview apps; **Desk** wants a canvas/WebGL layer for
  smooth pan-zoom, **Garden** is pure SVG with light animation.
- **One** benefits most from Tauri niceties: compact fixed-size window,
  `always_on_top` option, global shortcuts for its three verbs.
- **Dialogue**'s intent parsing can start as simple verb heuristics (the
  existing title-normalisation rules generalise) and grow into a local model;
  the structured-card confirmations make wrong guesses cheap to correct.
- Any of the four can still share the CLI's SQLite store — the concepts
  discard the CLI's *interface*, not its data model. Threads, branches,
  captures and events map cleanly onto cards, stems, holds and roots.
