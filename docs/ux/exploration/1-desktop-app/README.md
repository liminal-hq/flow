# Liminal Flow — Desktop Design (Tauri)

**This is the chosen design for the Flow desktop app.** Four screens for a
desktop companion to the `flo` CLI/TUI, targeting Tauri. Each screen is a
self-contained HTML file — open it in any browser. Annotated callouts on each
page explain the thinking. The build is tracked by the Flow Desktop epic on
GitHub, and `assets/` holds SVG renditions of each screen plus the epic's
dependency diagram for embedding in issues.

| # | Screen | What it covers |
|---|--------|----------------|
| 1 | [The Workbench](01-workbench.html) | The main window — streams sidebar, focus stack, notes river, capture bar |
| 2 | [Quick Capture](02-quick-capture.html) | Global-shortcut palette that floats over any app |
| 3 | [Companion & Tray](03-companion.html) | Always-on-top focus pill, tray flyout, "welcome back" resume card |
| 4 | [The Flow Map](04-flow-map.html) | Your day replayed as a braided river, rendered from the events table |

## Design principles

1. **The desktop app is a peer, not a replacement.** It reads and writes the
   same SQLite store (WAL mode) as the CLI and TUI. `flo now` in a terminal
   shows up in the desktop app within the existing 250 ms polling contract,
   and a click on *park* in the tray is indistinguishable from `flo park`.
   One brain, three faces.
2. **Same grammar everywhere.** Plain text is a note. `/` opens the identical
   slash-command palette with the same ranking rules as the TUI
   (name matches beat description matches). Muscle memory transfers 1:1.
3. **Ambient, not demanding.** Flow is a working-*memory* sidecar, so the
   desktop's job is glanceability: the titlebar breadcrumb, the companion
   pill, the state-coloured window ring and tray icon. Zero notifications by
   default — the only proactive surface is the opt-in "welcome back" card
   after system idle.
4. **Attention drawn as flow.** Threads render as streams; branches visually
   fork off their parents; the focus stack is a literal stack of cards; the
   Flow Map braids the whole day. The metaphor in the name becomes the UI.
5. **Visualisation as a receipt, not a form.** Everything in the Flow Map is
   a pure render of the append-only events table Flow already writes. The
   user never enters data to get the picture.

## Visual language

Derived from `assets/hero.svg`:

- Background: `#050507 → #0a1020` gradient, dark-first
- Brand gradient: `#ffaa40` (orange) → `#f43f5e` (rose) → `#a78bfa` (violet)
- Flow beams: `#5aa2ff` (blue); active focus: `#2ec66a` (green)
- State colours: green = active, amber = paused, violet = parked,
  green ring/tombstone = done
- Mono accents (`ui-monospace`) for timestamps, scopes, commands and hints —
  a deliberate nod to Flow's terminal-native roots

## Linux-first chrome

Flow is built Linux-first, and the desktop app wears GNOME/Adwaita chrome:
window controls on the right of the headerbar, a headerbar view switcher, and
Ctrl/Super key notation throughout (Ctrl+K for the palette, Super+Space for
Quick Capture). The SVG renditions in `assets/` reflect this; treat any
macOS-style chrome remaining in the HTML mock-ups as superseded.

There are exactly two kinds of bars in the design, and **no Flow window has a
menu bar**:

1. **App topbars** (Workbench, Flow Map): the shared Liminal HQ topbar
   pattern — **Spindle's `Topbar` component is the implementation
   reference** (window-control pattern originally from Threshold). Layout:
   brand mark, view switcher, contextual content over a drag region, then
   platform-appropriate window controls — on Linux, transparent buttons with
   thin line glyphs (minimise / maximise / close), Adwaita-style hover.
   Anything menu-like lives behind ⚙ in the topbar or the Ctrl+K palette.
2. **The GNOME shell top bar** appears only in scene-setting mock-ups (the
   Companion) because those depict the whole desktop — the tray indicator and
   its flyout hang from the shell bar, as GNOME intends. It is not Flow
   chrome.

Transient surfaces (Quick Capture, the Companion pill, the welcome-back card)
have no chrome at all — no headerbar, no controls, dismissed by Esc/blur.

## Navigating between screens

- The headerbar carries an Adwaita-style **view switcher** between the two
  full views: **Workbench ⇄ Flow Map**. Esc returns to the Workbench.
- The **Today rail** in the Workbench is a second, contextual route into the
  Flow Map (click the braid or ◎).
- **Quick Capture** and the **Companion pill** are transient windows, not
  views — they are summoned (Super+Space, always-on-top) and both offer an
  "open" action that raises the Workbench.
- **Ctrl+K** opens the palette from either view; commands there can jump
  views as well as act on the store.

## Window inventory & Tauri notes

| Window | Tauri specifics |
|--------|-----------------|
| Workbench | Frameless (`decorations: false`), custom titlebar as drag region, min ~980×640. State-coloured focus ring via translucent window + CSS. |
| Quick Capture | Transparent, undecorated, `always_on_top`, centered near top; summoned via `tauri-plugin-global-shortcut` (e.g. ⌥Space); hides on blur/Esc, never appears in the taskbar (`skip_taskbar`). |
| Companion pill | Tiny undecorated `always_on_top` window, draggable anywhere, position persisted; expands on hover. |
| Tray | `TrayIcon` with state-coloured icon + menu built from live store state; recent-streams items call the same resume logic as the TUI's `r`. |
| Welcome back | Small `always_on_top` card triggered by idle/unlock detection; entirely opt-in via config. |

Backend: the Tauri shell links the existing Rust crates directly (store,
model, context) rather than shelling out to `flo` — commands are `#[tauri::command]`
wrappers over the same core APIs the CLI uses, so the Five Core Rules
(single active thread, auto-pause, branches require a thread, back parks
branches, immutable events) are enforced in exactly one place.

## Deliberate non-goals

- No task-manager features (due dates, priorities, projects) — Flow tracks
  attention, not obligations.
- No productivity scoring or streaks. The Flow Map describes; it never grades.
- No cloud sync in this concept — the local SQLite store remains the single
  source of truth.
