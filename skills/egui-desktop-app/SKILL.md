---
name: egui-desktop-app
description: >-
  Build, extend, and verify cross-platform desktop apps in Rust with egui and
  eframe (macOS, Windows, Linux), set up so coding agents can check their own
  work headlessly: egui_kittest UI tests that find widgets by accessible name,
  wgpu pixel snapshots that run on a software GPU, and a JSON-lines control
  port that drives the real window under Xvfb and takes screenshots. Includes
  a tested starter workspace (core / ui / desktop crates, command registry,
  no-panic lints, layer check, `cargo xtask ci`, CI workflow) and the
  practices VectorCraft used to build a large egui app with agents. Use this
  whenever the user wants to create a desktop app or GUI tool in Rust, mentions
  egui, eframe, egui_kittest, or immediate-mode GUI, asks to add a panel,
  dialog, widget, shortcut, or theme to an egui app, needs to test or
  screenshot an egui UI without a display, or asks how an agent can verify a
  Rust GUI change — even if they don't name egui explicitly. Not for Tauri,
  Electron, or web front ends.
---

# Desktop apps with egui + eframe, built for agent verification

egui is an immediate-mode GUI: the UI is a function of state that runs every
frame. eframe puts it in a native window and draws it with wgpu. The pairing
suits agent-driven work well, because a bug is usually local to the function
that draws the broken thing, and because egui can run fully headless.

The point of this skill is a tight **change → verify** loop an agent can close
on its own. Most of what follows exists to make that loop cheap and reliable:
one door for every action, UI tests that need no window, pixels you can look
at, and a way to drive the real app.

Pinned stack (tested together, 2026-10): `egui`, `eframe`, `egui_kittest` **0.36**,
Rust **1.95+**, edition 2024. egui's API changes between minor versions; keep the
whole egui family on one version and bump them together. When something below
doesn't compile, check docs.rs for the pinned version rather than guessing from
older examples.

## Starting a new app

Copy the starter workspace and rename it. It builds, passes `cargo xtask ci`,
and already has every piece described below wired up in a small todo app.

```bash
cp -r ~/.claude/skills/egui-desktop-app/assets/template ./notes && cd notes
# Rename crates, binary, window title and env var (myapp → notes, MyApp → Notes)
grep -rlI --exclude-dir=target -e myapp -e MyApp -e MYAPP . \
  | xargs perl -pi -e 's/myapp/notes/g; s/MyApp/Notes/g; s/MYAPP/NOTES/g'
cargo fmt --all && git init && cargo xtask ci
```

Then replace the todo domain in `crates/core` with the real one, keeping the
shape: a `Session` that owns the document, `COMMANDS` that describe every
action, and `run(id, params)` that executes them atomically. Rewrite the UI and
its tests to match, delete the old `crates/ui/tests/snapshots/todo_list*.png`
(kittest never removes orphaned references), and regenerate snapshots.

The first build compiles wgpu and friends: expect several minutes and a few GB
in the target directory. Later builds take seconds.

```
crates/core    domain model + command registry + undo; no UI toolkit (layer-checked)
crates/ui      egui front end: AppUi::ui, theme tokens, control handling, kittest tests
apps/desktop   eframe window, CLI flags, control server, platform glue
xtask          `cargo xtask ci` = fmt --check, clippy -D warnings, layers, tests
```

For an existing egui app, adopt the pieces in this order, since each one pays
off on its own: the no-panic lints and `cargo xtask ci`, then kittest tests,
then routing actions through commands, then the control port.

## The core practices

These come from VectorCraft (a ~370k-line egui app built largely by agents) and
are all present in the template. Read `references/architecture.md` for details
and code.

1. **Everything is a command.** Each user-visible action is an id plus JSON
   params, run through one function. Buttons, menus, shortcuts, the control
   port, and tests all use it. This is what lets an agent or a test do anything
   a person can, without a window, and it gives undo and error reporting one
   home.
2. **The UI is thin.** Panels read state and call `app.run(id, params)`; they
   hold no domain logic. Nothing in `core` depends on egui, enforced by
   `cargo xtask layers`. Logic stays testable in plain unit tests.
3. **Never crash.** Clippy denies `unwrap`, `expect`, `panic!`, `todo!` outside
   tests. Files, clipboard, and control requests are untrusted input: return
   errors, use `.get()` instead of indexing, cap sizes. A `catch_unwind` guard
   around commands and the frame is the last line of defence, not error handling.
4. **Commands are atomic.** An `Err` leaves the document unchanged; a success
   that changes it records one undo step. Test both, and sweep every command
   with junk params to prove nothing panics.
5. **Every interactive widget has a unique accessible name.** Visible text,
   `labelled_by` for inputs, and an explicit label for repeated buttons
   ("Delete Buy milk", not five "Delete"s). Screen readers, kittest queries, and
   the control port all find widgets by these names. This is the single habit
   that makes UI tests easy to write.
6. **Colours and spacing come from theme tokens**, applied once. A restyle is
   one edit, and snapshot tests stay meaningful.
7. **One gate command** (`cargo xtask ci`) that agents run before every commit,
   and an `AGENTS.md` in the project that states these rules (the template ships one).

## Verifying a change headlessly

Use the cheapest level that proves the change, and go up a level when the
change is visual or touches the window. `references/verification.md` has the
full API, the control protocol, and CI setup.

| Level | What it proves | Needs | Command |
|---|---|---|---|
| 1. Core tests | Behaviour, errors, undo | nothing | `cargo test -p <app>-core` |
| 2. kittest interaction | Real UI code: find, click, type, keys, state | no window, no GPU | `cargo test -p <app>-ui` |
| 3. kittest snapshot | Real pixels of a panel or screen | wgpu adapter (lavapipe is fine) | `UPDATE_SNAPSHOTS=1 cargo test -p <app>-ui` then read the PNGs |
| 4. Live app | Window, fonts, platform glue, end to end | display or Xvfb | `--screenshot`, or `--control PORT` + `scripts/egui_ctl.py` |

The loop for a typical UI feature:

1. Add or change the command in `core`, with a unit test, including a refused-input case.
2. Draw it in `ui`. Give each new interactive widget an accessible name.
3. Write a kittest test that drives it the way a person would: query by role
   and label, click or type, `h.run()`, assert on `h.state()`.
4. If it is visual, add or update a snapshot, then **open the PNG and look at
   it**. A passing test with a broken layout is still broken.
5. Run the real app once for anything involving the window, fonts, file
   dialogs, or the clipboard, and read the screenshot.
6. `cargo xtask ci`.

A kittest test, in the template's style:

```rust
use egui::accesskit::Role;
use egui_kittest::{Harness, kittest::Queryable};

let mut h = Harness::builder()
    .with_size(egui::vec2(480.0, 320.0))
    .build_ui_state(|ui, app: &mut AppUi| { app.logic(ui.ctx()); app.ui(ui); }, AppUi::new(Session::new()));
let input = h.get_by_role_and_label(Role::TextInput, "New item");
input.focus();
input.type_text("Buy milk");
h.run();
h.get_by_label("Add").click();
h.run();
assert_eq!(h.state().session.doc().items.len(), 1);
```

Driving the real app (any OS; Xvfb is used automatically on Linux without a display):

```bash
S=~/.claude/skills/egui-desktop-app/scripts
export EGUI_CONTROL_PORT=7979                   # egui_ctl.py's port; pick your own per agent
cargo build -p notes
$S/run_headless.sh --port $EGUI_CONTROL_PORT -- ./target/debug/notes --control $EGUI_CONTROL_PORT
$S/egui_ctl.py ui.widgets '{"label": "Add"}'          # what is on screen, with rects
$S/egui_ctl.py ui.click '{"label": "New item"}'       # an input wins over the Label naming it
$S/egui_ctl.py ui.text '{"text": "Buy milk"}'
$S/egui_ctl.py ui.key '{"key": "Enter"}'
$S/egui_ctl.py ui.drag '{"label": "Estimate for Buy milk", "dx": 40}'   # e.g. a named DragValue
$S/egui_ctl.py app.inspect                            # state, status line, window, graphics adapter
$S/egui_ctl.py ui.screenshot '{"path": "/tmp/shot.png"}'   # then Read the PNG
$S/egui_ctl.py app.quit
```

Input requests reply only after the input was delivered and a frame ran, so a
batch (`egui_ctl.py --batch steps.jsonl`) is sequential and an inspect or
screenshot sent next sees the result. Prefer `app.run` for setting up state and
`ui.click`/`ui.text` for what you are actually testing. For a quick look with no
client, `./target/debug/notes --screenshot /tmp/shot.png` renders a few frames,
saves the window, and exits.

## egui rules that save time

`references/egui-patterns.md` covers these with code. The ones agents trip on most:

- **eframe 0.36's `App` has `logic(ctx, frame)` and `ui(ui, frame)`**, not the
  older `update(ctx, frame)`. Panels are `egui::Panel::top("id").show(ui, …)` and
  `egui::CentralPanel::default().show(ui, …)`.
- **Ids must be stable and unique.** Inside loops, `ui.push_id(item.id, …)` with
  an id from the data, never the index or the label.
- **Don't mutate the collection you are iterating while drawing it.** Clone a
  small view model or collect the clicked action and run it after the loop.
- **Never block `ui()`.** Long work goes to a thread that sends results over a
  channel and calls `ctx.request_repaint()`. File dialogs (rfd) the same way.
- **`ctx.global_style_mut` holds the context lock.** Read anything from `ctx`
  before the closure, or it deadlocks.
- **Shortcuts: check the longer modifier set first** (Cmd+Shift+Z before Cmd+Z),
  using `consume_shortcut`.
- **Bundle your fonts** with `include_bytes!` and install them in the theme, so
  the app and snapshot tests render identical glyphs on every OS.
- **Rows that must fit:** pin buttons with a fixed-height `right_to_left`
  layout, give the rest to the title, and truncate it
  (`TextWrapMode::Truncate`). A bare `ui.with_layout(right_to_left)` takes all
  remaining height. Snapshot at the minimum window size to catch clipping.
- **Some widgets clear their own name:** a `DragValue` is a `Role::SpinButton`
  with no label. Name it per row with `widgets::set_name`.
- **Continuous edits** (drags, sliders) run a command per frame. Merge them into
  one undo step: commit on `drag_stopped`, or coalesce repeats on the same target.

## Gotchas when running headless

- **Linux runtime libraries:** winit panics with "libxkbcommon-x11.so could not
  be loaded" without `libxkbcommon-x11-0`. Without a GPU, wgpu needs Mesa's
  lavapipe (`mesa-vulkan-drivers`). Xvfb comes from `xvfb`.
- **Snapshots differ by GPU.** Make reference PNGs on the platform CI compares
  them on (Linux + lavapipe in the template), and skip `*_snapshot` tests
  elsewhere. Thresholds live in `kittest.toml`.
- **Screenshots need a presented frame.** A minimized window or locked macOS
  screen never answers; the template replies with an error after 5 s.
- **Builds are big.** An egui workspace target dir reaches several GB. Keep it
  off small tmpfs mounts (a full disk shows up as a linker crash or bus error),
  and give parallel agents separate `CARGO_TARGET_DIR`s and control ports.
- **Xvfb log noise is harmless:** winit's XSETTINGS and "0mm display" warnings,
  and egui_wgpu's "Software rasterizer detected" on lavapipe. The app logs its
  adapter at start-up (`RUST_LOG=debug` for more).

## Files in this skill

- `assets/template/`: the starter workspace, tested with `cargo xtask ci` and a
  live Xvfb run. Copy it; don't edit it in place.
- `references/architecture.md`: commands, layering, robustness, state and
  persistence, background work, platform glue, packaging. Read when designing
  or extending an app.
- `references/egui-patterns.md`: egui/eframe 0.36 idioms and pitfalls with code.
  Read when writing UI code.
- `references/verification.md`: kittest API, snapshots, the control protocol,
  adding control methods, Xvfb/lavapipe, CI. Read when writing tests or driving
  the app.
- `scripts/egui_ctl.py`: JSON-lines client for the control port (single call or `--batch`).
- `scripts/run_headless.sh`: start the app in the background (under Xvfb when
  needed) and wait for the control port.
