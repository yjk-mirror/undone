# Undone — Design System

## Direction

**"Evening Reader"** — a literary reading app that takes itself seriously and has choices.
Not atmospheric-gothic (that's Fallen London's lane). Not clinical-modern. The prose is the
primary surface; everything else is periphery. The interface should disappear.

Research basis: Choice of Games (default Palatino → Georgia serif stack), Instapaper (narrow
column, large margins), Degrees of Lewdity (block-style stat widgets), iA Writer (65ch column,
1.5 line height), NN/G dark mode research (light default, dark by choice), IF community
consensus (avoid pure black/white, astigmatism affects 50% of users).

---

## Three Modes

All clean. Same structure, different register. Never pure #000000 or #FFFFFF.

### Warm Paper (light, default)

| Token | Value |
|---|---|
| `--ground` | `#F7F2E8` |
| `--page` | `#FDFAF4` (prose panel — slightly brighter than ground) |
| `--page-raised` | `#F0EBE0` |
| `--sidebar-ground` | `#EDE8DC` |
| `--ink` | `#1C1814` |
| `--ink-dim` | `#5A5248` |
| `--ink-ghost` | `#8C8078` |
| `--seam` | `rgba(28, 24, 20, 0.10)` |
| `--lamp` | `#B07030` |
| `--lamp-glow` | `rgba(176, 112, 48, 0.12)` |

### Sepia (warm, highest reading comfort for long sessions)

| Token | Value |
|---|---|
| `--ground` | `#F0E8D0` |
| `--page` | `#F5EDD8` |
| `--page-raised` | `#EBE3CC` |
| `--sidebar-ground` | `#E6DCCA` |
| `--ink` | `#2C200E` |
| `--ink-dim` | `#6B5840` |
| `--ink-ghost` | `#9A8870` |
| `--seam` | `rgba(44, 32, 14, 0.10)` |
| `--lamp` | `#A06820` |
| `--lamp-glow` | `rgba(160, 104, 32, 0.12)` |

### Night (dark)

| Token | Value |
|---|---|
| `--ground` | `#141210` |
| `--page` | `#1A1816` (prose panel — lighter than ground, the page is lighter than the room) |
| `--page-raised` | `#201E1C` |
| `--sidebar-ground` | `#111008` (slightly darker than ground) |
| `--ink` | `#E8DDD0` |
| `--ink-dim` | `#A89D90` |
| `--ink-ghost` | `#6E6560` |
| `--seam` | `rgba(232, 221, 208, 0.08)` |
| `--lamp` | `#C08040` |
| `--lamp-glow` | `rgba(192, 128, 64, 0.12)` |

---

## Token Reference

| Token | Role |
|---|---|
| `--page` | Prose panel surface (the primary reading surface) |
| `--page-raised` | Elevated surfaces (dropdowns, modals) |
| `--ground` | Outer / base surface |
| `--sidebar-ground` | Sidebar surface (distinct from page) |
| `--ink` | Primary text |
| `--ink-dim` | Secondary / supporting text |
| `--ink-ghost` | Muted / metadata / disabled text |
| `--seam` | All borders and dividers |
| `--lamp` | Interactive accent — buttons, links, focus rings |
| `--lamp-glow` | Interactive hover/active background tint |

---

## Typography

**Prose font stack:** `Literata, Palatino, Georgia, serif`
- Literata (variable, Google Fonts) — purpose-built for long-form screen reading, higher
  ascenders than Georgia, better hinting for modern displays, free
- System fallback: Palatino (warm, widely available) → Georgia → generic serif

**UI chrome font stack:** `system-ui, -apple-system, sans-serif`
- Sidebar labels, stat values, button text — functional, not literary

**Prose defaults (user-configurable):**
- Size: 17px (range 14–22px)
- Line height: 1.5 (range 1.3–1.8)
- Max column width: 65ch (research-backed sweet spot — iA Writer, Baymard, Instapaper)
- Alignment: left (never justified — causes uneven spacing on screen)

**UI chrome:** 13px, not configurable

---

## Layout

Two-panel, horizontal split:
- **Left (prose column):** `flex-grow: 1`, `max-width: 65ch`, scrolls independently
- **Right (sidebar):** fixed `280px`, sticky (does not scroll with content)

Prose column structure (top to bottom):
1. Scrollable story text area (`flex-grow: 1`)
2. Choices bar (fixed height `min: 64px`, `flex-row`, wraps)

Sidebar structure (top to bottom):
1. PC name display (signature element)
2. PC stats panel (block-style widgets)
3. NPC panel (conditional — only when NPC in scene, separated by `--seam` border)

---

## Signature Element — PC Name Display

The PC's active name is **not a label** — it shifts with femininity score (the three-name
system). It sits alone at the top of the sidebar:

- Font size: 18px
- Font weight: 300 (light — deliberately lighter than the stat labels below)
- Color: `--ink`
- Bottom margin: 16px before the first stat group
- No heading treatment, no icon, no "Name:" label prefix

When the name changes (femininity threshold crossed), it updates in place. No animation.
The sidebar simply shows who the PC is right now.

---

## Choice Buttons

The most important interactive element. These are decision points, not app buttons.

- Min height: **48px** (44px Apple HIG minimum + breathing room)
- Padding: `12px 20px`
- Border: `1px solid --seam` at rest → `1px solid --lamp` at hover/focus
- Background: transparent at rest → `--lamp-glow` at hover/focus
- Text color: `--ink`
- Border radius: 4px
- Font: UI chrome stack, 15px

**Number prefix:** Every button shows its keyboard shortcut as a prefix:
- `1·`, `2·`, `3·` etc. in `--ink-ghost` at rest, `--ink-dim` at hover
- Always visible — players discover keyboard shortcuts without a tutorial

**All five states required:**
1. Default: transparent bg, `--seam` border
2. Hover: `--lamp-glow` bg, `--lamp` border
3. Focus (keyboard): same as hover + `2px solid --lamp` outline with `2px` offset
4. Active/pressed: slightly darker than hover
5. Disabled: `--ink-ghost` text, no border change, no pointer change

**Keyboard navigation:**
- Tab / Shift-Tab: cycle through buttons
- Enter or Space: activate focused button
- Number keys 1–9: activate choice by position (1 = first button, etc.)
- No hover-to-reveal patterns — everything visible at rest

---

## Stats Sidebar — Block Style

Research finding: list-style stats cause too much vertical scrolling (Degrees of Lewdity
redesign, Fallen London player complaints). Use block-style widgets.

Each stat group:
- Label: `--ink-ghost`, 12px, uppercase, 0.5px letter-spacing
- Value: `--ink`, 13px
- Row height: 28px
- Space between label and value: `flex justify-between`
- Groups separated by 8px gap (not dividers)

Stat groups for PC sidebar (in order):
1. Name (signature element — separate, see above)
2. Core stats: Femininity, Money, Stress, Anxiety
3. State: Arousal, Alcohol
4. (Future) Relationship status

Progressive disclosure: summary visible by default, detail on interaction (future).

---

## Depth Strategy

**Borders-only. No shadows.**

The seam between prose column and sidebar: single hairline at `--seam`.
Higher elevation (dropdowns, modals): `--seam` at higher opacity (`0.20`) on a
`--page-raised` background.

Craft whispers. Nothing should jump out.

---

## Spacing Scale

Base unit: 8px

| Name | Value | Use |
|---|---|---|
| Micro | 4px | Icon gaps, tight internal labels |
| Component | 8–16px | Button padding, internal widget spacing |
| Section | 24px | Between stat groups, between panels |
| Major | 48px | Between distinct layout regions |

---

## Border Radius

| Element | Radius |
|---|---|
| Buttons, inputs | 4px |
| Panels, sidebar | 0px (flat — fits literary register) |
| Modals | 6px |

---

## UserPrefs — Player-Configurable

The following must be stored and applied at runtime:

```rust
pub struct UserPrefs {
    pub mode: ThemeMode,       // Light | Sepia | Dark
    pub font_family: String,   // default: "Literata"
    pub font_size: u8,         // 14–22, default: 17
    pub line_height: f32,      // 1.3–1.8, default: 1.5
}

pub enum ThemeMode {
    Light,
    Sepia,
    Dark,
}
```

Persisted to disk alongside save files. Applies immediately on change (reactive signal).

---

## What This System Does NOT Include (yet)

- Markdown rendering in prose (pulldown-cmark → floem RichText — planned, separate task)
- Settings panel UI (planned, separate task)
- Mobile layout (desktop-only for now)
- Scroll-to-bottom on new prose (nice-to-have)
- Character creation screen
- Save/load UI
