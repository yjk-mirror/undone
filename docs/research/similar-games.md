# Research Notes: Similar Games — Design Patterns

*Session: 2026-02-22*

---

## UI / UX Patterns

### Story Text Presentation

**Scrollable continuous feed** (most common — DoL, Fallen London, SugarCube):
New content appends to bottom. Player scrolls up to re-read. Web-native default.

**Column-that-tumbles-up** (Disco Elysium):
New text appears at bottom, old text recedes upward. Inspired by Twitter feed.
Bottom-right placement mirrors where PC users already focus (system tray area).
Eye-tracking data confirms ~60% focus lower-right. Not applicable to turn-structured games.

**Beat-by-beat conversational** (80 Days / inkle):
Short exchanges, player taps to advance each beat. Feels participatory, not just reading.
Principle: *reduction* — remove everything that doesn't earn its place.
Most applicable to games with tight conversational scenes.

**Page-by-page** (Choice of Games / ChoiceScript):
Whole text block, then choices, then next block. Kindle/iBooks minimalism.
No competing chrome. Sacrifices ambient stat awareness.

**For Undone:** Turn-structured life sim → lean toward ChoiceScript (block per scene beat)
with fixed-width column and short paragraph chunking.

---

### Choice / Action Presentation

**Explicit buttons below scene text** (ChoiceScript, Fallen London):
Clear affordance — player always knows where to look. Visual separation from prose.

**Inline hyperlinks** (parser-adjacent Twine):
Creates scanning burden — player must read to find choices. Not right for charged social choices.

**Sequential micro-decision escalation** (80 Days):
"Approach... go closer... talk to him." Each choice narrows possibility space incrementally.
Feels like conversation. Requires tight authoring control.

**Fallen London card anatomy:**
Title + icon + body prose + branch buttons + "Perhaps Not" back button.
Colour-coded border = rarity/permanence signal. Most information-dense single interaction unit.

**For Undone:** Explicit labelled buttons below scene text. Consistent with Newlife model.
Tooltip on hover shows `detail` field. "Perhaps Not" / back equivalent worth considering.

---

### Stat Display

**Permanent sidebar** (Fallen London, DoL, Newlife):
PC stats always visible. Changes notified as inline toast or report. Sidebar is informational, not interactive.
DoL: body-state sprite + stat bars (stress, trauma, hunger, cold) + location + clothing.
Newlife: character attributes + current NPC info in side panel.

**Hidden behind tab** (Choice of Games):
Separate "Show Stats" screen. Prose narrates consequence ("Persuasion just increased").
Works when prose carries informational load. No ambient awareness.

**Diegetic inline** (Disco Elysium):
Skills speak as voices in dialogue column, colour-coded. Stat check shows inline before option.
Beautiful. Requires 24 distinct skill personalities. Not applicable at smaller scale.

**Tooltip supplement** (universal):
Hover on stat name/check = tooltip with current value + description. Secondary mechanism only.

**For Undone:**
- Fixed sidebar for PC stats (always visible)
- NPC info: contextual sub-panel, only when NPC is present in scene
- Trait/skill gains: brief inline report after scene text; sidebar reflects new value

---

### Handling "Walls of Text"

1. **Beat structure** — keep each unit short; player action between beats provides pacing
2. **Short paragraphs** — 2–4 sentences max, clear whitespace between
3. **Progressive reveal / visual differentiation** — Disco Elysium: each skill voice is a styled entry
4. **Prose narrates mechanical consequence** — ChoiceScript: reduces need for chrome
5. **Column width discipline** — 45–75 chars per line (66 ideal). Always constrain, never full-width.

**For Undone:** Constrain prose column to ~65–70 chars at chosen font size. floem default line
spacing is tight for extended reading — needs explicit tuning.

---

### Typography

| Parameter | Recommended Value |
|---|---|
| Body text size | 16–18px equivalent |
| Line height | 1.45–1.6× font size |
| Max line measure | 65–70 chars (~30–36em) |
| Paragraph spacing | ≥1× line height |
| Font family | Serif = literary/British register; sans = modern clarity |

Fallen London: serif for Victorian literary feel — deliberate tone signal.
80 Days: refined serif, consistent with late-Victorian premise.
DoL: system sans-serif, pragmatic.

**For Undone:** Serif fits British social-realist register. One body family, not mixed.
floem `RichText` supports font size + colour; font family via eframe font data.

---

### Games Praised for UI

| Game | Praise | Key Innovation |
|---|---|---|
| **80 Days** | IGF, BAFTA nominations | Beat-by-beat reduction, globe nav |
| **Disco Elysium** | Novelty + integration | Skill-as-voice, Twitter-column layout |
| **Fallen London** | Information management | Quality sidebar + card deck model |
| **Choice of Games** | Text-first minimalism | E-reader model, no chrome |
| **Degrees of Lewdity** | Polished for its genre | Body sprite that reflects world state |

---

### Summary: Recommended Patterns for Undone

| Dimension | Pattern | Reference |
|---|---|---|
| Text presentation | Fixed-width column, continuous scroll, paragraph-chunked | Fallen London, DoL |
| Beat pacing | Short paragraphs + inline consequence reporting | 80 Days, ChoiceScript |
| Choice display | Explicit labelled buttons below scene text | ChoiceScript, Fallen London |
| PC stat display | Fixed sidebar, always visible, changes notified inline | Fallen London, DoL, Newlife |
| NPC stat display | Contextual sub-panel when NPC present | Fallen London card model |
| Text column width | ~65–70 chars, not full-width | UX research consensus |
| Font | Serif for literary tone; 16–18px; 1.5× line height | Tone + readability |
| "Walls" solution | Short paragraphs + beat structure + explicit column width | 80 Days, ChoiceScript |
| Mechanical changes | Brief inline report after scene text | Fallen London quality reports |

---

## Character Creation Patterns

### Format Spectrum

| Game | Format | Duration | Immersion | Key mechanic |
|---|---|---|---|---|
| Newlife | Separate form | 5–10 min | Low | Trait point-buy; male persona → female appearance |
| Degrees of Lewdity | Separate form | 3–7 min | Low | Background selection (only mechanically weighted choice) |
| Free Cities | World config form | 10–20 min | Very low | Not a life sim — world rules, not personal identity |
| Lilith's Throne | Narrative-wrapped form | 15–30 min | Moderate | 8–10 appearance sub-menus; three-name system |
| Choice of Games | Fully narrative | 0 setup / 30+ min emergence | Highest | *Establishing choices* — stats emerge from early story decisions |

**Dominant pattern in this genre: separate form screen.** Fast to implement, easy to understand,
lets players configure a specific build before committing.

### Key Findings

**Newlife's two-phase approach:**
Male persona (minimal) → female appearance derived from it. Transformation is not in the
creation form — it is handled narratively once the game starts. The "before" self exists only
enough to establish the derivation. Community strategy guides exist for its trait system.

**Degrees of Lewdity's three-axis model:**
Sex / Gender identity / Presentation are three *independent* axes. Unusually sophisticated.
Enables wide play range without privileging any. Background selection is the single
mechanically weighted choice. Cosmetics can be changed later via in-world mirrors.

**Choice of Games' establishing choices:**
The most transferable insight. Stats are never shown at creation — they emerge from early
story choices ("how did you capture the prisoner?" = sets Brutality stat). Character forms
gradually over first 15–30% of the game. More immersive, more replayable, harder to author.

**Lilith's Throne's three-name system:**
Masculine / androgynous / feminine name variants that auto-swap based on current body
femininity. Architecturally elegant for a transformation game.
*Directly applicable to Undone's FEMININITY skill (0–100) architecture.*

### The Gap None of These Fill

The transformation premise creates a specific opportunity: the *before* self.
- Newlife: gestures at it (male persona → female appearance) but keeps it minimal
- Lilith's Throne: configures current body only, not who you were
- DoL: blank-slate orphan by design

**An Undone character creation that takes the "before" seriously — even briefly — would be
genuinely distinctive.** The male life, the pre-transformation persona, could be established
through narrative rather than a form. This is an unexplored design space.

---

## NPC Spawning / Pool Patterns

### Approaches Observed

**Newlife model** — small fixed pool, all at game start:
- ~8–10 male NPCs generated at character creation, persist the full run
- No new named characters enter the pool mid-game (special events may add 1 coworker etc.)
- Each gets 1 of 5 personality archetypes + trait draw from a weighted set
- Pool size *is* the relationship cap — you know everyone from week one
- Community custom NPC system (YAML files) injects authored characters into the pool
- **Result:** intimacy, permanence. Replayability depends on trait randomisation being rich.

**Degrees of Lewdity model** — three-tier architecture:

| Tier | Count | Creation | Persistence |
|---|---|---|---|
| Love Interests | 9 | Authored; unlock via story triggers | Full (unique per-char stats: jealousy, trust, etc.) |
| People of Interest | 18 | Authored, exist from start | Full (love + dominance) |
| Persistent NPCs | 7 | Procedural name + archetype on first encounter | Full |
| Generic NPCs | ∞ | Stateless; encounter-by-encounter | None |

Love Interests have "Character Progression Phases" — they evolve to suit player choices.
This is the most polished model but requires authoring 9 deeply-written characters.

**Lilith's Throne / Strive for Power** — authored quest companions + location-spawned procedural:
Named NPCs pre-seeded at fixed world locations; generic NPCs spawned on tile entry.
Mid-range complexity — authored cast ~15, unlimited background population.

**Free Cities** — fully procedural, no authored NPCs:
30–100+ slaves, all generated on acquisition. Only works because the game is about
managing a population aggregate, not individual relationships.

### Key Synthesis

**For Undone: the Newlife model is the right ancestor** — small fixed pool, full persistence,
personality archetypes driving scene branching. A life sim where you *know* these people.

**Critical open question:** should the spawner guarantee minimum diversity?
"At least one romantic type, at least one jerk type" — ensures the game is completable
and the full scene range is reachable on any run.

**Pool size suggestion:** 6–8 men + 2–3 female NPCs. Intimate enough that every
relationship has weight. Large enough for variety.

### Personality Archetype Question

The research confirms personality archetypes serve two engine-level purposes:
1. **Scene condition weighting** — e.g., CHARMING types more likely to offer umbrella
2. **Pool diversity guarantees** — spawner ensures archetypes are represented

Both require the engine to *reason about* personality, not just store it as an opaque string.
This argues for promoting the 5 core personalities to an engine-level enum, not data-driven ID.

---

## Personality System Patterns

*(To be researched)*
