# Floem Layout Reference

Floem is a native Rust UI library using taffy (CSS flexbox) for layout. This skill covers
the layout model, common patterns, and pitfalls specific to this project.

## Core Model

Every floem view is a **flex item** inside a **flex container**. Default display is `Flex`,
default direction is `Row`.

- `v_stack(children)` = flex container with `flex_direction: Column`
- `h_stack(children)` = flex container with `flex_direction: Row`
- `container(child)` = flex container with default direction (Row)
- `scroll(child)` = scrollable viewport (special layout behavior, see below)

### Axis Terminology

| Direction | Main Axis | Cross Axis |
|---|---|---|
| Row (h_stack, container) | Horizontal | Vertical |
| Column (v_stack) | Vertical | Horizontal |

- `justify_content` / `justify_center()` aligns on **main axis**
- `align_items` / `items_center()` aligns on **cross axis**

So for a v_stack (column): `justify_center()` = vertical centering, `items_center()` = horizontal centering.
For a container/h_stack (row): `justify_center()` = horizontal centering, `items_center()` = vertical centering.

## Sizing

| Method | CSS Equivalent | Notes |
|---|---|---|
| `width(px)` | `width: Npx` | Fixed pixel width |
| `height(px)` | `height: Npx` | Fixed pixel height |
| `width_full()` | `width: 100%` | 100% of parent content box |
| `height_full()` | `height: 100%` | 100% of parent content box |
| `size_full()` | `width: 100%; height: 100%` | Both axes |
| `min_width(px)` | `min-width: Npx` | Minimum constraint |
| `min_height_pct(100.0)` | `min-height: 100%` | At least parent height |
| `min_height_full()` | `min-height: 100%` | Shortcut for above |
| `max_width(px)` | `max-width: Npx` | Maximum constraint |
| `flex_grow(1.0)` | `flex-grow: 1` | Grow to fill remaining space |
| `flex_basis(0.0)` | `flex-basis: 0` | Start at 0 before growing |

**Percentage units resolve to the parent's content box** (after padding, before border).

## Scroll Container (Critical)

`scroll(child)` creates a scrollable viewport. It has special layout behavior:

1. **The scroll container itself** is a flex item that takes space from its parent.
2. **The child inside scroll** is laid out with the scroll's width as constraint but
   **unbounded height** -- the child can grow as tall as it needs.
3. The scroll clips the child and provides scroll bars.

### Why `size_full()` inside scroll doesn't work for centering

```rust
// WRONG: child gets unbounded height, so height: 100% has no reference
scroll(
    container(content).style(|s| s.size_full().items_center().justify_center())
)
```

The child's `height: 100%` resolves to... nothing useful, because the scroll doesn't
constrain the child's height. The child collapses to content size.

### `shrink_to_fit()` on scroll_style

```rust
scroll(child).scroll_style(|s| s.shrink_to_fit())
```

This sets `min_size(0, 0)` and `size_full()` on the scroll container itself.
Effect: the scroll container fills available space in its parent flex layout while
allowing shrinking. **Always use this when scroll is inside a flex layout.**

### The correct scroll centering pattern

```rust
// Horizontal centering inside a scroll:
let centered = container(content).style(|s| {
    s.width_full().flex_row().justify_center()
});
scroll(centered)
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.size_full())
```

The container fills the scroll's width (`width_full()`), is a row layout (`flex_row()`),
and centers the content horizontally (`justify_center()`). The content's height determines
the scroll content height.

For vertical centering inside a scroll, use `min_height_pct(100.0)` on the centering
container so it fills at least the viewport:

```rust
let centered = container(content).style(|s| {
    s.width_full()
        .min_height_pct(100.0)  // at least viewport height
        .flex_row()
        .justify_center()       // horizontal center
        .items_center()         // vertical center (cross axis of row)
});
scroll(centered)
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.size_full())
```

`min_height_pct(100.0)` works because scroll *does* pass its viewport height as the
percentage reference, even though it doesn't constrain the child's max height.

## Common Patterns in This Project

### Pattern A: Horizontally centered scrollable content

Used in char_creation.rs, left_panel.rs:

```rust
let centered = container(content).style(|s| {
    s.width_full().flex_row().justify_center()
});
scroll(centered)
    .scroll_style(|s| s.shrink_to_fit())
    .style(|s| s.size_full())
```

### Pattern B: Full-viewport centering (both axes, no scroll)

Used for empty states, placeholders:

```rust
container(label("Empty")).style(|s| {
    s.size_full().items_center().justify_center()
})
```

This works because the container has a definite size from `size_full()` resolving
against its parent, so flexbox can distribute free space.

### Pattern C: Flex-grow scroll area in a column

Used in left_panel.rs for story panel (scroll + fixed footer):

```rust
v_stack((
    scroll(centered_prose)
        .scroll_style(|s| s.shrink_to_fit())
        .style(|s| s.flex_grow(1.0).flex_basis(0.0)),
    footer.style(|s| s.height(40.0)),
))
.style(|s| s.flex_grow(1.0))
```

The scroll area grows to fill remaining space after the fixed footer.
`flex_basis(0.0)` ensures it starts at 0 and grows, rather than starting at
content size and potentially overflowing.

### Pattern D: Full-viewport centered page (no scroll needed)

For a page with minimal content that should be centered in the viewport:

```rust
let content = v_stack((...)).style(|s| s.width_full().max_width(720.0));

container(content).style(move |s| {
    s.size_full()
        .flex_col()
        .items_center()      // horizontal center (cross axis of column)
        .justify_center()    // vertical center (main axis of column)
        .padding_horiz(32.0)
        .padding_vert(36.0)
        .background(colors.ground)
})
```

**Critical prerequisite:** The parent view must have a definite size. If the parent
is a `dyn_container`, it MUST have `.style(|s| s.size_full())`. Otherwise
`size_full()` resolves to content size, not viewport size, and centering fails.

### Pattern E: Centered content that may need scrolling

If the content might overflow the viewport, put the centering container OUTSIDE
the scroll:

```rust
let content = v_stack((...)).style(|s| s.width_full().max_width(640.0));

// Centering wrapper outside scroll — gets definite size from parent
let page = container(
    scroll(content)
        .scroll_style(|s| s.shrink_to_fit())
        .style(|s| s.size_full())
).style(move |s| {
    s.size_full()
        .flex_col()
        .items_center()
        .padding_horiz(32.0)
        .background(colors.ground)
});
```

## Pitfalls

1. **`size_full()` inside scroll does NOT pin to viewport height.** Use
   `min_height_pct(100.0)` instead for "at least viewport height".

2. **Percentage widths inside scroll children don't resolve to viewport.**
   `width_full()`, `min_width_full()`, `width_pct(100.0)` all resolve to the
   scroll's CONTENT width, not its viewport width. This is because the scroll
   lays out its child with unbounded constraints. Use `min_height_pct(100.0)`
   for vertical (works for height), but for horizontal you need either:
   - Structure the centering container OUTSIDE the scroll
   - Use explicit pixel widths
   - Use `margin(PxPctAuto::Auto)` with a definite parent width

3. **Forgetting `shrink_to_fit()` on scroll.** Without it, the scroll container
   may not size correctly inside flex layouts.

4. **`items_center()` on a v_stack centers horizontally** (cross axis), not
   vertically. `justify_center()` on a v_stack centers vertically (main axis).
   This is the opposite of what intuition suggests.

5. **`container` default direction is Row.** So `justify_center()` on a container
   centers horizontally, `items_center()` centers vertically.

6. **Children stretch to fill cross-axis by default** (CSS `align-items: stretch`).
   Setting `items_center()` or `items_start()` on the parent overrides this.
   A child with `max_width` inside a container with default stretch will still
   expand to fill width -- you need `items_center()` or `items_start()` on the
   parent to make `max_width` visually meaningful.

7. **Unsized `dyn_container` parents break child `size_full()`.**
   `dyn_container(...)` with no `.style()` has `width: auto, height: auto`.
   If a child view uses `size_full()`, it resolves to the content size, not
   the viewport. Always add `.style(|s| s.size_full())` to `dyn_container`
   wrappers in the view hierarchy, or they act as size-collapsing barriers.
   This was the root cause of centering failures in this project's phase
   containers (`lib.rs` line ~184).

## Quick Reference: "I want to center X"

| Goal | Pattern |
|---|---|
| Center horizontally in scroll | `container(x).style(\|s\| s.width_full().flex_row().justify_center())` then `scroll(...)` |
| Center vertically in scroll | Add `.min_height_pct(100.0).items_center()` to the container |
| Center both axes in scroll | Combine: `.width_full().min_height_pct(100.0).flex_row().justify_center().items_center()` |
| Center both axes (no scroll) | `container(x).style(\|s\| s.size_full().items_center().justify_center())` |
| Center text in a column | Wrap in `container().style(\|s\| s.width_full().flex_row().justify_center())` |
| Fixed-width centered column | Set `max_width(N)` on content, wrap in centering container |
