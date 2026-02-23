# Undone — Design Decisions: Open Questions

*Session: 2026-02-22. Research-driven. See docs/research/similar-games.md for sources.*

---

## 1. Character Creation Flow

### Decision: Two-Phase Hybrid

**Phase 1 — The Before (narrative establishing choices)**

A brief scene, not a form. The player makes 3–4 choices about their male life through
action rather than configuration. These choices set the "before" data that the writing
engine uses for the rest of the game.

Example establishing choices:
- "What kind of work do you do?" → sets job_before, class signal
- "How do you spend your evenings?" → seeds a personality trait
- "When you picture yourself with someone, who do you see?" → sets before_sexuality

These choices feel like a ChoiceScript opening — you are briefly *living* the before self,
not filling in a form about him. Duration: 5–10 minutes. The transformation event closes it.

**Phase 2 — The After (configured form)**

Standard form for the current body. This is what the player plays as going forward:
- Name(s) — see three-name system below
- Race (current — may differ from before)
- Age (current — may differ from before)
- Figure, breast size
- Starting trait picks (influenced by Phase 1 choices)

**The Three-Name System (from Lilith's Throne)**

Three name slots: `name_fem`, `name_androg`, `name_masc`.
The active display name shifts based on FEMININITY score:
- FEMININITY 0–30 → `name_masc`
- FEMININITY 31–69 → `name_androg`
- FEMININITY 70+ → `name_fem`

The player fills all three, or just one (others default to the same value).
NPCs who knew the male persona use `name_masc` until given the new name in-scene.

### What the Engine Needs to Capture

New fields on `Player`:

```rust
pub before_age: u32,             // age at transformation
pub before_race: String,         // race before transformation
pub before_sexuality: Sexuality, // what the male persona was attracted to
pub name_masc: String,
pub name_androg: String,
pub name_fem: String,
```

`Sexuality` enum (engine-level, not data-driven — the engine reasons about this for
internal-conflict writing branches):

```rust
pub enum Sexuality {
    StraightMale,       // was attracted to women; now attracted to men = new territory
    GayMale,            // was attracted to men; now attracted to men = familiar desire, new position
    BiMale,             // was attracted to both; some attractions are familiar, some aren't
    AlwaysFemale,       // always_female=true; before_sexuality is not applicable
}
```

### Writing Principle: Continuity of Self

*This belongs in the pack's writing guide, not just the engine.*

The character does not reset. She carries her entire former life. The writing must use it:

- **Age delta:** Large deltas are *charged*. A 50-year-old in a 19-year-old's body has
  decades of professional, social, and bodily experience. Scenes involving condescension,
  inexperience assumptions, or social hierarchy should acknowledge this where possible.

- **Race delta:** If the before and after race differ, scenes involving racial dynamics
  are experienced from a new position with memory of the old one. This is one of the most
  under-addressed territories in the genre. Addressing it thoughtfully is distinctive.

- **Sexuality as a live question:** The internal experience of attraction should never be
  presented as settled. "You are now attracted to men" is not a scene. The scene is: the
  attraction exists, it has a texture, it intersects with everything you remember about
  being on the other side of it. Especially if the male persona was straight — the new
  attraction to men is not an explanation, it is a live question the character carries.

- **Internal conflict without resolution:** The game is not therapy. The character is not
  progressing toward self-acceptance as a mandatory arc. She is navigating. Some paths
  resolve; others stay open. The writing principle is *honesty*, not *arc*.

---

## 2. NPC Spawning / Pool Seeding

### Decision: Newlife Model with Diversity Guarantees

**Pool composition:**
- 6–8 male NPCs generated at game start, persist for the full run
- 2–3 female NPCs (friends, coworkers) seeded similarly
- No on-demand spawning of new named characters mid-game
  (special events may introduce 1 new NPC via a scene, but this is exceptional)

**Creation timing:** All at game start, in a seeding phase before week 1.

**Diversity guarantees:**

The spawner ensures minimum personality archetype representation:
- At least 1 ROMANTIC type (the game must be completable)
- At least 1 JERK type (tension source, required for some scene branches)
- At least 1 FRIEND type (platonic track)
- Remaining slots: weighted random from all archetypes

Without guarantees, some runs would lack the NPC types required to reach key scenes.

**Appearance + trait generation:**
- Appearance: random within trait-gated bounds
- Traits: drawn from a weighted pool, personality-gated (e.g., TRADITIONAL skews toward
  JERK archetype, SENSITIVE skews toward ROMANTIC)
- Names: drawn from a British names list in the base pack

**Custom NPCs (future):**
The custom NPC hook (YAML-defined authored characters that inject into the pool) is
out of scope for the NPC spawning session but should be designed as an explicit slot.
The pool has 1–2 reserved slots at the high end for custom/authored NPCs.

---

## 3. PersonalityId — Engine Enum vs. Data-Driven

### Decision: Engine Enum for Core 5, Data Extension Stays Open

**Core personalities as engine enum:**

```rust
pub enum Personality {
    Romantic,    // warm, attentive, pursues connection
    Jerk,        // selfish, crude, high-status-seeking
    Friend,      // platonic, reliable, low-romantic-pressure
    Intellectual, // curious, distant, attracted to wit
    Lad,         // social, laddish, peer-pressure driven
}
```

The engine reasons about personality for:
1. NPC action weight multipliers in scenes (ROMANTIC → higher weight on affectionate NPC actions)
2. Pool diversity guarantee logic in the spawner
3. Scheduling slot eligibility (some slots only fire when a JERK is in the pool)

**Data extension stays open:**
`PersonalityId` in `NpcCore` remains an interned string so packs can add personalities.
The engine's scheduling/spawning code special-cases the 5 core variants; custom personalities
get no engine-level scheduling logic but work fine for authored scenes that check them by ID.

The alternative (fully data-driven) was rejected because the engine *needs to reason about*
personality for spawning diversity and scene weighting — that logic can't live in pack data.

---

## 4. UI Design

*See docs/research/similar-games.md for full findings.*

### Recommended Layout

```
┌────────────────────────────────────┬───────────────────┐
│                                    │                   │
│  STORY TEXT                        │  PC STATS         │
│  (fixed-width column, ~65–70 chars)│  (always visible) │
│                                    │                   │
│  [Mechanical change reports here]  ├───────────────────┤
│                                    │  NPC PANEL        │
│                                    │  (contextual —    │
│                                    │  only when NPC    │
│                                    │  present)         │
│                                    │                   │
├────────────────────────────────────┴───────────────────┤
│  [ Action A ]  [ Action B ]  [ Action C ]              │
└────────────────────────────────────────────────────────┘
```

**Key decisions:**
- Text column: constrained to ~65–70 chars at body font size. Never full-width.
- Choices: explicit labelled buttons below scene text, always in same region
- PC stats: fixed sidebar, always visible, left-right doesn't matter — pick one
- NPC info: contextual sub-panel in the sidebar, only shown when an NPC is in scene
- Mechanical changes (skill gain, trait added): brief inline text after scene text,
  sidebar value updates immediately — no separate event log
- floem line spacing needs explicit tuning (default is too tight for extended reading)

**Typography:**
- Serif font for prose — fits the literary register of the setting.
- 16–18px body size equivalent (user-configurable)
- 1.45–1.6× line height (explicit in floem, not default; user-configurable)
- One body font family — no mixing (font family user-configurable, serif default)
- Both light and dark modes required; both must be clean (not just an inversion)

*UI implementation is its own session. This is the design anchor for that session.*

---

## 5. Save File Versioning / Migration

### Decision: Current approach is sufficient for now; add migration framework at v0.1

The current save system validates ID stability before loading. This handles the main
failure mode (pack content changed between saves).

The versioning strategy:
- `SaveFile.version: u32` is already present
- Migration = a `migrate(old: SaveFile) -> Result<SaveFile>` function per version bump
- No migration logic needed until we have real content (v0.1 milestone)
- When content stabilises, add a `migrations/` module to `undone-save` that chains
  version-to-version upgrades: `v0 → v1 → v2` etc.

Community pack removal is already handled by `TooManyIds` error — force the user to
remove the pack before loading the save. This is the right UX for save integrity.

---

*Document status: design session complete. Ready for implementation planning.*
