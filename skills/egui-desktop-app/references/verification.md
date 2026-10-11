# Headless verification for egui apps

How an agent proves a UI change works without a person watching: kittest
tests, pixel snapshots, and driving the real app through a control port. All of
it runs on a Linux box with no display and no GPU. Everything here is
implemented and tested in `assets/template/`.

## Contents

1. Choosing a level
2. egui_kittest: interaction tests
3. Snapshot tests
4. The live app: control port
5. Adding control methods
6. Running without a display or GPU
7. CI
8. Troubleshooting

## 1. Choosing a level

| Change | Minimum proof |
|---|---|
| Domain logic, a new command | Core unit test, plus a refused-input test |
| A widget's behaviour (click does X, Enter submits, disabled state) | kittest interaction test |
| Layout, styling, a new panel | kittest snapshot, then read the PNG |
| Window, fonts, menus, dialogs, clipboard, startup | Live app screenshot, read it |
| A bug report "when I do A then B, C happens" | Reproduce in kittest first; live app if kittest can't reach it |

Always look at images you produce. Read the PNG with the file-reading tool;
it shows the picture.

## 2. egui_kittest: interaction tests

kittest runs your real UI function in an `egui::Context` with no window, finds
widgets through the AccessKit tree (like a screen reader), and injects input.

```toml
# crates/ui/Cargo.toml
[dev-dependencies]
egui_kittest = { version = "0.36", features = ["wgpu", "snapshot"] }
```

Build a harness around your app state:

```rust
use egui::accesskit::Role;
use egui_kittest::{Harness, kittest::{NodeT, Queryable}};

fn harness(app: AppUi) -> Harness<'static, AppUi> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(480.0, 320.0))
        .build_ui_state(|ui, app: &mut AppUi| { app.logic(ui.ctx()); app.ui(ui); }, app);
    h.run();
    h
}
```

Other builders: `build_ui(|ui| …)` for stateless snippets, and `build_eframe(|cc| MyApp::new(cc))`
(egui_kittest `eframe` feature) to test the whole `eframe::App`. Note that
kittest does not call `raw_input_hook`.

API that covers most tests:

| Call | Does |
|---|---|
| `h.get_by_label("Add")` | The one node with that name; panics on zero or several |
| `h.get_by_role_and_label(Role::TextInput, "New item")` | Disambiguates a label from the input it names |
| `h.query_by_label(..)` / `query_all_by_label(..)` | `Option` / iterator; for "is it gone?" checks |
| `h.get_by_label_contains(..)`, `get_by_value(..)`, `get_by_role(..)` | Other lookups |
| `node.click()`, `click_secondary()`, `hover()`, `focus()` | Pointer and focus |
| `node.type_text("…")` | Text into the focused widget (call `focus()` first) |
| `node.rect()`, `node.value()`, `node.accesskit_node().is_disabled()` | Inspect |
| `h.key_press(Key::Enter)`, `h.key_press_modifiers(Modifiers::COMMAND, Key::Z)` | Keyboard |
| `h.drag_at(pos)`, `h.drop_at(pos)`, `h.event(egui::Event::…)` | Raw input |
| `h.run()` | Run frames until the UI settles. Call it after every interaction |
| `h.run_steps(n)`, `h.step()` | Exact frame counts (animations, one-event-per-frame input) |
| `h.state()`, `h.state_mut()` | Your app state, for assertions and set-up |
| `h.ctx` | The `egui::Context` (public field) |
| `h.set_size(..)` | Resize; test small windows |

Patterns:

- **Set up through commands, act through the UI.** `h.state_mut().run("item.add", …)`
  for the starting state, then `click`/`type_text` for the behaviour under test.
- **Assert on state, not pixels**, in interaction tests. Pixels belong in snapshots.
- **Test the error path:** click Add with an empty field, assert nothing was
  added and the status line explains why.
- **Panel-level harnesses** keep tests fast and focused: render one panel's
  `show(app, ui)` instead of the whole app.
- **Drags:** move, press, several moves, release, each followed by
  `run_steps(1)`; egui needs separate frames to recognise a drag. The
  template's `drag_values_are_named_and_dragged` test is a working example.
- **Roles for `get_by_role_and_label`:** `Button`, `CheckBox`, `RadioButton`,
  `TextInput`, `SpinButton` (DragValue), `Slider`, `ComboBox`, `Label`.

When the AccessKit tree isn't enough (custom-painted canvas, text inside a
painter), read the frame's shapes. VectorCraft collects every `Shape::Text`
with its rect from `h.output().shapes` to find text in custom widgets, and
reads widget rects with `ctx.read_response(Id::new(...))`.

## 3. Snapshot tests

```rust
#[test]
fn todo_list_snapshot() {
    let mut h = Harness::builder().with_size(egui::vec2(480.0, 320.0)).wgpu()
        .build_ui_state(|ui, app: &mut AppUi| app.ui(ui), app_with_three_items());
    h.run();
    h.snapshot("todo_list");   // compares with tests/snapshots/todo_list.png
}
```

- First run, or after an intended visual change: `UPDATE_SNAPSHOTS=1 cargo test -p <app>-ui`
  writes the reference. `UPDATE_SNAPSHOTS=force` rewrites even passing ones.
- On a mismatch kittest writes `name.new.png` and `name.diff.png` next to the
  reference. Read both before deciding whether the change is intended.
- Settings in `kittest.toml` at the workspace root: `output_path`, `threshold`,
  `max_failed_pixels`, and per-OS overrides (`[windows]`, `[mac]`, `[linux]`).
- Snapshots render through wgpu and need an adapter. On a machine without a
  GPU, Mesa's lavapipe works. GPUs anti-alias slightly differently, so
  compare snapshots on one platform (the template uses Linux + lavapipe in CI)
  and name snapshot tests `*_snapshot` so other platforms can
  `cargo test -- --skip _snapshot`.
- Keep snapshots deterministic: fixed window size and `pixels_per_point`,
  bundled fonts, no clocks or random data in the frame. Mask dynamic regions or
  leave them out of the snapshot.
- Commit reference PNGs. Ignore `*.new.png`, `*.diff.png` and `*.old.png`.
- For a set of screens across themes, sizes and scales, loop and save
  `h.render()?` images (VectorCraft's `capture_chrome_visual_fixtures` does this
  behind an env var) and review them as a batch.

## 4. The live app: control port

The template's desktop binary takes `--control PORT` (or `MYAPP_CONTROL_PORT`)
and serves JSON lines on `127.0.0.1:PORT`:

```
→ {"id":1,"method":"app.inspect","params":{}}
← {"id":1,"ok":true,"result":{...}}
← {"id":2,"ok":false,"error":"no widget labelled \"Sav\" (see ui.widgets)"}
```

| Method | Params | Result |
|---|---|---|
| `app.commands` | | the command registry |
| `app.run` | `{command, params?}` | the command's result, or its error |
| `app.inspect` | | document, draft text, status line, undo state, window size, graphics adapter |
| `ui.widgets` | `{label?}` | `[{role, label, value, rect:[x,y,w,h], disabled}]` from the AccessKit tree |
| `ui.click` | `{label}` or `{x, y}`; `button?: "right"`, `cmd?`, `shift?`, `alt?` | the point clicked |
| `ui.drag` | `{label}` or `{x, y}`, then `{toX, toY}` or `{dx, dy}`; `steps?` (8) | `{from, to}` |
| `ui.key` | `{key, cmd?, shift?, alt?}` (egui key names: `Enter`, `Escape`, `Z`, `ArrowDown`…) | |
| `ui.text` | `{text}` | (into the focused widget) |
| `ui.screenshot` | `{path}` | `{path, size}` once a frame was captured |
| `app.quit` | | |

Behaviour worth knowing:

- Requests are handled on the UI thread between frames, so they see
  consistent state.
- Input requests (`ui.click`, `ui.drag`, `ui.key`, `ui.text`) reply after the
  input was delivered and one more frame ran. Each pointer move, press and
  release gets its own frame, as with a real mouse, so egui sees real clicks
  and drags. A batch is therefore sequential: an `app.inspect` after a click
  sees the click's effect.
- `{label}` targets (`control::find_widget`) prefer an exact name over a
  substring, and an interactive widget over static text. A text field and the
  Label naming it share a name, and the AccessKit tree lists nodes in no fixed
  order, so without that rule a click could land on the label. `ui.widgets`
  with no filter lists everything on screen; use it when a click "does nothing".
- Coordinates are egui points (logical pixels), the same units as the rects.
- The server binds loopback only, bounds line length, and closes the connection
  on anything that isn't a request object.

Clients: `scripts/egui_ctl.py METHOD [PARAMS]`, `egui_ctl.py --batch steps.jsonl`
(one `{"method","params"}` per line, `#` comments allowed), or `nc` /
`socat` by hand. `--wait 30` retries the connection while the app starts.
Exit status: 0 all ok, 1 an error reply, 2 could not connect.

A typical end-to-end check:

```jsonl
# steps.jsonl
{"method": "app.run", "params": {"command": "item.add", "params": {"text": "Seeded"}}}
{"method": "ui.click", "params": {"label": "New item"}}
{"method": "ui.text", "params": {"text": "Typed"}}
{"method": "ui.key", "params": {"key": "Enter"}}
{"method": "ui.click", "params": {"label": "Typed"}}
{"method": "app.inspect"}
{"method": "ui.screenshot", "params": {"path": "/tmp/after.png"}}
{"method": "app.quit"}
```

The one-shot alternative needs no client: `myapp --screenshot /tmp/shot.png`
renders five frames, saves the window, prints the reply, and exits.

## 5. Adding control methods

Add a method when an agent repeatedly needs something it can only get by
guessing from screenshots, such as a canvas's view transform, or opening a
dialog. Keep the transport dumb and handle everything in `ui/src/control.rs`:

1. Add a match arm in `control::handle`. Read params with
   `p.get("x").and_then(Value::as_f64)` and refuse missing or invalid ones with `err(...)`.
2. Return `ok(value)` at once, `after_input(app, req, value)` if you queued
   synthetic input, or push a pending entry and return `None` if the answer
   needs a future frame (as `ui.screenshot` does).
3. Document it in the module comment and test it in the kittest
   `control_channel_drives_the_ui` style: same `handle`, no socket.

Prefer exposing state over adding special actions: `app.inspect` returning a
rect lets an agent click it with ordinary `ui.click`. VectorCraft's
`ui.inspect` reports the canvas rect, the view, open dialogs, background jobs
and frame timings.

If agents may run the app on a real user's machine, confine what control
requests can read and write (VectorCraft's `--automation-read-root` /
`--automation-write-root`) and never bind beyond loopback.

## 6. Running without a display or GPU

On a Linux box with no display:

```bash
# Packages (Debian/Ubuntu)
sudo apt-get install -y xvfb mesa-vulkan-drivers libxkbcommon-x11-0

# Start, wait for the port, drive, stop
scripts/run_headless.sh --port 7979 -- ./target/debug/myapp --control 7979
scripts/egui_ctl.py --batch steps.jsonl
scripts/egui_ctl.py app.quit
```

`run_headless.sh` wraps the command in `xvfb-run -a` only when `DISPLAY` and
`WAYLAND_DISPLAY` are unset, so the same command works on a desktop, where the
window opens normally. With no GPU, wgpu picks lavapipe automatically when it
is the only Vulkan driver. The template logs the adapter at start-up
(`graphics: llvmpipe … (Vulkan, Cpu)`) and reports it in `app.inspect`.

Harmless log lines under Xvfb: winit's "error setting XSETTINGS" and "display's
0mm in size", and egui_wgpu's "Software rasterizer detected".

Without root, a missing shared library can be supplied locally:

```bash
mkdir -p /tmp/libs && cd /tmp/libs
apt-get download libxkbcommon-x11-0 libxcb-xkb1
for d in *.deb; do dpkg -x "$d" root; done
export LD_LIBRARY_PATH=/tmp/libs/root/usr/lib/x86_64-linux-gnu
```

macOS: the window opens on the real screen; screenshots fail while the screen
is locked or the window is minimized. Windows: same, and CI runners do have a
desktop session.

## 7. CI

The template's `.github/workflows/ci.yml`:

- Linux: installs `libxkbcommon-x11-0 libwayland-dev libx11-dev mesa-vulkan-drivers xvfb`,
  runs `cargo xtask ci` (snapshots included), then smoke-tests the real window
  with `xvfb-run -a ./target/debug/myapp --screenshot smoke.png`.
- macOS and Windows: `cargo test --workspace -- --skip _snapshot`.
- Uploads `smoke.png` and any `*.new.png` / `*.diff.png`, so a failed snapshot
  can be inspected from the run page.

## 8. Troubleshooting

| Problem | Fix |
|---|---|
| `Library libxkbcommon-x11.so could not be loaded` | Install `libxkbcommon-x11-0` (or supply it as in section 6) |
| Snapshot test: no adapter / wgpu error | Install `mesa-vulkan-drivers`; check `vulkaninfo --summary` |
| `get_by_label` panics: several nodes | A label and its input share the name; use `get_by_role_and_label` |
| `get_by_label` panics: not found | Missing accessible name, widget not drawn yet (call `h.run()`), or scrolled out of view |
| `type_text` does nothing | The widget wasn't focused: `node.focus()` first, then `h.run()` |
| Live click "does nothing" | Check `ui.widgets`; a modal may cover it; the window may be smaller than expected (`app.inspect` window size) |
| `ui.screenshot` times out | No frame was presented: window minimized, screen locked, or no display |
| Linker crash or bus error during `cargo test` | The target dir's disk is full, often a small `/tmp` tmpfs: set `CARGO_TARGET_DIR` on a real disk |
| Port already in use | Another agent's app: pick another port, one per agent |
