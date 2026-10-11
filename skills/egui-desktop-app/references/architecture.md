# Architecture for an agent-built egui app

How to structure an egui/eframe desktop app so it stays easy to change and easy
to verify as it grows. Every rule here exists in the starter template
(`assets/template/`), and most are taken from VectorCraft, an Illustrator-class
editor whose ~370k lines of egui code were written largely by agents.

## Contents

1. Workspace and layers
2. Commands: one door for every action
3. The thin UI
4. Robustness: never crash
5. State, persistence, recovery
6. Background work and the frame loop
7. Platform glue
8. Packaging and releases
9. The project's AGENTS.md

## 1. Workspace and layers

Split by what may depend on what, not by feature:

```
crates/core      model, commands, undo, file formats       (no egui)
crates/ui        panels, dialogs, theme, control handling  (egui only, no eframe)
apps/desktop     eframe::App, window, CLI, control server, OS glue
xtask            quality gates
```

Why: `core` stays testable with plain `cargo test`, and could serve a CLI or an
MCP server. The UI crate doesn't depend on eframe, so kittest can run it inside
a bare `egui::Context`. The app crate stays small: anything platform-specific
lives there behind `#[cfg(target_os = …)]`.

As the app grows, split `core` further (model → algorithms → formats → engine)
the way VectorCraft has eight layers, and list the layers in `xtask` so a
violation fails CI. The template's `cargo xtask layers` walks `cargo metadata`
and fails if a UI-free crate pulls in `egui`, `eframe`, `winit`, `wgpu` or `rfd`,
directly or transitively.

Workspace-level settings worth keeping from the template:

- `[workspace.dependencies]` pins the egui family in one place.
- `[workspace.lints]` with `unwrap_used`, `expect_used`, `panic`, `todo`,
  `unimplemented` denied, and `clippy.toml` allowing them in tests.
- `[profile.dev.package."*"] opt-level = 2`: dependencies optimized in debug
  builds, so the app and pixel tests run at usable speed while iterating.

## 2. Commands: one door for every action

Each user-visible action is a command: a string id, JSON params, and a
function on the session. A small registry describes them:

```rust
pub struct CommandSpec { pub id: &'static str, pub label: &'static str,
                         pub shortcut: Option<&'static str>, pub params: &'static str }
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec { id: "item.add", label: "Add Item", shortcut: None, params: "{text: string}" },
    // …
];

impl Session {
    pub fn run(&mut self, id: &str, params: &Value) -> Result<Value, String> { … }
}
```

What this buys:

- **Agents and tests can do anything a person can**, with no window: the
  control port's `app.run`, kittest set-up code, and unit tests all call `run`.
- **Undo, dirty tracking and error reporting have one home.** The template
  snapshots the document before a command and pushes it to the undo stack only
  if the command succeeded and changed something. Use structural sharing
  (`Arc` per object, or `im` collections) once documents get large, so a
  snapshot is cheap.
- **Menus, a command palette, shortcuts and docs come from the registry**, so
  they can't drift from what actually exists. VectorCraft generates its menus,
  its command palette and its MCP tool list from ~700 such specs.
- **Atomicity is testable.** Run every command with junk params and assert
  nothing panics and failed commands change nothing (the template's
  `every_command_survives_junk_params` and `invalid_params_are_refused_and_change_nothing`).

Interactive tools (drag to draw, resize handles) fit the same model by emitting
begin / preview / commit commands: preview updates a transient state without
recording undo; commit records one step.

Continuous edits in ordinary widgets (a `DragValue`, a slider, a colour picker)
change the value every frame. Two ways to keep that to one undo step:

- **Commit on release:** edit a local copy while `response.dragged()`, and run
  the command once on `response.drag_stopped()` (and on `changed()` when the
  value was typed).
- **Coalesce in the session:** when the same command hits the same target as
  the previous step (`task.setEstimate` on task 3 twice in a row), replace the
  last undo entry's "after" instead of pushing a new one. Close the run on any
  other command or after a pause.

Commands that only change UI state (open a panel, switch a tab) can live in the
UI crate with their own small registry, still reachable through the control port.

## 3. The thin UI

- Panels read state (`app.session.doc()`) and act only through `app.run(id, params)`.
- `AppUi::run` turns errors into a status-bar message. The user sees why
  something didn't happen, and agents read the same text via `app.inspect`.
- Colours, spacing and fonts come from `theme::Tokens`; `theme::apply` runs
  once per context. No hex colours in panels.
- Give every panel a function `fn show(app: &mut AppUi, ui: &mut egui::Ui)`
  in its own module, so a kittest test can render just that panel.
- Keep per-widget UI state (scroll offsets, which tab is open) either in egui's
  memory (`ui.data_mut`, collapsing state) or in a serializable `UiState`
  struct. Anything that should survive a restart goes in `UiState`.

## 4. Robustness: never crash

A crash loses unsaved work, and agents tend to write `unwrap()` by default.
The template makes the safe path the easy one:

- **Lints** deny `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` in shipped
  code. Use `?`, `.ok_or(...)?`, `let … else { return … }`, `unwrap_or_default`.
- **Implicit panics** need discipline the lints don't catch: use `.get(i)` for
  indices that come from data, avoid slicing strings at byte offsets from input,
  check divisors and float finiteness, cap allocation sizes and loop counts read
  from files.
- **Silencing is explicit.** `let _ = …` only where ignoring a failure is the
  right behaviour, with a comment saying why.
- **The guard** (`core::catch_panic`) wraps each command and the UI frame, so a
  missed case becomes an error message and a rollback. It is a safety net, not
  error handling, and does nothing on wasm (panic = abort).
- **Fuzz untrusted input.** Importers, parsers and the control protocol get
  property tests or a fuzz target. Start with the junk-params command sweep.

## 5. State, persistence, recovery

- **Preferences and UI layout:** a serde struct saved to the platform config
  dir (`directories` crate) on exit and when it changes, loaded at start with
  `unwrap_or_default` so a corrupt file never blocks launch. eframe's
  `persistence` feature (`App::save`, `cc.storage`) is fine for small apps.
- **Documents:** versioned format (`{"format": "myapp", "version": N, …}`),
  read old versions forever, write atomically (write to a temp file in the same
  directory, then rename).
- **Crash recovery:** for document apps, autosave a recovery copy in the
  background every minute or so while the document is dirty, and offer it on
  the next launch.

## 6. Background work and the frame loop

eframe only repaints when there is input or a repaint request. Rules:

- `ui()` must return quickly. Anything slow (file I/O, rendering a big
  document, network) runs on a thread. Send results over a channel, call
  `ctx.request_repaint()` from the thread (the `Context` is cheap to clone and
  `Send`), and pick them up in `logic()`.
- Animations and polling: `ctx.request_repaint_after(duration)`, not a busy loop.
- Heavy drawing: VectorCraft rasterizes its canvas on worker threads with
  `vello_cpu`, caches per object, and lets the GPU only composite a texture.
  Use `egui::Painter` shapes for ordinary UI, and textures you update
  incrementally for big canvases.
- Show progress and keep the app responsive. Prevent a second click from
  starting the same job twice.

## 7. Platform glue

Keep it in the app crate, each concern in its own module:

| Need | Crate / approach |
|---|---|
| File dialogs | `rfd`. Use `AsyncFileDialog` (or a thread) and feed the result back like any background job; a blocking dialog freezes the frame loop and on macOS can crash when shown as a sheet. |
| Clipboard text | `ctx.copy_text`, and egui's paste events. Images: `arboard` with `image-data`. |
| macOS menu bar | `muda` (Tauri's standalone menu crate), behind `cfg(target_os = "macos")`. In-window menus everywhere else. Generate both from the command registry. |
| Open URL | `ctx.open_url(egui::OpenUrl::new_tab(url))`. |
| Window size/position | `ViewportBuilder` at start; save `ctx.input(|i| i.viewport().inner_rect)` on exit. |
| Fonts | Bundle with `include_bytes!`. System fonts only as a fallback. |
| Windows console | `#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]`. |
| Accessibility | eframe's `accesskit` feature; it is also what the tests and control port rely on. |

## 8. Packaging and releases

- `cargo-packager` or `cargo-bundle` produce `.app`/`.dmg`, `.msi`, `.deb`,
  AppImage. VectorCraft uses its own `cargo xtask bundle` plus a GitHub Actions
  release matrix (universal macOS, Windows x64/arm64, Linux x86_64/aarch64,
  Flatpak).
- macOS: sign with a Developer ID and notarize, or Gatekeeper blocks the app.
  Windows: sign the `.exe` and `.msi`, or SmartScreen warns.
- Release profile: `lto = "thin"`, `codegen-units = 1`; `strip = true` if you
  don't ship debug symbols.
- Linux runtime deps to declare: `libxkbcommon-x11-0`, a Vulkan or GL driver,
  and Wayland/X11 libraries.

## 9. The project's AGENTS.md

Agents follow the rules written in the repo, so state them there. The
template's `AGENTS.md` covers layout, the non-negotiables above, the
verification ladder, and gotchas. Grow it as the project learns: when an agent
makes the same mistake twice, add a line. Keep it short enough to read every
session; move long material to `docs/` and link it.
