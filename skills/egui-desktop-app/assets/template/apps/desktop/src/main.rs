//! MyApp desktop.
//!
//! Usage: `myapp [--control PORT] [--screenshot PATH] [--size WxH]`
//!
//! - `--control PORT` (or `MYAPP_CONTROL_PORT`): JSON-lines control server on 127.0.0.1, for agents
//!   and end-to-end checks. Methods: `myapp_ui::control`.
//! - `--screenshot PATH`: render the first frames, save a PNG of the window and exit (a quick look
//!   for agents, no client needed).
//!
//! Logs go to stderr: `RUST_LOG=debug` for more. The graphics adapter is logged at start-up.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod control_server;

use std::sync::mpsc::Receiver;

use myapp_core::Session;
use myapp_ui::AppUi;
use myapp_ui::control::{self, ControlRequest};
use serde_json::{Value, json};

#[derive(Default)]
struct Args {
    control: Option<u16>,
    screenshot: Option<String>,
    size: Option<[f32; 2]>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        control: std::env::var("MYAPP_CONTROL_PORT").ok().and_then(|p| p.parse().ok()),
        ..Args::default()
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || it.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--control" => args.control = Some(value()?.parse().map_err(|e| format!("--control: {e}"))?),
            "--screenshot" => args.screenshot = Some(value()?),
            "--size" => {
                let v = value()?;
                let (w, h) = v.split_once('x').ok_or("--size takes WxH, e.g. 1280x800")?;
                args.size = Some([
                    w.parse().map_err(|_| "bad width")?,
                    h.parse().map_err(|_| "bad height")?,
                ]);
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    Ok(args)
}

struct DesktopApp {
    ui: AppUi,
    /// `--screenshot`: where to save, and the pending reply once requested.
    screenshot: Option<String>,
    shot: Option<Receiver<Value>>,
}

impl eframe::App for DesktopApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.ui.raw_input_hook(raw);
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui.logic(ctx);
        self.one_shot_screenshot(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Last line of defence: a bug in a panel costs one frame and a status message, not the
        // user's unsaved work. (Not a substitute for returning errors.)
        if let Err(e) = myapp_core::catch_panic(|| self.ui.ui(ui)) {
            self.ui.status = e;
        }
    }
}

impl DesktopApp {
    fn one_shot_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = &self.screenshot else { return };
        if let Some(rx) = &self.shot {
            if let Ok(reply) = rx.try_recv() {
                println!("{reply}");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            return;
        }
        // Let layout and fonts settle for a few frames first.
        if ctx.cumulative_frame_nr() >= 5 {
            let (req, rx) = ControlRequest::new("ui.screenshot", json!({ "path": path }));
            if let Some(reply) = control::handle(&mut self.ui, ctx, &req) {
                println!("{reply}");
            }
            self.shot = Some(rx);
        }
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,myapp=info,myapp_ui=info,myapp_core=info"),
    )
    .init();
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("myapp: {e}");
            std::process::exit(2);
        }
    };
    let size = args.size.unwrap_or([960.0, 640.0]);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MyApp")
            .with_inner_size(size)
            .with_min_inner_size([360.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MyApp",
        options,
        Box::new(move |cc| {
            let mut ui = AppUi::new(Session::new());
            if let Some(render) = &cc.wgpu_render_state {
                let info = render.adapter.get_info();
                let graphics = format!("{} ({:?}, {:?})", info.name, info.backend, info.device_type);
                log::info!("graphics: {graphics}");
                ui.graphics = Some(graphics);
            }
            if let Some(port) = args.control {
                match control_server::start(port, cc.egui_ctx.clone()) {
                    Ok(rx) => {
                        control::attach(&mut ui, &cc.egui_ctx, rx);
                        log::info!("control server listening on 127.0.0.1:{port}");
                    }
                    // Keep running: the person at the keyboard doesn't need it.
                    Err(e) => log::error!("control server failed to bind 127.0.0.1:{port}: {e}"),
                }
            }
            Ok(Box::new(DesktopApp {
                ui,
                screenshot: args.screenshot,
                shot: None,
            }))
        }),
    )
}
