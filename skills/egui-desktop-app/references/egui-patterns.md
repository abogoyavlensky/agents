# egui / eframe 0.36 patterns and pitfalls

Idioms for writing UI code that works the first time. Checked against egui,
eframe and egui_kittest 0.36.2. egui's API moves between minor versions: if a
snippet doesn't compile on another version, read that version's docs.rs page
instead of reaching for older examples (many online examples use pre-0.30 APIs).

## Contents

1. The eframe app shape
2. Layout: panels, windows, modals
3. Ids and state
4. Borrowing while drawing
5. Input, focus, shortcuts
6. Accessible names
7. Theme and fonts
8. Background work and repaint
9. Common mistakes

## 1. The eframe app shape

eframe 0.36 splits the frame into `logic` (no drawing) and `ui` (drawing). The
older `fn update(&mut self, ctx, frame)` is gone.

```rust
impl eframe::App for DesktopApp {
    // Optional: rewrite input before egui sees it (synthetic input from the control port).
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.ui.raw_input_hook(raw);
    }
    // Before each `ui`, and also while the window is hidden. No widgets here.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.logic(ctx); // drain channels, control requests, timers
    }
    // The root viewport's UI. The given Ui has no margin or background.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui.ui(ui);
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MyApp")
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([360.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native("MyApp", options, Box::new(|cc| Ok(Box::new(DesktopApp::new(cc)))))
}
```

eframe features for a desktop app: `default-features = false` plus
`["default_fonts", "wgpu", "accesskit", "wayland", "x11"]`. Add `"persistence"`
if you use `App::save` / `cc.storage`. `glow` instead of `wgpu` only for old
hardware (VectorCraft keeps a Windows 7 build on glow).

## 2. Layout: panels, windows, modals

Add side and top/bottom panels **before** the central panel; the central panel
takes what is left.

```rust
egui::Panel::top("toolbar").show(ui, |ui| { ui.horizontal(|ui| { /* … */ }); });
egui::Panel::bottom("status").show(ui, |ui| { ui.label(&self.status); });
egui::Panel::left("sidebar").resizable(true).default_size(220.0).show(ui, |ui| { /* … */ });
egui::CentralPanel::default().show(ui, |ui| { /* main content */ });
```

Floating windows and modals take the `Context`:

```rust
egui::Window::new("Preferences").open(&mut self.prefs_open).show(ui.ctx(), |ui| { /* … */ });

let modal = egui::Modal::new(egui::Id::new("confirm-delete")).show(ui.ctx(), |ui| {
    ui.heading("Delete 3 items?");
    ui.horizontal(|ui| {
        if ui.button("Delete").clicked() { /* run the command */ ui.close(); }
        if ui.button("Cancel").clicked() { ui.close(); }
    });
});
if modal.should_close() { self.confirm = None; }
```

Model a dialog as `Option<DialogState>` in your UI state: `Some` means open.
Then a control request or a test can open it, fill it, and confirm it the same
way the user does.

Use `egui::ScrollArea::vertical()` around anything that can grow, and test at a
small window size (kittest `with_size`) to catch content that runs off screen.

Rows that must fit their width, from the template. The button is pinned right
and the title gets what is left, truncated with "…":

```rust
let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
ui.allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), |ui| {
    if ui.small_button("Delete").clicked() { /* … */ }
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
        ui.checkbox(&mut done, &item.text);
    });
});
```

A bare `ui.with_layout(Layout::right_to_left(..))` inside a vertical layout
takes all the remaining height and centres its contents in it, which looks like
a row floating in the middle of the panel. Bound the height as above.

An input that fills the row next to a fixed button:

```rust
let width = (ui.available_width() - 56.0 - ui.spacing().item_spacing.x).max(80.0);
ui.add(egui::TextEdit::singleline(&mut self.draft).desired_width(width));
ui.add_sized([56.0, ui.spacing().interact_size.y], egui::Button::new("Add"));
```

## 3. Ids and state

egui identifies widgets that keep state (scroll areas, collapsing headers, text
edits, combo boxes) by an id derived from the parent id plus a salt. Two widgets
with the same id fight over state, and egui paints a red warning.

- In loops, wrap each row: `ui.push_id(item.id, |ui| { … })`, using an id from
  the data. The index changes when rows are inserted; the text can repeat.
- Give stateful containers explicit salts: `ScrollArea::vertical().id_salt("list")`,
  `ComboBox::from_id_salt("unit")`, `CollapsingHeader::new("Advanced").id_salt(…)`.
- Small transient UI state can live in egui's memory:
  `ui.data_mut(|d| d.insert_temp(id, value))` / `ui.data(|d| d.get_temp::<T>(id))`.
  State you want to test, persist, or drive from the control port belongs in
  your own `UiState` struct instead.

## 4. Borrowing while drawing

Closures borrow `self`, so this fails to compile or invalidates the iterator:

```rust
for item in &self.session.doc().items {          // borrows self.session
    if ui.button("Delete").clicked() {
        self.run("item.remove", json!({"id": item.id}));   // needs &mut self
    }
}
```

Two fixes. For small lists, iterate a clone (`let items = self.session.doc().items.clone();`).
For large ones, record the action and run it after the loop:

```rust
let mut action = None;
for item in &self.session.doc().items {
    if ui.button("Delete").clicked() { action = Some(("item.remove", json!({"id": item.id}))); }
}
if let Some((id, params)) = action { let _ = self.run(id, params); }
```

The second form also keeps "draw" and "change" separate, which makes a frame
easier to reason about.

## 5. Input, focus, shortcuts

- Read input with `ui.input(|i| …)`; consume with `ui.input_mut(|i| i.consume_shortcut(&sc))`
  so one shortcut doesn't fire two handlers.
- Check the longer modifier set first: `Cmd+Shift+Z` before `Cmd+Z`, since the
  shorter one also matches. `Modifiers::COMMAND` is Cmd on macOS, Ctrl elsewhere.
- Text fields swallow keys while focused. Global shortcuts that should not fire
  while typing: check `ui.ctx().memory(|m| m.focused()).is_none()` or
  `!ctx.egui_wants_keyboard_input()`.
- Submit on Enter: `let r = ui.text_edit_singleline(&mut s); if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { … }`.
  Call `r.request_focus()` afterwards to keep typing.
- Right-click menus: `response.context_menu(|ui| { … })`.

## 6. Accessible names

Tests (kittest), the control port (`ui.widgets`, `ui.click {label}`) and screen
readers all see the AccessKit tree. Make every interactive widget findable by a
unique name:

- Buttons, checkboxes, radio buttons: their visible text is the name. Keep it unique per screen.
- Inputs: put a label next to them and link it:
  `let l = ui.label("Width"); ui.add(egui::DragValue::new(&mut w)).labelled_by(l.id);`
- Repeated or icon-only buttons: set the name explicitly:

```rust
let r = ui.small_button("Delete");
let enabled = r.enabled();
r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, format!("Delete {}", item.text)));
```

Some widgets set or clear their own name. A `DragValue` is a
`Role::SpinButton` whose label egui clears, and a `labelled_by` column header
would give every row the same name. Set the node's label directly (the
template's `widgets::set_name`):

```rust
let r = ui.add(egui::DragValue::new(&mut task.estimate).range(1..=8));
r.ctx.accesskit_node_builder(r.id, |node| node.set_label(format!("Estimate for {}", task.title)));
// test: h.get_by_role_and_label(Role::SpinButton, "Estimate for Write report")
```

How egui stores names, which matters when you read the tree yourself: a
`Label`'s text is the node's **value**, other widgets' text is its **label**,
and `labelled_by` is a reference to another node. kittest's `get_by_label`
resolves all three; the template's `ui.widgets` does the same.

## 7. Theme and fonts

Apply the theme once (the template does it on the first frame) and read colours
from tokens in panels.

```rust
pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    let accent = Tokens::get(ctx).accent;      // read BEFORE global_style_mut:
    ctx.global_style_mut(|style| {             // it holds the context lock, and
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);   // reading ctx inside deadlocks
        style.visuals.selection.bg_fill = accent;
    });
}
```

Bundle fonts so every OS and every snapshot renders the same glyphs:

```rust
use egui::epaint::text::{FontInsert, FontPriority, InsertFontFamily};
ctx.add_font(FontInsert::new(
    "Inter",
    egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf")),
    vec![InsertFontFamily { family: egui::FontFamily::Proportional, priority: FontPriority::Highest }],
));
```

For many fonts, build an `egui::FontDefinitions` and call `ctx.set_fonts` once.
Check font licences (OFL is fine) and record them.

Images and SVG icons: `egui_extras` with the `svg`/`image` features and
`egui_extras::install_image_loaders(ctx)` once, then `egui::Image::new(egui::include_image!("icon.svg"))`.

## 8. Background work and repaint

```rust
// Start: in a click handler.
let (tx, rx) = std::sync::mpsc::channel();
let ctx = ui.ctx().clone();
std::thread::spawn(move || {
    let result = load_big_file(&path);   // slow
    let _ = tx.send(result);             // the app may have closed the job; fine
    ctx.request_repaint();               // wake the event loop
});
self.jobs.push(rx);

// Finish: in logic(), every frame.
self.jobs.retain(|rx| match rx.try_recv() {
    Ok(result) => { self.apply(result); false }
    Err(std::sync::mpsc::TryRecvError::Empty) => true,
    Err(_) => false,
});
```

- `ctx.request_repaint_after(Duration::from_millis(500))` for timers and polling.
- Async file dialogs: `rfd::AsyncFileDialog` driven on a thread (`pollster::block_on`)
  with the same channel pattern.
- Don't hold a lock across a frame that the worker also needs; send owned data.

## 9. Common mistakes

| Symptom | Cause | Fix |
|---|---|---|
| `no method update` / `fn ui` signature errors | Pre-0.36 example code | `logic(ctx, frame)` + `ui(ui, frame)`; `Panel::top(...).show(ui, …)` |
| Red "id clash" text over widgets | Same id twice (loops, two scroll areas) | `push_id` with data ids, `id_salt` on containers |
| Click handled twice, or undo fires while typing | Shortcut not consumed, or not checking focus | `consume_shortcut`; check `egui_wants_keyboard_input` |
| UI freezes during load | Work on the UI thread | Thread + channel + `request_repaint` |
| App hangs on theme change | Reading `ctx` inside `global_style_mut` | Read values first |
| Layout looks fine in a test but clipped in the app | Different size or fonts | Test at the minimum window size, bundle fonts |
| Changes only show after moving the mouse | No repaint requested from the worker | `ctx.request_repaint()` after sending results |
| Kittest can't find a widget | No accessible name, or duplicate names | Section 6; `get_by_role_and_label` to disambiguate |
| Button pushed off screen at small sizes | Unbounded row width | Section 2's row recipe; snapshot at the minimum size |
| One drag makes twenty undo steps | A command per frame while dragging | See architecture.md, section 2 |
