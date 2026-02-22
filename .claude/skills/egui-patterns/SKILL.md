# egui/eframe Patterns

egui is an immediate-mode GUI library. eframe is the application framework on top of it.

## The Immediate-Mode Paradigm

Every frame, you describe the entire UI from scratch. egui does not retain widget state — your `App` struct does.

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label(&self.message);
        if ui.button("Click me").clicked() {
            self.count += 1;
        }
    });
}
```

## Layout

```rust
// Horizontal
ui.horizontal(|ui| {
    ui.label("Name:");
    ui.text_edit_singleline(&mut self.name);
});

// Columns
ui.columns(2, |cols| {
    cols[0].label("Left");
    cols[1].label("Right");
});

// Scroll
egui::ScrollArea::vertical().show(ui, |ui| {
    for item in &self.items { ui.label(item); }
});
```

## State Management

- Simple values: store in `App` struct, pass `&mut` to egui widgets
- Selection: store index or ID in `App`
- Shared state across panels: `Arc<Mutex<T>>`
- Panel open/closed: `bool` field in `App`

## eframe App Lifecycle

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // called every frame
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.persistent_data);
    }
}
```

## Common Widgets

```rust
ui.label("text");
ui.heading("Big");
ui.separator();
ui.button("label")                              // returns Response
ui.checkbox(&mut bool, "label")
ui.radio_value(&mut val, Variant::A, "label")
ui.text_edit_singleline(&mut String)
ui.text_edit_multiline(&mut String)
ui.add(egui::Slider::new(&mut f32, 0.0..=1.0))
ui.add(egui::DragValue::new(&mut i32))
ui.add_enabled(condition, egui::Button::new("maybe disabled"))
```

## Performance Anti-Patterns

- **Heavy computation in `update()`** — spawn a thread; store result in `App`; call `ctx.request_repaint()` when done
- **`ctx.request_repaint()` every frame** — only call when state actually changes
- **`format!` in hot path for stable text** — cache the string in `App`
- **Storing egui widget handles or IDs** — don't; they're ephemeral

## Common Mistake

```rust
// WRONG: forgetting ctx.request_repaint() after async state change
// If you update App state from another thread, the UI won't redraw
// until the user moves the mouse. Always call ctx.request_repaint().
```
