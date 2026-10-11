# MyApp

A desktop app built with [egui](https://github.com/emilk/egui) and eframe.

```sh
cargo run -p myapp                          # run it
cargo xtask ci                              # fmt, clippy, layer check, tests
UPDATE_SNAPSHOTS=1 cargo test -p myapp-ui   # refresh UI snapshot images after a visual change
```

Linux build dependencies (Debian/Ubuntu): `libxkbcommon-x11-0 libwayland-dev libx11-dev`.
Headless runs and snapshot tests without a GPU also need `xvfb mesa-vulkan-drivers`.

See `AGENTS.md` for the architecture and rules.
