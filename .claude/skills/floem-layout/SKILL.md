# Floem 0.2.0 Reference For Undone

Use this for any Floem UI work in this repo.

## Source Of Truth

This workspace is pinned to `floem = "0.2"` and currently resolves to `floem 0.2.0`.

Before trusting any snippet:

1. Check [`Cargo.toml`](../../../Cargo.toml) and [`Cargo.lock`](../../../Cargo.lock)
2. Prefer the local crate source in `C:\Users\YJK\.cargo\registry\src\...\floem-0.2.0\`
3. Prefer docs.rs for `0.2.0`
4. Treat GitHub `main` examples as suspicious until verified

Why this matters: upstream Floem moves quickly. Many current `main` branch examples use newer APIs and naming that do not map cleanly onto the `0.2.0` crate used here.

## Safe Syntax Patterns In This Repo

### App bootstrap

```rust
use floem::prelude::*;

fn main() {
    floem::launch(app_view);
}

fn app_view() -> impl View {
    label(|| "Hello".to_string())
}
```

For custom windows, use `Application::new().window(..., Some(WindowConfig::default()...)).run()`.

### Signals and reactive closures

```rust
use floem::prelude::*;
use floem::reactive::{create_effect, create_rw_signal, SignalGet, SignalUpdate};

let count = create_rw_signal(0);

let text = label(move || format!("Count: {}", count.get()));

let inc = button("Increment").action(move || {
    count.update(|value| *value += 1);
});
```

Rules:

- Use `move || signal.get()` or `move || format!(...)` for reactive labels
- Use `.action(move || ...)` on buttons for no-argument click handlers
- Use `SignalUpdate::update` or `set` for mutation
- `RwSignal<T>` is `Copy`, so passing signals around is cheap

### Conditional views

```rust
let body = dyn_container(
    move || phase.get(),
    move |phase| match phase {
        Phase::Loading => label(|| "Loading".to_string()).into_any(),
        Phase::Ready => main_view().into_any(),
    },
)
.style(|s| s.size_full());
```

Rules:

- `dyn_container(update_fn, child_fn)` is the main conditional view primitive in `0.2.0`
- Return `.into_any()` from match arms when view types differ
- If the dynamic container is a full-page wrapper, give it `.style(|s| s.size_full())`
- Unsized `dyn_container` wrappers are a real source of broken centering in this repo

The `dyn_view!` macro exists in `0.2.0`, but `dyn_container(...)` is usually easier to read in this codebase.

### Dynamic lists

```rust
let rows = dyn_stack(
    move || items.get(),
    |item| item.id.clone(),
    move |item| label(move || item.name.clone()),
);
```

Rules:

- The key function must be stable and unique
- `dyn_stack` preserves children when keys stay stable
- Use `virtual_stack` or `virtual_list` only when the item count is large enough to justify virtualization

### Inputs and rich text

```rust
let name = create_rw_signal(String::new());

let input = text_input(name);

let prose = rich_text(move || build_layout(story.get()));
```

Rules:

- `text_input(...)` takes an `RwSignal<String>`
- `rich_text(...)` takes a closure returning `TextLayout`
- For styled markdown-like text, build `TextLayout` directly with `Attrs`, `AttrsList`, and `TextLayout`

## Layout Rules That Matter Here

Floem uses Taffy flex layout. Most bugs in this repo come from flex sizing, scroll sizing, or dynamic wrappers with no explicit size.

### Axis rules

| Parent | Main axis | Cross axis |
|---|---|---|
| `h_stack`, default `container` | horizontal | vertical |
| `v_stack` | vertical | horizontal |

- `justify_*` works on the main axis
- `items_*` works on the cross axis

### Scroll rule

Inside flex layouts, use:

```rust
scroll(child)
    .scroll_style(|s| s.shrink_to_fit())
```

`shrink_to_fit()` matters because it applies `min_size(0, 0).size_full()` to the scroll view. Without it, the viewport often sizes to content instead of the available flex space.

### Scroll flex-shrink bug (floem 0.2.0)

**Root cause:** `to_taffy_style()` in `floem-0.2.0/src/style.rs` ends with
`..Default::default()`, which sets `overflow: Visible` on every node. Floem
exposes no `overflow` style property — you cannot change this.

**Why it matters:** When taffy sees `overflow: visible`, CSS automatic minimum
size kicks in: `min-height: auto` resolves to the content height for flex items.
This means `flex_grow(1.0) + flex_basis(0.0) + min_height(0.0)` still cannot
shrink the scroll below its content height — taffy overrides the explicit
`min_height(0)` with the content-based minimum.

**What works:** `max_height` is reliably respected by taffy regardless of
overflow mode. Use a reactive `max_height` computed from sibling sizes when
the scroll must shrink to accommodate growing siblings.

**Pattern (from left_panel.rs):**
```rust
scroll(child)
    .scroll_style(|s| s.shrink_to_fit())
    .style(move |s| {
        // Compute available height from known sibling sizes
        let sibling_height = compute_sibling_height();
        let max_h = (available_height - sibling_height).max(min_reasonable);
        s.max_height(max_h as f32)
            .flex_grow(1.0)
            .flex_basis(0.0)
            .min_height(0.0)
    })
```

**What does NOT work** (all tested, all failed):
- `min_height(0.0)` alone — overridden by content minimum
- `flex_basis(0.0)` alone — same override
- Wrapping scroll in `container()` with min_height(0) — container also affected
- `Position::Absolute` on siblings — floem rendering doesn't match taffy expectations
- `height(0.0)` — collapses the scroll entirely

### Centering rule

For full-page centering without scrolling:

```rust
container(content).style(|s| {
    s.size_full()
        .flex_col()
        .items_center()
        .justify_center()
})
```

For centered scrollable content:

```rust
let centered = container(content).style(|s| {
    s.width_full().flex_row().justify_center()
});

scroll(centered)
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.size_full())
```

Important:

- `size_full()` inside a scroll child does not mean "viewport height"
- Scroll children effectively get unbounded height
- For "at least viewport height" inside a scroll child, use `min_height_pct(100.0)` where it actually resolves correctly
- In column layouts, scroll areas usually need `.flex_grow(1.0).flex_basis(0.0)`

## Common Failure Modes

### Wrong Floem generation

If a snippet uses names or patterns that do not appear in the local `floem-0.2.0` crate source, stop and verify before adapting it.

### Unsized dynamic wrapper

If centering or `size_full()` is not working, inspect parent wrappers first. In this repo the usual offender is an outer `dyn_container(...)` missing `.style(|s| s.size_full())`.

### Reactive closure missing

`label(|| "text")` is static. `label(move || signal.get())` is reactive.

### Bad list keys

If list rows redraw strangely, selection jumps, or state resets, check the `dyn_stack` key function before changing layout code.

### Wrong scroll mental model

Do not treat `scroll(child)` like a normal container. It is a viewport plus a child with special sizing behavior.

## Debugging Checklist

1. Confirm the crate version is still `0.2.0`
2. Confirm the API exists in local crate source before copying syntax
3. Add `.debug_name("...")` to important wrappers when layout is confusing
4. Check every full-page `dyn_container` for `.style(|s| s.size_full())`
5. Check every flex-contained `scroll(...)` for `.scroll_style(|s| s.shrink_to_fit())`
6. Check whether the problem is really axis confusion: `justify_*` vs `items_*`
7. If scroll won't shrink for growing siblings, use reactive `max_height` (see "Scroll flex-shrink bug" above)

## Supporting Reference

For exact `0.2.0` syntax, constructor signatures, local source paths, and project examples, use:

- [reference/floem-0.2.0-syntax.md](./reference/floem-0.2.0-syntax.md)
