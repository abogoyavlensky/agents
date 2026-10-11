//! The egui front end. It owns no domain logic: it draws [`Session`] state and changes it only
//! through [`AppUi::run`], the same door the control channel and tests use.

pub mod control;
pub mod theme;
pub mod widgets;

use std::collections::VecDeque;

use egui::{Key, KeyboardShortcut, Modifiers};
use myapp_core::Session;
use serde_json::{Value, json};

pub struct AppUi {
    pub session: Session,
    /// Text in the "New item" field.
    pub draft: String,
    /// The status bar: the last error or confirmation. Agents read it through `app.inspect`.
    pub status: String,
    /// Input injected by the control channel, fed to egui by [`Self::raw_input_hook`].
    pub synthetic: VecDeque<egui::Event>,
    /// Control requests waiting for a frame (screenshots).
    pub control: control::ControlState,
    /// The graphics adapter the window draws with (set by the desktop app), for `app.inspect`.
    pub graphics: Option<String>,
    synthetic_modifiers: bool,
    themed: bool,
}

impl AppUi {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            draft: String::new(),
            status: String::new(),
            synthetic: VecDeque::new(),
            control: control::ControlState::default(),
            graphics: None,
            synthetic_modifiers: false,
            themed: false,
        }
    }

    /// Runs a command and reports failures in the status bar instead of failing loudly.
    pub fn run(&mut self, id: &str, params: Value) -> Result<Value, String> {
        let result = self.session.run(id, &params);
        match &result {
            Ok(_) => self.status.clear(),
            Err(e) => self.status = e.clone(),
        }
        result
    }

    /// Feeds injected input to egui at a human pace: each pointer move, press and release gets
    /// its own frame, as with a real mouse, so widgets see a proper click or drag.
    ///
    /// egui reads held modifiers from `ModifiersChanged` events, so an injected Cmd+Z holds Cmd for
    /// its frames and lets go once the queue is empty.
    pub fn raw_input_hook(&mut self, raw: &mut egui::RawInput) {
        if self.synthetic.is_empty() {
            if std::mem::take(&mut self.synthetic_modifiers) {
                raw.events.push(egui::Event::ModifiersChanged(Modifiers::NONE));
            }
            return;
        }
        while let Some(event) = self.synthetic.pop_front() {
            if let egui::Event::Key { modifiers, .. } | egui::Event::PointerButton { modifiers, .. } = &event {
                raw.events.push(egui::Event::ModifiersChanged(*modifiers));
                self.synthetic_modifiers = true;
            }
            let ends_frame = matches!(
                event,
                egui::Event::PointerButton { .. } | egui::Event::Key { pressed: false, .. }
            );
            raw.events.push(event);
            if ends_frame {
                break;
            }
        }
    }

    /// Everything that isn't drawing: control requests, background work. Call once per frame.
    pub fn logic(&mut self, ctx: &egui::Context) {
        control::drain(self, ctx);
        if !self.synthetic.is_empty() {
            ctx.request_repaint();
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        if !self.themed {
            theme::apply(ui.ctx());
            self.themed = true;
        }
        control::collect_screenshots(self, ui.ctx());
        self.shortcuts(ui.ctx());

        let tokens = theme::Tokens::get(ui.ctx());
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Todos");
                ui.add_space(tokens.gap);
                if ui
                    .add_enabled(self.session.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    let _ = self.run("edit.undo", json!({})); // failures show in the status bar
                }
                if ui
                    .add_enabled(self.session.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    let _ = self.run("edit.redo", json!({}));
                }
            });
        });
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(egui::RichText::new(&self.status).color(tokens.muted));
        });
        egui::CentralPanel::default().show(ui, |ui| {
            self.add_row(ui);
            ui.separator();
            self.item_list(ui);
        });
    }

    fn add_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = ui.label("New item");
            // The field takes the width the button leaves, so the row never runs off screen.
            let button_width = 56.0;
            let width = (ui.available_width() - button_width - ui.spacing().item_spacing.x).max(80.0);
            let edit = ui
                .add(
                    egui::TextEdit::singleline(&mut self.draft)
                        .hint_text("What needs doing?")
                        .desired_width(width),
                )
                .labelled_by(label.id);
            let submitted = edit.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            let add = ui.add_sized([button_width, ui.spacing().interact_size.y], egui::Button::new("Add"));
            if add.clicked() || submitted {
                let text = std::mem::take(&mut self.draft);
                if self.run("item.add", json!({ "text": text })).is_err() {
                    self.draft = text; // keep what the user typed
                }
                edit.request_focus();
            }
        });
    }

    fn item_list(&mut self, ui: &mut egui::Ui) {
        // Clone the small view model so the loop can call `self.run`. For big documents, collect
        // the clicked action in the loop instead and run it after.
        let items = self.session.doc().items.clone();
        if items.is_empty() {
            ui.weak("Nothing to do.");
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for item in &items {
                // Stable ids from the data, never from the position or the text.
                ui.push_id(item.id, |ui| {
                    // Row recipe: fixed height, the button pinned right, the title taking what is
                    // left and truncated with "…" instead of pushing the button off screen.
                    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                    ui.allocate_ui_with_layout(size, egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let delete = ui.small_button("Delete");
                        // A unique accessible name: screen readers, kittest and `ui.widgets` can
                        // tell the rows' buttons apart.
                        widgets::set_label(&delete, egui::WidgetType::Button, format!("Delete {}", item.text));
                        if delete.clicked() {
                            let _ = self.run("item.remove", json!({ "id": item.id }));
                        }
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                            let mut done = item.done;
                            if ui.checkbox(&mut done, &item.text).changed() {
                                let _ = self.run("item.toggle", json!({ "id": item.id }));
                            }
                        });
                    });
                });
            }
        });
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        // Longest modifier set first: Cmd+Shift+Z would also match Cmd+Z.
        let redo = KeyboardShortcut::new(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
        let undo = KeyboardShortcut::new(Modifiers::COMMAND, Key::Z);
        if ctx.input_mut(|i| i.consume_shortcut(&redo)) {
            let _ = self.run("edit.redo", json!({}));
        } else if ctx.input_mut(|i| i.consume_shortcut(&undo)) {
            let _ = self.run("edit.undo", json!({}));
        }
    }
}
