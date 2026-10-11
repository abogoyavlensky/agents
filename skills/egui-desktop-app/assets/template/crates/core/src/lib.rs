//! The application's domain: the document, the command registry and undo.
//!
//! Every user-visible action is a command: an id plus JSON params, run through [`Session::run`].
//! The UI, the control channel, the CLI and the tests all go through this one door, so anything a
//! person can do, an agent or a test can do too, without a window.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

mod guard;
pub use guard::catch_panic;

/// Upper bound for untrusted text (commands arrive from files, the control channel, the clipboard).
pub const MAX_TEXT: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub text: String,
    pub done: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub items: Vec<Item>,
    next_id: u64,
}

/// What a command is, for menus, the command palette, `app.commands` and docs.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub params: &'static str,
}

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        id: "item.add",
        label: "Add Item",
        shortcut: None,
        params: "{text: string}",
    },
    CommandSpec {
        id: "item.toggle",
        label: "Toggle Item",
        shortcut: None,
        params: "{id: number}",
    },
    CommandSpec {
        id: "item.remove",
        label: "Delete Item",
        shortcut: None,
        params: "{id: number}",
    },
    CommandSpec {
        id: "item.clearDone",
        label: "Clear Completed",
        shortcut: None,
        params: "{}",
    },
    CommandSpec {
        id: "edit.undo",
        label: "Undo",
        shortcut: Some("Cmd+Z"),
        params: "{}",
    },
    CommandSpec {
        id: "edit.redo",
        label: "Redo",
        shortcut: Some("Cmd+Shift+Z"),
        params: "{}",
    },
    CommandSpec {
        id: "doc.inspect",
        label: "Inspect Document",
        shortcut: None,
        params: "{}",
    },
];

#[derive(Debug, Default)]
pub struct Session {
    doc: Document,
    undo: Vec<Document>,
    redo: Vec<Document>,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn doc(&self) -> &Document {
        &self.doc
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Runs one command. Atomic: on `Err` (or a panic caught by the guard) the document is exactly
    /// as it was. A command that changes the document records one undo step.
    pub fn run(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        match id {
            "edit.undo" => return self.step(true),
            "edit.redo" => return self.step(false),
            _ => {}
        }
        let before = self.doc.clone();
        let result = catch_panic(|| execute(&mut self.doc, id, params)).and_then(|r| r);
        match result {
            Ok(value) => {
                if self.doc != before {
                    self.undo.push(before);
                    self.redo.clear();
                }
                Ok(value)
            }
            Err(e) => {
                self.doc = before;
                Err(e)
            }
        }
    }

    fn step(&mut self, undo: bool) -> Result<Value, String> {
        let (from, to) = if undo {
            (&mut self.undo, &mut self.redo)
        } else {
            (&mut self.redo, &mut self.undo)
        };
        let Some(state) = from.pop() else {
            return Err(format!("nothing to {}", if undo { "undo" } else { "redo" }));
        };
        to.push(std::mem::replace(&mut self.doc, state));
        Ok(Value::Null)
    }
}

fn execute(doc: &mut Document, id: &str, p: &Value) -> Result<Value, String> {
    match id {
        "item.add" => {
            let text = str_param(p, "text")?.trim();
            if text.is_empty() {
                return Err("`text` is empty".into());
            }
            if text.len() > MAX_TEXT {
                return Err(format!("`text` is longer than {MAX_TEXT} bytes"));
            }
            doc.next_id += 1;
            doc.items.push(Item {
                id: doc.next_id,
                text: text.to_string(),
                done: false,
            });
            Ok(json!({ "id": doc.next_id }))
        }
        "item.toggle" => {
            let id = id_param(p)?;
            let item = doc
                .items
                .iter_mut()
                .find(|i| i.id == id)
                .ok_or(format!("no item {id}"))?;
            item.done = !item.done;
            Ok(json!({ "done": item.done }))
        }
        "item.remove" => {
            let id = id_param(p)?;
            let index = doc
                .items
                .iter()
                .position(|i| i.id == id)
                .ok_or(format!("no item {id}"))?;
            doc.items.remove(index);
            Ok(Value::Null)
        }
        "item.clearDone" => {
            let before = doc.items.len();
            doc.items.retain(|i| !i.done);
            Ok(json!({ "removed": before - doc.items.len() }))
        }
        "doc.inspect" => serde_json::to_value(&doc.items).map_err(|e| e.to_string()),
        other => Err(format!("unknown command `{other}`")),
    }
}

fn str_param<'a>(p: &'a Value, key: &str) -> Result<&'a str, String> {
    p.get(key)
        .and_then(Value::as_str)
        .ok_or(format!("missing string `{key}`"))
}

fn id_param(p: &Value) -> Result<u64, String> {
    p.get("id")
        .and_then(Value::as_u64)
        .ok_or("missing number `id`".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_toggle_remove_with_undo() {
        let mut s = Session::new();
        let id = s.run("item.add", &json!({"text": " Buy milk "})).unwrap()["id"]
            .as_u64()
            .unwrap();
        assert_eq!(s.doc().items[0].text, "Buy milk");
        s.run("item.toggle", &json!({"id": id})).unwrap();
        assert!(s.doc().items[0].done);
        s.run("item.remove", &json!({"id": id})).unwrap();
        assert!(s.doc().items.is_empty());
        s.run("edit.undo", &json!({})).unwrap();
        assert_eq!(s.doc().items.len(), 1);
        s.run("edit.redo", &json!({})).unwrap();
        assert!(s.doc().items.is_empty());
    }

    #[test]
    fn invalid_params_are_refused_and_change_nothing() {
        let mut s = Session::new();
        s.run("item.add", &json!({"text": "a"})).unwrap();
        let before = s.doc().clone();
        for (id, p) in [
            ("item.add", json!({})),
            ("item.add", json!({"text": "   "})),
            ("item.add", json!({"text": "x".repeat(MAX_TEXT + 1)})),
            ("item.toggle", json!({"id": "1"})),
            ("item.remove", json!({"id": 999})),
            ("nope", json!({})),
        ] {
            assert!(s.run(id, &p).is_err(), "{id} {p}");
            assert_eq!(s.doc(), &before);
        }
    }

    #[test]
    fn every_command_survives_junk_params() {
        // A tiny "command sweep": every registered command, fed junk, must answer Ok or Err, never panic.
        let junk = [
            json!(null),
            json!({}),
            json!([]),
            json!({"id": -1}),
            json!({"text": 42}),
            json!({"id": u64::MAX}),
        ];
        let mut s = Session::new();
        for c in COMMANDS {
            for p in &junk {
                let _ = s.run(c.id, p);
            }
        }
    }
}
