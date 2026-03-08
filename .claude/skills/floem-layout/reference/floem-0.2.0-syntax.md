# Floem 0.2.0 Syntax And Documentation Notes

This file is the version-pinned reference for the Floem crate actually used by this repo.

## Version Guard

- Workspace dependency: `floem = { version = "0.2" }` in [`Cargo.toml`](../../../../Cargo.toml)
- Resolved crate: `floem 0.2.0` in [`Cargo.lock`](../../../../Cargo.lock)
- Local source: `C:\Users\YJK\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\floem-0.2.0\`
- Official crate docs for this version: [docs.rs/floem/0.2.0](https://docs.rs/floem/0.2.0/floem/)
- Official upstream repo: [lapce/floem](https://github.com/lapce/floem)

Do not assume GitHub `main` matches this repo. Floem's current branch examples already show API drift beyond what `0.2.0` exposes.

## Primary Sources Used

- `floem-0.2.0/README.md`
- `floem-0.2.0/CHANGELOG.md`
- `floem-0.2.0/src/lib.rs`
- `floem-0.2.0/src/views/{mod,decorator,stack,dyn_container,dyn_stack,scroll,label,button,text_input,rich_text,dropdown,checkbox,virtual_stack}.rs`
- Repo usage in [`crates/undone-ui/src/lib.rs`](../../../../crates/undone-ui/src/lib.rs), [`crates/undone-ui/src/left_panel.rs`](../../../../crates/undone-ui/src/left_panel.rs), [`crates/undone-ui/src/char_creation.rs`](../../../../crates/undone-ui/src/char_creation.rs), [`crates/undone-ui/src/settings_panel.rs`](../../../../crates/undone-ui/src/settings_panel.rs), and related UI files

## What Changed In 0.2.0

The `0.2.0` changelog is worth reading because it explains why older or newer examples disagree. High-value items for this repo:

- prelude added
- `dyn_view!` macro added
- scroll extensions added
- `SignalGet`, `SignalWith`, and `SignalUpdate` traits added
- scroll docs, views docs, and decorators docs improved
- `Dropdown` and rich text support improved
- custom debug names and conditional classes added

## Recommended Imports

### Minimal common imports

```rust
use floem::prelude::*;
```

`prelude::*` re-exports:

- `IntoView`, `View`
- all `views::*`
- `create_rw_signal`, `create_signal`
- `RwSignal`
- `SignalGet`, `SignalTrack`, `SignalUpdate`, `SignalWith`
- `Color`
- `UnitExt`, `DurationUnitExt`

### Common explicit imports in this repo

```rust
use floem::prelude::*;
use floem::reactive::{create_effect, create_rw_signal, RwSignal};
use floem::views::dyn_stack;
use floem::style::{CursorStyle, FlexWrap, Position};
use floem::text::{Attrs, AttrsList, FamilyOwned, TextLayout, Weight};
```

Use explicit imports when it makes view code clearer or when the type is not re-exported by the prelude.

## App And Window Bootstrap

### Simplest app

```rust
use floem::prelude::*;

fn main() {
    floem::launch(app_view);
}

fn app_view() -> impl View {
    label(|| "Hello".to_string())
}
```

### Custom window

```rust
use floem::prelude::*;
use floem::window::WindowConfig;
use peniko::kurbo::Size;

fn main() {
    floem::Application::new()
        .window(
            |_| app_view(),
            Some(
                WindowConfig::default()
                    .title("Undone")
                    .size(Size::new(1280.0, 800.0))
                    .undecorated(true),
            ),
        )
        .run();
}
```

`WindowConfig` in `0.2.0` supports:

- `size(...)`
- `position(...)`
- `show_titlebar(...)`
- `undecorated(...)`
- `undecorated_shadow(...)`
- `with_transparent(...)`
- `fullscreen(...)`
- `window_icon(...)`
- `title(...)`
- `enabled_buttons(...)`
- `resizable(...)`
- `window_level(...)`
- `apply_default_theme(...)`
- `font_embolden(...)`

If a snippet uses some other builder that you cannot find in `src/window.rs`, do not copy it.

## Reactive Syntax

### Signals

```rust
let count = RwSignal::new(0);
let story = create_rw_signal(String::new());
```

### Reading

```rust
count.get()
count.with(|value| *value > 0)
count.get_untracked()
```

### Writing

```rust
count.set(3);
count.update(|value| *value += 1);
story.update(|text| text.push_str("More"));
```

### Effects

```rust
create_effect(move |_| {
    let latest = count.get();
    log::debug!("count = {latest}");
});
```

Rule of thumb:

- Use `get()` in reactive closures and effects
- Use `get_untracked()` only when you explicitly do not want to subscribe
- Use `update(...)` when mutating in place
- Use `set(...)` when replacing the value

## Core View Constructors

### Static stacks

```rust
stack((a, b, c))
h_stack((a, b, c))
v_stack((a, b, c))
stack_from_iter(iter)
h_stack_from_iter(iter)
v_stack_from_iter(iter)
```

Semantics:

- `stack(...)` has no default direction override
- `h_stack(...)` defaults to row
- `v_stack(...)` defaults to column

### Containers

```rust
container(child)
clip(child)
scroll(child)
```

`container(...)` is just a single-child view container; do not assume it behaves like a `v_stack`.

### Text and labels

```rust
text("static")
static_label("also static")
label(move || signal.get())
```

Use:

- `text(...)` for non-reactive display
- `label(...)` for reactive text closures

### Buttons

```rust
button("Save").action(move || save_game());
label(|| "Save".to_string()).button().action(move || save_game());
```

`button(...)` accepts any `IntoView` child. `Button::action(...)` wraps a click handler with no event parameter.

### Dynamic containers

```rust
dyn_container(
    move || phase.get(),
    move |phase| match phase {
        Phase::A => view_a().into_any(),
        Phase::B => view_b().into_any(),
    },
)
```

There is also a macro:

```rust
dyn_view!(phase => match phase {
    Phase::A => view_a().into_any(),
    Phase::B => view_b().into_any(),
})
```

In this repo, plain `dyn_container(...)` is usually clearer.

### Dynamic stacks

```rust
dyn_stack(
    move || items.get(),
    |item| item.id,
    move |item| row_view(item),
)
```

Important:

- the `each_fn` is reactive
- the key function must be unique and stable
- the view builder gets the item by value

### Virtual stacks

```rust
virtual_stack(
    VirtualDirection::Vertical,
    VirtualItemSize::Fixed(Box::new(|| 28.0)),
    move || rows.get(),
    |row| row.id,
    move |row| row_view(row),
)
```

Use only for large collections. For ordinary action lists and settings lists, `dyn_stack` is simpler and matches this repo better.

## Widget-Specific Notes

### `text_input`

Signature in `0.2.0`:

```rust
text_input(buffer: RwSignal<String>) -> TextInput
```

Example:

```rust
let name = create_rw_signal(String::new());

let input = text_input(name)
    .style(|s| s.width_full());
```

### `rich_text`

Signature:

```rust
rich_text(text_layout: impl Fn() -> TextLayout + 'static) -> RichText
```

This repo uses rich text for markdown-like prose rendering in [`left_panel.rs`](../../../../crates/undone-ui/src/left_panel.rs).

### `Dropdown`

`Dropdown<T>` is constructor-based, not just a free function. `0.2.0` exposes:

- `Dropdown::new_rw(...)`
- `Dropdown::new(...)`
- `Dropdown::custom(...)`
- `main_view(...)`
- `list_item_view(...)`
- `dropdown_style(...)`

If you need dropdown work, read `src/views/dropdown.rs` first because it is more customizable than the rest of Floem's surface API suggests.

### `Checkbox`

Useful constructors:

- `Checkbox::new(...)`
- `Checkbox::new_rw(...)`
- `Checkbox::labeled(...)`
- `Checkbox::labeled_rw(...)`
- `checkbox(...)`
- `labeled_checkbox(...)`

This repo currently uses `Checkbox` directly in character creation.

## Decorators And Event Handling

Everything here comes from `views::Decorators`.

### Core decorators

```rust
view.style(|s| s.padding(8.0))
view.class(MyClass)
view.class_if(move || enabled.get(), MyClass)
view.debug_name("story-scroll")
view.disabled(move || is_disabled.get())
view.keyboard_navigable()
```

### Event handlers

```rust
view.on_click_stop(move |_| { ... })
view.on_click_cont(move |_| { ... })
view.on_event(EventListener::PointerDown, move |e| { ... })
view.on_key_down(Key::Named(NamedKey::Enter), |_| true, move |_| { ... })
```

Notes:

- `button(...).action(...)` is usually better than wiring click handlers manually for a basic button
- keyboard handlers require `.keyboard_navigable()`

### Animation

```rust
view.animation(|a| {
    a.duration(300.millis())
        .keyframe(0, |f| f.computed_style())
        .keyframe(100, |f| f.style(|s| s.opacity(0.0)))
})
```

This exists in `0.2.0`, but this repo is mostly using static styling so far.

## Style Builder Surface

The style builder is broad. High-use methods in this repo:

### Sizing

```rust
.width(320.0)
.height(40.0)
.width_full()
.height_full()
.size_full()
.min_width(0.0)
.min_height_pct(100.0)
.max_width(720.0)
.flex_grow(1.0)
.flex_basis(0.0)
```

### Flex

```rust
.flex_row()
.flex_col()
.flex_wrap(FlexWrap::Wrap)
.items_start()
.items_center()
.items_end()
.justify_start()
.justify_center()
.justify_end()
.justify_between()
```

### Spacing

```rust
.gap(12.0)
.row_gap(12.0)
.padding(16.0)
.padding_horiz(24.0)
.padding_vert(20.0)
.margin_top(8.0)
```

### Visuals

```rust
.background(Color::BLACK)
.color(Color::WHITE)
.border(1.0)
.border_radius(12.0)
.cursor(CursorStyle::Pointer)
.z_index(10)
```

### Conditional styling

```rust
.apply_if(is_active, |s| s.background(Color::LIGHT_BLUE))
.hover(|s| s.background(Color::LIGHT_GRAY))
.transition(Background, Transition::linear(150.millis()))
.class(MyClass, |s| s.padding(8.0))
```

Prefer checking `src/style.rs` for exact method names if you are reaching beyond the common set above.

## Scroll And Layout Notes

### Scroll methods

`Scroll` in `0.2.0` supports:

- `on_scroll(...)`
- `ensure_visible(...)`
- `scroll_delta(...)`
- `scroll_to(...)`
- `scroll_to_percent(...)`
- `scroll_to_view(...)`
- `scroll_style(...)`

### `shrink_to_fit()`

Documented in `src/views/scroll.rs` as:

> internally this does `min_size(0., 0.).size_full()`

This is why it fixes the common flex-contained scroll bug in this repo.

### Repo-specific scroll pattern

```rust
let centered = container(content).style(|s| {
    s.width_full().flex_row().justify_center()
});

scroll(centered)
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.size_full())
```

See:

- [`crates/undone-ui/src/char_creation.rs`](../../../../crates/undone-ui/src/char_creation.rs)
- [`crates/undone-ui/src/settings_panel.rs`](../../../../crates/undone-ui/src/settings_panel.rs)
- [`crates/undone-ui/src/left_panel.rs`](../../../../crates/undone-ui/src/left_panel.rs)

### Dynamic wrapper pitfall

This repo has already hit a real bug where a `dyn_container(...)` without `.style(|s| s.size_full())` caused descendant `size_full()` calls to resolve against content size instead of viewport size.

See the top-level app phase switching in [`crates/undone-ui/src/lib.rs`](../../../../crates/undone-ui/src/lib.rs).

## Project Examples Worth Copying

- Full app shell and phase switching: [`crates/undone-ui/src/lib.rs`](../../../../crates/undone-ui/src/lib.rs)
- Scrollable centered prose and action list: [`crates/undone-ui/src/left_panel.rs`](../../../../crates/undone-ui/src/left_panel.rs)
- Complex form layout with dropdowns, checkboxes, and dynamic sections: [`crates/undone-ui/src/char_creation.rs`](../../../../crates/undone-ui/src/char_creation.rs)
- Settings page with centered scroll content: [`crates/undone-ui/src/settings_panel.rs`](../../../../crates/undone-ui/src/settings_panel.rs)
- Save list with `dyn_stack`: [`crates/undone-ui/src/saves_panel.rs`](../../../../crates/undone-ui/src/saves_panel.rs)

Copy from this repo before copying from the web when possible.

## Fast Verification Workflow

When writing Floem code for this repo:

1. Confirm the API in local `floem-0.2.0` source
2. Prefer an existing pattern from `crates/undone-ui/src`
3. Only then write or adapt new code
4. Run `cargo check -p undone-ui`
5. If layout is wrong, inspect wrapper sizing and scroll setup before changing leaf widgets

## Red Flags

Stop and verify if you see:

- `Stack::vertical`, `Container::new`, `Label::derived`, or other APIs you cannot find in the local `0.2.0` crate
- a full-page `dyn_container(...)` with no explicit size style
- `scroll(...)` inside a flex layout with no `.scroll_style(|s| s.shrink_to_fit())`
- reactive state read outside a closure where you expected UI updates
- a `dyn_stack` key function based on list index when the list can reorder
