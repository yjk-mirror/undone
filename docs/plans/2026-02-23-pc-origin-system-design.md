# PC Origin System — Design Document

**Date:** 2026-02-23
**Status:** Approved

## Summary

Replace the `always_female: bool` + hidden trait system with an explicit `PcOrigin` enum.
Add a trans woman PC type (`TransWomanTransformed`) — same magical transformation event
as the cis male path, but she was already living as a woman. The magic gave her what she
always wanted. Radically different emotional register at the same mechanical level.

## PcOrigin Enum

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PcOrigin {
    CisMaleTransformed,     // cis man → magically transformed to female body
    TransWomanTransformed,  // trans woman → magic gave her the body she wanted
    CisFemaleTransformed,   // cis woman → transformation variant
    AlwaysFemale,           // no transformation frame at all
}
```

### Helper methods

- `was_transformed() -> bool` — true for all except `AlwaysFemale`
- `was_male_bodied() -> bool` — true for `CisMaleTransformed` and `TransWomanTransformed`
- `has_before_life() -> bool` — true for all transformed variants

## Player Struct Changes

- Remove `always_female: bool`
- Add `origin: PcOrigin`
- Change `before_sexuality: Sexuality` → `before_sexuality: Option<BeforeSexuality>`

## BeforeSexuality Enum (replaces Sexuality)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BeforeSexuality {
    AttractedToWomen,  // was: StraightMale
    AttractedToMen,    // was: GayMale
    AttractedToBoth,   // was: BiMale
}
```

The `AlwaysFemale` sentinel variant is removed. When `origin` is `AlwaysFemale` or
`CisFemaleTransformed`, `before_sexuality` is `None`.

## Auto-Injected Traits & Starting FEMININITY

| Origin | Hidden traits injected | FEMININITY start |
|---|---|---|
| `CisMaleTransformed` | *(none)* | 10 |
| `TransWomanTransformed` | `TRANS_WOMAN` | 70 |
| `CisFemaleTransformed` | `ALWAYS_FEMALE` | 75 |
| `AlwaysFemale` | `ALWAYS_FEMALE`, `NOT_TRANSFORMED` | 75 |

`TRANS_WOMAN` is a new hidden trait added to `packs/base/data/traits.toml`.

## Character Creation UI — Two-Step Flow

**Step 1:** "Were you transformed?"
- "Yes — something happened to change your body"
- "No — you've always been a woman"

**Step 2 (if yes):** "What was your life before?"
- "I was a man" → `CisMaleTransformed`
- "I was a trans woman" → `TransWomanTransformed`
- "I was a woman" → `CisFemaleTransformed`

### Form adaptation by origin

| Origin | "Your Past" section | Before sexuality |
|---|---|---|
| `CisMaleTransformed` | Full (age, sexuality) | "Attracted to women" / "Attracted to men" / "Attracted to both" |
| `TransWomanTransformed` | Full (age, sexuality) | Same orientation-only labels (no gender-identity framing) |
| `CisFemaleTransformed` | Partial (age only) | Hidden |
| `AlwaysFemale` | Hidden entirely | Hidden |

## Expression Evaluator

**New accessor:**
- `w.pcOrigin()` → returns string name of the enum variant

**Updated accessor:**
- `w.alwaysFemale()` → returns `true` for `AlwaysFemale` and `CisFemaleTransformed`
  (backward compat: any origin that has the `ALWAYS_FEMALE` trait)

**Unchanged:**
- `w.hasTrait('TRANS_WOMAN')` — works via auto-injected trait
- `w.getSkill('FEMININITY')` — different starting values, same accessor

### Scene branching pattern

```jinja
{# Coarse gate: was there a transformation at all? #}
{% if w.pcOrigin() != 'AlwaysFemale' %}
  {# Fine gate: what kind of transformation experience? #}
  {% if w.hasTrait('TRANS_WOMAN') %}
    She catches her reflection and stops. This is real. This is *hers*.
  {% else %}
    She catches her reflection and stops. That's... her? Still not used to it.
  {% endif %}
{% else %}
  She catches her reflection. Mondays.
{% endif %}
```

## Save Migration

Old saves use `always_female: bool`. Migration rules:
- `always_female: false` → `CisMaleTransformed`
- `always_female: true` + has `NOT_TRANSFORMED` trait → `AlwaysFemale`
- `always_female: true` without `NOT_TRANSFORMED` → `CisFemaleTransformed`

`before_sexuality` migration:
- `StraightMale` → `Some(AttractedToWomen)`
- `GayMale` → `Some(AttractedToMen)`
- `BiMale` → `Some(AttractedToBoth)`
- `AlwaysFemale` → `None`

## DLC Extensibility

The enum is the engine's closed set. Adding a new origin (e.g., `Hermaphrodite`)
requires:

1. **Engine:** Add variant to `PcOrigin`, handle in all `match` arms
2. **Pack:** Add hidden trait to `traits.toml`, add origin config
3. **Scenes:** Branch on `w.hasTrait(...)` or `w.pcOrigin() == '...'`
4. **Char creation:** Step 2 gains another radio button

Everything else is data-driven — prose, body descriptions, NPC reactions, social
dynamics all live in pack scene files. A modder can write new content for any origin
without touching engine code.
