# MyApp: instructions for agents

MyApp is a cross-platform desktop app in Rust: egui for the UI, eframe for the window, wgpu for
drawing. It runs on macOS, Windows and Linux.

## Layout
- `crates/core`: the domain model and every command. No UI toolkit (`cargo xtask layers` fails if
  anything here depends on egui, eframe, winit, wgpu or rfd).
- `crates/ui`: the egui front end. Thin: panels read core state and change it only through
  `AppUi::run(id, params)`. Colours and spacing come from `theme::Tokens`.
- `apps/desktop`: the eframe window, platform glue and the control server. Thin.
- `xtask`: the quality gates.

## Non-negotiables
- **Everything is a command.** A user-visible action is a command in `crates/core` (id, label,
  params, `run`) with tests. Buttons, menus, shortcuts, the control channel and tests all call it.
- **Never crash.** No `unwrap`, `expect`, `panic!`, `todo!` in shipped code (clippy denies them;
  tests may). Untrusted input (files, clipboard, control requests) returns errors. Index with
  `.get()`, cap sizes read from input. The panic guard is a last resort, not error handling.
- **Commands are atomic.** An `Err` leaves the document exactly as it was.
- **Accessible names.** Every interactive widget has a unique name a screen reader would read:
  visible text, `labelled_by` for inputs, `widgets::set_label` for repeated buttons. Tests and the
  control channel find widgets by these names.
- **Quality gate before every commit:** `cargo xtask ci` (fmt, clippy -D warnings, layers, tests).

## Verifying your work (no human needed)
1. Core behaviour: unit tests in `crates/core` (`cargo test -p myapp-core`).
2. UI behaviour: `crates/ui/tests/ui.rs` with egui_kittest. Find widgets by role and label, click,
   type, press keys, assert on state. No window or GPU needed.
3. Look: snapshot tests render real pixels, at the normal and the minimum window size (`h.snapshot("name")`). After a visual change run
   `UPDATE_SNAPSHOTS=1 cargo test -p myapp-ui`, then open the changed PNGs in
   `crates/ui/tests/snapshots/` and check them before committing.
4. The real app: `cargo run -p myapp -- --screenshot /tmp/shot.png` for a quick look, or start it
   with `--control 7979` and drive it with JSON lines (`app.run`, `ui.widgets`, `ui.click`,
   `ui.drag`, `ui.text`, `ui.key`, `app.inspect`, `ui.screenshot`, `app.quit`). On Linux without a display,
   run it under `xvfb-run -a`. Methods: `crates/ui/src/control.rs`.

Read every screenshot you take. A passing test with a broken layout is still broken.

## Gotchas
- Linux needs `libxkbcommon-x11-0` at runtime and, without a GPU, Mesa's lavapipe
  (`mesa-vulkan-drivers`) for wgpu.
- Snapshot pixels differ slightly between GPUs and lavapipe. Make references on the platform CI
  uses; thresholds live in `kittest.toml`.
- Parallel agents: give each its own `CARGO_TARGET_DIR` and control port.
