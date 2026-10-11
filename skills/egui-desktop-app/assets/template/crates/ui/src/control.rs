//! Programmatic control of the running app, for agents and end-to-end checks.
//!
//! The transport (a loopback TCP server, see `apps/desktop/src/control_server.rs`) only moves JSON;
//! everything is handled here, on the UI thread, between frames. Methods:
//!
//! - `app.commands`: the command registry
//! - `app.run {command, params?}`: run any command (the same path as the UI)
//! - `app.inspect`: document, UI state, status line, window size
//! - `ui.widgets {label?}`: the AccessKit tree as `[{role, label, value, rect:[x,y,w,h], disabled}]`
//!   in egui points, optionally filtered by a label substring
//! - `ui.click {label} | {x, y}` (`button?: "right"`, `cmd?`, `shift?`, `alt?`): a real egui click.
//!   A label names the widget a screen reader would: an input wins over the Label naming it.
//! - `ui.drag {label} | {x, y}` plus `{toX, toY} | {dx, dy}`, `steps?` (default 8): press, move, release
//! - `ui.key {key, cmd?, shift?, alt?}`, `ui.text {text}`: keyboard input (to the focused widget)
//!
//!   Input requests reply once the input was delivered and a frame ran, so requests are sequential:
//!   an `app.inspect` or `ui.screenshot` sent next sees the effect.
//! - `ui.screenshot {path}`: PNG of the next presented frame (answered once it is captured)
//! - `app.quit`

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use egui::{Context, Event, Modifiers, PointerButton, pos2};
use serde::Serialize;
use serde_json::{Value, json};

use crate::AppUi;

pub struct ControlRequest {
    pub method: String,
    pub params: Value,
    pub reply: Sender<Value>,
}

impl ControlRequest {
    pub fn new(method: impl Into<String>, params: Value) -> (Self, Receiver<Value>) {
        let (reply, rx) = std::sync::mpsc::channel();
        (
            Self {
                method: method.into(),
                params,
                reply,
            },
            rx,
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct WidgetNode {
    pub role: String,
    pub label: Option<String>,
    pub value: Option<String>,
    /// `[x, y, width, height]` in egui points (the coordinates `ui.click` takes).
    pub rect: [f32; 4],
    pub disabled: bool,
}

#[derive(Default)]
pub struct ControlState {
    rx: Option<Receiver<ControlRequest>>,
    widgets: Arc<Mutex<Vec<WidgetNode>>>,
    shots: Vec<PendingShot>,
    /// Replies to input requests, sent once the injected input has been delivered.
    input_waiters: Vec<(Sender<Value>, Value)>,
    /// Frames since the injected input queue went empty.
    idle_frames: u32,
    next_token: u64,
    pub quit_requested: bool,
}

struct PendingShot {
    token: u64,
    path: String,
    reply: Sender<Value>,
    /// egui time (seconds) after which we give up: no frame was presented.
    deadline: f64,
}

/// Connects a request channel to the app and starts recording the widget tree.
pub fn attach(app: &mut AppUi, ctx: &Context, rx: Receiver<ControlRequest>) {
    app.control.rx = Some(rx);
    ctx.enable_accesskit();
    ctx.add_plugin(WidgetTree(Arc::clone(&app.control.widgets)));
}

/// Records each frame's AccessKit nodes so `ui.widgets` and `ui.click {label}` can find widgets
/// the way a screen reader (or an egui_kittest query) does.
struct WidgetTree(Arc<Mutex<Vec<WidgetNode>>>);

impl egui::Plugin for WidgetTree {
    fn debug_name(&self) -> &'static str {
        "control-widget-tree"
    }

    fn output_hook(&mut self, _ctx: &Context, output: &mut egui::FullOutput) {
        let Some(update) = &output.platform_output.accesskit_update else {
            return;
        };
        // A node's name as a screen reader says it: its label, a Label's text (egui stores that
        // as the value), or the text of the widget it is `labelled_by` (a TextEdit's label).
        let own_text = |n: &egui::accesskit::Node| {
            n.label()
                .or_else(|| (n.role() == egui::accesskit::Role::Label).then(|| n.value()).flatten())
                .map(str::to_string)
        };
        let texts: HashMap<_, _> = update
            .nodes
            .iter()
            .filter_map(|(id, n)| Some((*id, own_text(n)?)))
            .collect();
        let nodes = update
            .nodes
            .iter()
            .filter_map(|(id, n)| {
                let b = n.bounds()?;
                let label = texts.get(id).cloned().or_else(|| {
                    let names: Vec<_> = n.labelled_by().iter().filter_map(|l| texts.get(l).cloned()).collect();
                    (!names.is_empty()).then(|| names.join(" "))
                });
                let value = n.value().map(str::to_string);
                if label.is_none() && value.is_none() {
                    return None;
                }
                let rect = [b.x0 as f32, b.y0 as f32, (b.x1 - b.x0) as f32, (b.y1 - b.y0) as f32];
                Some(WidgetNode {
                    role: format!("{:?}", n.role()),
                    label,
                    value,
                    rect,
                    disabled: n.is_disabled(),
                })
            })
            .collect();
        if let Ok(mut widgets) = self.0.lock() {
            *widgets = nodes;
        }
    }
}

/// The widget a label names: exact matches before substring matches, and within each, an
/// interactive widget before static text. A text field and the Label naming it share a name
/// (`labelled_by`), and the AccessKit update lists nodes in no particular order.
pub fn find_widget<'a>(widgets: &'a [WidgetNode], label: &str) -> Option<&'a WidgetNode> {
    let is_text = |w: &WidgetNode| matches!(w.role.as_str(), "Label" | "TextRun" | "Paragraph");
    let exact = |w: &&WidgetNode| w.label.as_deref() == Some(label);
    let partial = |w: &&WidgetNode| w.label.as_deref().is_some_and(|l| l.contains(label));
    let pick = |matches: Vec<&'a WidgetNode>| matches.iter().find(|w| !is_text(w)).or(matches.first()).copied();
    pick(widgets.iter().filter(exact).collect()).or_else(|| pick(widgets.iter().filter(partial).collect()))
}

fn center(w: &WidgetNode) -> egui::Pos2 {
    pos2(w.rect[0] + w.rect[2] / 2.0, w.rect[1] + w.rect[3] / 2.0)
}

/// The point a request aims at: `{label}` (the widget's centre) or `{x, y}`.
fn aim(app: &AppUi, p: &Value) -> Result<egui::Pos2, String> {
    if let Some(label) = p.get("label").and_then(Value::as_str) {
        let widgets = app.control.widgets.lock().map(|w| w.clone()).unwrap_or_default();
        return find_widget(&widgets, label)
            .map(center)
            .ok_or(format!("no widget labelled {label:?} (see ui.widgets)"));
    }
    let f = |k: &str| p.get(k).and_then(Value::as_f64);
    match (f("x"), f("y")) {
        (Some(x), Some(y)) => Ok(pos2(x as f32, y as f32)),
        _ => Err("give `label` or `x` and `y`".into()),
    }
}

pub(crate) fn drain(app: &mut AppUi, ctx: &Context) {
    let Some(rx) = app.control.rx.take() else { return };
    while let Ok(req) = rx.try_recv() {
        if let Some(reply) = handle(app, ctx, &req) {
            let _ = req.reply.send(reply); // the client may have gone; nothing to do then
        }
    }
    app.control.rx = Some(rx);

    // Input requests answer after their events were fed to egui and one more frame ran, so the
    // next request (an inspect, a screenshot) sees their effect.
    if app.synthetic.is_empty() {
        app.control.idle_frames = app.control.idle_frames.saturating_add(1);
        if app.control.idle_frames >= 2 {
            for (reply, value) in app.control.input_waiters.drain(..) {
                let _ = reply.send(value);
            }
        }
    } else {
        app.control.idle_frames = 0;
    }
    if !app.control.input_waiters.is_empty() {
        ctx.request_repaint();
    }
}

/// Defers the reply of a request that injected input (see [`drain`]).
fn after_input(app: &mut AppUi, req: &ControlRequest, result: Value) -> Option<Value> {
    app.control.idle_frames = 0;
    app.control
        .input_waiters
        .push((req.reply.clone(), json!({ "ok": true, "result": result })));
    None
}

fn ok(v: Value) -> Option<Value> {
    Some(json!({ "ok": true, "result": v }))
}

fn err(e: impl std::fmt::Display) -> Option<Value> {
    Some(json!({ "ok": false, "error": e.to_string() }))
}

fn modifiers(p: &Value) -> Modifiers {
    let b = |k: &str| p.get(k).and_then(Value::as_bool).unwrap_or(false);
    Modifiers {
        alt: b("alt"),
        ctrl: b("ctrl"),
        shift: b("shift"),
        mac_cmd: b("cmd") && cfg!(target_os = "macos"),
        command: b("cmd"),
    }
}

/// Answers a request now (`Some`) or later (`None`: the reply is sent when the frame arrives).
pub fn handle(app: &mut AppUi, ctx: &Context, req: &ControlRequest) -> Option<Value> {
    let p = &req.params;
    let s = |k: &str| p.get(k).and_then(Value::as_str);
    ctx.request_repaint();
    match req.method.as_str() {
        "app.commands" => ok(serde_json::to_value(myapp_core::COMMANDS).unwrap_or_default()),
        "app.run" => {
            let Some(id) = s("command") else {
                return err("missing `command`");
            };
            let params = p.get("params").cloned().unwrap_or(json!({}));
            match app.run(id, params) {
                Ok(v) => ok(v),
                Err(e) => err(e),
            }
        }
        "app.inspect" => ok(json!({
            "doc": app.session.doc(),
            "draft": app.draft,
            "status": app.status,
            "canUndo": app.session.can_undo(),
            "graphics": app.graphics,
            "window": [ctx.content_rect().width(), ctx.content_rect().height()],
        })),
        "ui.widgets" => {
            let widgets = app.control.widgets.lock().map(|w| w.clone()).unwrap_or_default();
            let filter = s("label");
            let found: Vec<_> = widgets
                .into_iter()
                .filter(|w| filter.is_none_or(|f| w.label.as_deref().is_some_and(|l| l.contains(f))))
                .collect();
            ok(serde_json::to_value(found).unwrap_or_default())
        }
        "ui.click" => {
            let at = match aim(app, p) {
                Ok(at) => at,
                Err(e) => return err(e),
            };
            let button = if s("button") == Some("right") {
                PointerButton::Secondary
            } else {
                PointerButton::Primary
            };
            let modifiers = modifiers(p);
            app.synthetic.push_back(Event::PointerMoved(at));
            app.synthetic.push_back(Event::PointerButton {
                pos: at,
                button,
                pressed: true,
                modifiers,
            });
            app.synthetic.push_back(Event::PointerButton {
                pos: at,
                button,
                pressed: false,
                modifiers,
            });
            after_input(app, req, json!({ "x": at.x, "y": at.y }))
        }
        "ui.drag" => {
            let from = match aim(app, p) {
                Ok(at) => at,
                Err(e) => return err(e),
            };
            let f = |k: &str| p.get(k).and_then(Value::as_f64).map(|v| v as f32);
            let to = match (f("toX"), f("toY"), f("dx"), f("dy")) {
                (Some(x), Some(y), _, _) => pos2(x, y),
                (_, _, dx, dy) if dx.is_some() || dy.is_some() => {
                    from + egui::vec2(dx.unwrap_or(0.0), dy.unwrap_or(0.0))
                }
                _ => return err("give `toX` and `toY`, or `dx`/`dy`"),
            };
            let steps = p.get("steps").and_then(Value::as_u64).unwrap_or(8).clamp(1, 100);
            let (button, modifiers) = (PointerButton::Primary, modifiers(p));
            app.synthetic.push_back(Event::PointerMoved(from));
            app.synthetic.push_back(Event::PointerButton {
                pos: from,
                button,
                pressed: true,
                modifiers,
            });
            for i in 1..=steps {
                app.synthetic
                    .push_back(Event::PointerMoved(from + (to - from) * (i as f32 / steps as f32)));
            }
            app.synthetic.push_back(Event::PointerButton {
                pos: to,
                button,
                pressed: false,
                modifiers,
            });
            after_input(app, req, json!({ "from": [from.x, from.y], "to": [to.x, to.y] }))
        }
        "ui.key" => {
            let Some(key) = s("key").and_then(egui::Key::from_name) else {
                return err("unknown or missing `key`");
            };
            let modifiers = modifiers(p);
            for pressed in [true, false] {
                app.synthetic.push_back(Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                });
            }
            after_input(app, req, Value::Null)
        }
        "ui.text" => {
            app.synthetic
                .push_back(Event::Text(s("text").unwrap_or_default().to_string()));
            after_input(app, req, Value::Null)
        }
        "ui.screenshot" => {
            let Some(path) = s("path") else {
                return err("missing `path`");
            };
            app.control.next_token += 1;
            let token = app.control.next_token;
            let deadline = ctx.input(|i| i.time) + 5.0;
            app.control.shots.push(PendingShot {
                token,
                path: path.to_string(),
                reply: req.reply.clone(),
                deadline,
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(token)));
            None
        }
        "app.quit" => {
            app.control.quit_requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            ok(Value::Null)
        }
        other => err(format!("unknown method `{other}`")),
    }
}

/// Answers pending `ui.screenshot` requests with the frames eframe captured.
pub(crate) fn collect_screenshots(app: &mut AppUi, ctx: &Context) {
    if app.control.shots.is_empty() {
        return;
    }
    let captured: Vec<(u64, Arc<egui::ColorImage>)> = ctx.input(|i| {
        i.raw
            .events
            .iter()
            .filter_map(|e| match e {
                Event::Screenshot { user_data, image, .. } => {
                    let token = user_data.data.as_ref()?.downcast_ref::<u64>().copied()?;
                    Some((token, Arc::clone(image)))
                }
                _ => None,
            })
            .collect()
    });
    for (token, image) in captured {
        if let Some(i) = app.control.shots.iter().position(|s| s.token == token) {
            let shot = app.control.shots.remove(i);
            let reply = match save_png(&image, &shot.path) {
                Ok(()) => json!({ "ok": true, "result": { "path": shot.path, "size": image.size } }),
                Err(e) => json!({ "ok": false, "error": e }),
            };
            let _ = shot.reply.send(reply);
        }
    }
    let now = ctx.input(|i| i.time);
    app.control.shots.retain(|s| {
        if now < s.deadline {
            return true;
        }
        let _ = s
            .reply
            .send(json!({ "ok": false, "error": "no frame was presented (window hidden or minimized?)" }));
        false
    });
    ctx.request_repaint();
}

pub fn save_png(image: &egui::ColorImage, path: &str) -> Result<(), String> {
    let [w, h] = image.size;
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let buffer = image::RgbaImage::from_raw(w as u32, h as u32, bytes).ok_or("bad image size")?;
    buffer.save(path).map_err(|e| format!("{path}: {e}"))
}
