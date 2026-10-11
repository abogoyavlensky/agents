//! Loopback JSON-lines transport for `myapp_ui::control`: one request per line, one reply per line.
//!
//! `{"id":1,"method":"app.inspect","params":{}}` -> `{"id":1,"ok":true,"result":{...}}`
//!
//! Only 127.0.0.1. Lines are bounded, and a line that isn't a request object closes the connection,
//! so a browser's stray cross-origin request can't drive the app.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use myapp_ui::control::ControlRequest;
use serde_json::{Value, json};

const MAX_LINE: u64 = 4 * 1024 * 1024;
const REPLY_TIMEOUT: Duration = Duration::from_secs(30);

pub fn start(port: u16, ctx: egui::Context) -> std::io::Result<Receiver<ControlRequest>> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let (tx, ctx) = (tx.clone(), ctx.clone());
            std::thread::spawn(move || serve(stream, &tx, &ctx));
        }
    });
    Ok(rx)
}

fn serve(stream: TcpStream, tx: &Sender<ControlRequest>, ctx: &egui::Context) {
    let Ok(mut out) = stream.try_clone() else { return };
    let mut reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        match reader.by_ref().take(MAX_LINE + 1).read_line(&mut line) {
            Ok(0) | Err(_) => return,
            Ok(n) if n as u64 > MAX_LINE => {
                let _ = writeln!(out, "{}", json!({"ok": false, "error": "line too long"}));
                return;
            }
            Ok(_) => {}
        }
        if line.trim().is_empty() {
            continue;
        }
        let parsed: Option<Value> = serde_json::from_str(&line).ok();
        let Some((id, method, params)) = parsed.as_ref().and_then(|v| {
            Some((
                v.get("id").cloned().unwrap_or(Value::Null),
                v.get("method")?.as_str()?.to_string(),
                v.get("params").cloned(),
            ))
        }) else {
            let _ = writeln!(
                out,
                "{}",
                json!({"ok": false, "error": "expected {\"id\", \"method\", \"params\"}"})
            );
            return;
        };
        let (req, rx) = ControlRequest::new(method, params.unwrap_or(json!({})));
        if tx.send(req).is_err() {
            return; // the app is gone
        }
        ctx.request_repaint(); // wake the event loop: requests are handled between frames
        let mut reply = rx
            .recv_timeout(REPLY_TIMEOUT)
            .unwrap_or_else(|_| json!({"ok": false, "error": "timeout"}));
        if let Some(obj) = reply.as_object_mut() {
            obj.insert("id".into(), id);
        }
        if writeln!(out, "{reply}").is_err() {
            return;
        }
    }
}
