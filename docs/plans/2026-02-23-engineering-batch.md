# Engineering Batch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix five independent engineering issues: packs_dir robustness, female NPC effects, active_npc signal wiring, Literata font embedding, and markdown-in-prose rendering.

**Architecture:** Five independent changes across different crates. Tasks A–D have zero file overlap and run fully in parallel. Task E (markdown) depends cosmetically on D (font) but can run in parallel if needed — font just makes the output look correct.

**Tech Stack:** Rust, floem 0.2 (cosmic-text/fontdb), pulldown-cmark, minijinja templates.

---

## Task A: Fix `packs_dir` Relative Path

**Files:**
- Modify: `crates/undone-ui/src/game_state.rs:26`

**Step 1: Write the failing test**

Add a unit test in `game_state.rs` that verifies `resolve_packs_dir()` returns a path ending in `packs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_packs_dir_returns_path_ending_in_packs() {
        let dir = resolve_packs_dir();
        assert_eq!(dir.file_name().unwrap(), "packs");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-ui resolve_packs_dir`
Expected: FAIL — `resolve_packs_dir` not found.

**Step 3: Write implementation**

Extract a `resolve_packs_dir()` function in `game_state.rs`:

```rust
use std::path::PathBuf;

/// Resolve the packs directory. Tries:
/// 1. `<exe_dir>/packs` (distribution layout)
/// 2. `./packs` (cargo run from workspace root)
fn resolve_packs_dir() -> PathBuf {
    // Distribution: packs/ next to the executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join("packs");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }
    // Development: cwd (cargo run from workspace root)
    PathBuf::from("packs")
}
```

Then replace line 26 (`let packs_dir = Path::new("packs");`) with:

```rust
let packs_dir = resolve_packs_dir();
```

And change `load_packs(packs_dir)` to `load_packs(&packs_dir)`.

**Step 4: Run test to verify it passes**

Run: `cargo test -p undone-ui resolve_packs_dir`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test --workspace`
Expected: all 88+ tests pass.

---

## Task B: Female NPC Effects

**Files:**
- Modify: `crates/undone-scene/src/effects.rs:133-178`
- Test: `crates/undone-scene/src/effects.rs` (existing test module)

**Step 1: Write failing tests for female NPC effects**

Add tests in the existing `mod tests` block in `effects.rs`:

```rust
fn make_female_npc() -> FemaleNpc {
    FemaleNpc {
        core: NpcCore {
            name: "Fiona".into(),
            age: Age::Twenties,
            race: "white".into(),
            eye_colour: "green".into(),
            hair_colour: "red".into(),
            personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
            traits: HashSet::new(),
            relationship: RelationshipStatus::Stranger,
            pc_liking: LikingLevel::Neutral,
            npc_liking: LikingLevel::Neutral,
            pc_love: LoveLevel::None,
            npc_love: LoveLevel::None,
            pc_attraction: AttractionLevel::Unattracted,
            npc_attraction: AttractionLevel::Unattracted,
            behaviour: Behaviour::Neutral,
            relationship_flags: HashSet::new(),
            sexual_activities: HashSet::new(),
            custom_flags: HashMap::new(),
            custom_ints: HashMap::new(),
            knowledge: 0,
            contactable: true,
            arousal: ArousalLevel::Comfort,
            alcohol: AlcoholLevel::Sober,
        },
        char_type: CharTypeId(lasso::Spur::try_from_usize(0).unwrap()),
        figure: PlayerFigure::Slim,
        breasts: BreastSize::Medium,
        clothing: FemaleClothing::default(),
        pregnancy: None,
        virgin: true,
    }
}

#[test]
fn add_npc_liking_works_for_female() {
    let mut world = make_world();
    let key = world.female_npcs.insert(make_female_npc());
    let mut ctx = SceneCtx::new();
    ctx.active_female = Some(key);
    let reg = PackRegistry::new();
    apply_effect(
        &EffectDef::AddNpcLiking { npc: "f".into(), delta: 1 },
        &mut world, &mut ctx, &reg,
    ).unwrap();
    assert_eq!(world.female_npcs[key].core.pc_liking, LikingLevel::Ok);
}

#[test]
fn add_npc_love_works_for_female() {
    let mut world = make_world();
    let key = world.female_npcs.insert(make_female_npc());
    let mut ctx = SceneCtx::new();
    ctx.active_female = Some(key);
    let reg = PackRegistry::new();
    apply_effect(
        &EffectDef::AddNpcLove { npc: "f".into(), delta: 2 },
        &mut world, &mut ctx, &reg,
    ).unwrap();
    assert_eq!(world.female_npcs[key].core.npc_love, LoveLevel::Confused);
}

#[test]
fn set_npc_flag_works_for_female() {
    let mut world = make_world();
    let key = world.female_npcs.insert(make_female_npc());
    let mut ctx = SceneCtx::new();
    ctx.active_female = Some(key);
    let reg = PackRegistry::new();
    apply_effect(
        &EffectDef::SetNpcFlag { npc: "f".into(), flag: "kissed".into() },
        &mut world, &mut ctx, &reg,
    ).unwrap();
    assert!(world.female_npcs[key].core.relationship_flags.contains("kissed"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p undone-scene -- female`
Expected: FAIL — `BadNpcRef("f")`

**Step 3: Implement female NPC dispatch**

Replace `resolve_male_npc_ref` with a general resolver. In the NPC effect arms, dispatch on male vs female:

```rust
enum NpcRef {
    Male(MaleNpcKey),
    Female(FemaleNpcKey),
}

fn resolve_npc_ref(npc: &str, ctx: &SceneCtx) -> Result<NpcRef, EffectError> {
    match npc {
        "m" => ctx.active_male.map(NpcRef::Male).ok_or(EffectError::NoActiveMale),
        "f" => ctx.active_female.map(NpcRef::Female).ok_or(EffectError::NoActiveFemale),
        _ => Err(EffectError::BadNpcRef(npc.to_string())),
    }
}
```

Then update each NPC effect arm to match on the `NpcRef`:

```rust
EffectDef::AddNpcLiking { npc, delta } => {
    match resolve_npc_ref(npc, ctx)? {
        NpcRef::Male(key) => {
            let npc_data = world.male_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, *delta);
        }
        NpcRef::Female(key) => {
            let npc_data = world.female_npc_mut(key).ok_or(EffectError::NpcNotFound)?;
            npc_data.core.pc_liking = step_liking(npc_data.core.pc_liking, *delta);
        }
    }
}
```

Apply the same pattern to `AddNpcLove`, `AddWLiking`, `SetNpcFlag`, `AddNpcTrait`.

Add necessary imports: `FemaleNpcKey`, `FemaleNpc`, `FemaleClothing` (for tests).

**Step 4: Run tests to verify they pass**

Run: `cargo test -p undone-scene`
Expected: all tests pass.

---

## Task C: Wire `active_npc` Signal

**Files:**
- Modify: `crates/undone-scene/src/engine.rs:42-47` (add `EngineEvent::NpcActivated`)
- Modify: `crates/undone-ui/src/lib.rs:250-274` (handle new event in `process_events`)

**Step 1: Write failing test — engine emits NpcActivated on StartScene with active male**

In `engine.rs` test module, add:

```rust
#[test]
fn start_scene_with_set_active_male_emits_npc_activated() {
    let mut engine = make_engine_with(make_simple_scene());
    let mut world = make_world();
    let registry = undone_packs::PackRegistry::new();

    // Insert an NPC and set active
    let npc = MaleNpc {
        core: NpcCore {
            name: "Jake".into(),
            age: Age::Twenties,
            race: "white".into(),
            eye_colour: "blue".into(),
            hair_colour: "brown".into(),
            personality: PersonalityId(lasso::Spur::try_from_usize(0).unwrap()),
            traits: HashSet::new(),
            relationship: RelationshipStatus::Stranger,
            pc_liking: LikingLevel::Neutral,
            npc_liking: LikingLevel::Neutral,
            pc_love: LoveLevel::None,
            npc_love: LoveLevel::None,
            pc_attraction: AttractionLevel::Unattracted,
            npc_attraction: AttractionLevel::Unattracted,
            behaviour: Behaviour::Neutral,
            relationship_flags: HashSet::new(),
            sexual_activities: HashSet::new(),
            custom_flags: HashMap::new(),
            custom_ints: HashMap::new(),
            knowledge: 0,
            contactable: true,
            arousal: ArousalLevel::Comfort,
            alcohol: AlcoholLevel::Sober,
        },
        figure: MaleFigure::Average,
        clothing: MaleClothing::default(),
        had_orgasm: false,
        has_baby_with_pc: false,
    };
    let key = world.male_npcs.insert(npc);

    // Set active male, then start scene
    engine.send(EngineCommand::SetActiveMale(key), &mut world, &registry);
    engine.drain(); // clear any events from SetActiveMale

    engine.send(EngineCommand::StartScene("test::simple".into()), &mut world, &registry);
    let events = engine.drain();

    assert!(
        events.iter().any(|e| matches!(e, EngineEvent::NpcActivated(..))),
        "expected NpcActivated event"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p undone-scene -- npc_activated`
Expected: FAIL — `NpcActivated` variant doesn't exist.

**Step 3: Add `EngineEvent::NpcActivated` and emit it**

In `engine.rs`, add the variant. Use `NpcCore` clone since it's in `undone-domain` (shared):

```rust
#[derive(Debug)]
pub enum EngineEvent {
    ProseAdded(String),
    ActionsAvailable(Vec<ActionView>),
    NpcActivated(Option<undone_domain::NpcCore>),
    SceneFinished,
}
```

In `start_scene()`, after pushing the frame, emit the NPC info if an active NPC is set from a previous scene context carry-over. More importantly, in `SetActiveMale` and `SetActiveFemale` commands, emit the event:

```rust
EngineCommand::SetActiveMale(key) => {
    if let Some(frame) = self.stack.last_mut() {
        frame.ctx.active_male = Some(key);
    }
    if let Some(npc) = world.male_npc(key) {
        self.events.push_back(EngineEvent::NpcActivated(Some(npc.core.clone())));
    }
}
EngineCommand::SetActiveFemale(key) => {
    if let Some(frame) = self.stack.last_mut() {
        frame.ctx.active_female = Some(key);
    }
    if let Some(npc) = world.female_npc(key) {
        self.events.push_back(EngineEvent::NpcActivated(Some(npc.core.clone())));
    }
}
```

Also emit `NpcActivated(None)` on `SceneFinished` to clear the panel:

In `evaluate_next`, where `SceneFinished` is pushed, also push `NpcActivated(None)`:
```rust
if branch.finish {
    self.stack.pop();
    self.events.push_back(EngineEvent::NpcActivated(None));
    self.events.push_back(EngineEvent::SceneFinished);
    return;
}
```

**Step 4: Handle the event in `process_events`**

In `crates/undone-ui/src/lib.rs`, add a match arm in `process_events`:

```rust
EngineEvent::NpcActivated(core) => {
    signals.active_npc.set(core.as_ref().map(NpcSnapshot::from));
}
```

**Step 5: Run tests**

Run: `cargo test --workspace`
Expected: all tests pass.

---

## Task D: Embed Literata Font

**Files:**
- Create: `assets/fonts/` directory with Literata `.ttf` files
- Modify: `src/main.rs:32` (register fonts before `Application::new()`)

**Step 1: Download Literata font files**

Literata is OFL-licensed, available from Google Fonts. Download the variable-weight TTF or the static Regular + Italic + Bold + BoldItalic `.ttf` files into `assets/fonts/`.

Minimum files needed:
- `assets/fonts/Literata-Regular.ttf`
- `assets/fonts/Literata-Italic.ttf`
- `assets/fonts/Literata-Bold.ttf`
- `assets/fonts/Literata-BoldItalic.ttf`
- `assets/fonts/OFL.txt` (license)

**Step 2: Register fonts in `main.rs`**

Before the `Application::new()` call, register the font data with floem's cosmic-text font system:

```rust
// Register bundled Literata font (OFL-licensed).
{
    use floem::text::FONT_SYSTEM;
    let mut fs = FONT_SYSTEM.lock();
    let db = fs.db_mut();
    db.load_font_data(include_bytes!("../assets/fonts/Literata-Regular.ttf").to_vec());
    db.load_font_data(include_bytes!("../assets/fonts/Literata-Italic.ttf").to_vec());
    db.load_font_data(include_bytes!("../assets/fonts/Literata-Bold.ttf").to_vec());
    db.load_font_data(include_bytes!("../assets/fonts/Literata-BoldItalic.ttf").to_vec());
}
```

Note: `floem::text::FONT_SYSTEM` is the re-export path. If this doesn't compile, check via `floem_renderer::text::FONT_SYSTEM` or `floem::cosmic_text::FontSystem`. The agent should verify the actual export path by searching floem's source.

**Step 3: Verify**

Run: `cargo build`
Expected: compiles. The font family string `"Literata, Palatino, Georgia, serif"` in `theme.rs` will now resolve to the embedded Literata. Visual verification via screenshot-mcp.

**Step 4: Commit**

---

## Task E: Markdown in Prose (RichText)

**Files:**
- Modify: `Cargo.toml` (workspace deps: add `pulldown-cmark`)
- Modify: `crates/undone-ui/Cargo.toml` (add `pulldown-cmark`)
- Modify: `crates/undone-ui/src/left_panel.rs:69-78` (replace `label()` with `rich_text()`)

**Step 1: Add pulldown-cmark dependency**

In workspace `Cargo.toml`:
```toml
pulldown-cmark = "0.13"
```

In `crates/undone-ui/Cargo.toml`:
```toml
pulldown-cmark = { workspace = true }
```

**Step 2: Write a markdown-to-TextLayout helper**

Create a function in `left_panel.rs` (or a new `prose.rs` module if it gets large) that:

1. Parses markdown with `pulldown-cmark::Parser`
2. Walks events, building a flat `String` and tracking ranges
3. For each range, sets `Attrs` (bold → `Weight::BOLD`, italic → `Style::Italic`, heading → larger font size)
4. Returns a `floem::text::TextLayout`

```rust
use floem::text::{TextLayout, Attrs, AttrsList, Weight, Style as FontStyle};
use pulldown_cmark::{Parser, Event, Tag, TagEnd};

fn markdown_to_layout(
    markdown: &str,
    font_family: &str,
    font_size: f32,
    line_height: f32,
    ink: Color,
) -> TextLayout {
    let mut text = String::new();
    let mut attrs_list = AttrsList::new(
        Attrs::new()
            .family(&[FamilyOwned::Name(font_family.to_string())])
            .font_size(font_size)
            .color(ink)
            .line_height(line_height),
    );

    let mut bold = false;
    let mut italic = false;
    let mut in_heading = false;

    let parser = Parser::new(markdown);
    for event in parser {
        match event {
            Event::Start(Tag::Strong) => bold = true,
            Event::End(TagEnd::Strong) => bold = false,
            Event::Start(Tag::Emphasis) => italic = true,
            Event::End(TagEnd::Emphasis) => italic = false,
            Event::Start(Tag::Heading { .. }) => in_heading = true,
            Event::End(TagEnd::Heading(..)) => {
                in_heading = false;
                text.push('\n');
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                text.push_str("\n\n");
            }
            Event::Text(t) => {
                let start = text.len();
                text.push_str(&t);
                let end = text.len();

                let mut attrs = Attrs::new()
                    .family(&[FamilyOwned::Name(font_family.to_string())])
                    .font_size(if in_heading { font_size * 1.3 } else { font_size })
                    .color(ink)
                    .line_height(line_height);

                if bold || in_heading {
                    attrs = attrs.weight(Weight::BOLD);
                }
                if italic {
                    attrs = attrs.style(FontStyle::Italic);
                }

                attrs_list.add_span(start..end, attrs);
            }
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            _ => {}
        }
    }

    let mut layout = TextLayout::new();
    layout.set_text(&text, attrs_list);
    layout
}
```

**Important:** The exact floem API for `TextLayout`, `Attrs`, `AttrsList`, `FamilyOwned` must be verified against the actual floem 0.2 exports. The agent implementing this should `grep` for these types in floem's source to confirm the import paths and method signatures.

**Step 3: Replace `label()` with `rich_text()`**

In `left_panel.rs`, replace the `prose_label` binding:

```rust
let prose_view = rich_text(move || {
    let prefs = signals.prefs.get();
    let colors = ThemeColors::from_mode(prefs.mode);
    markdown_to_layout(
        &story.get(),
        &prefs.font_family,
        prefs.font_size as f32,
        prefs.line_height,
        colors.ink,
    )
}).style(move |s| {
    s.padding(24.0).max_width(680.0)
});
```

Update `centered_prose` to use `prose_view` instead of `prose_label`.

**Step 4: Run tests and verify**

Run: `cargo test --workspace`
Expected: all tests pass (no prose tests break — tests don't exercise the UI widget).

Run: `cargo build` — must compile.

Visual verification via screenshot-mcp: prose should show bold/italic formatting from markdown in scene templates.

---

## Parallel Execution Map

```
Time →
Agent 1: [Task A: packs_dir fix]
Agent 2: [Task B: female NPC effects]
Agent 3: [Task C: active_npc signal]
Agent 4: [Task D: Literata font] → [Task E: markdown prose]
```

Tasks A, B, C, D are fully independent — zero file overlap. Task E follows D on the same agent since both touch `undone-ui` and the font must be registered for RichText to render correctly.

**File ownership per agent:**
- Agent 1: `crates/undone-ui/src/game_state.rs`
- Agent 2: `crates/undone-scene/src/effects.rs`
- Agent 3: `crates/undone-scene/src/engine.rs`, `crates/undone-ui/src/lib.rs`
- Agent 4: `src/main.rs`, `assets/fonts/*`, `Cargo.toml`, `crates/undone-ui/Cargo.toml`, `crates/undone-ui/src/left_panel.rs`
