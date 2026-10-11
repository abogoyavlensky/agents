//! Headless UI tests: the real `AppUi::ui` code in an egui context with no window.
//! Widgets are found the way a screen reader finds them (AccessKit), input is simulated, and
//! snapshot tests render real pixels through wgpu (a software Vulkan driver works in CI).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use myapp_core::Session;
use myapp_ui::AppUi;
use myapp_ui::control::{self, ControlRequest, WidgetNode};
use serde_json::{Value, json};

fn harness(app: AppUi) -> Harness<'static, AppUi> {
    let mut h = Harness::builder().with_size(egui::vec2(480.0, 320.0)).build_ui_state(
        |ui, app: &mut AppUi| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        app,
    );
    h.run();
    h
}

fn with_items(texts: &[&str]) -> AppUi {
    let mut app = AppUi::new(Session::new());
    for t in texts {
        app.run("item.add", json!({ "text": t })).unwrap();
    }
    app
}

fn texts(h: &Harness<'_, AppUi>) -> Vec<String> {
    h.state().session.doc().items.iter().map(|i| i.text.clone()).collect()
}

#[test]
fn typing_and_clicking_add_creates_an_item() {
    let mut h = harness(AppUi::new(Session::new()));
    let input = h.get_by_role_and_label(Role::TextInput, "New item");
    input.focus();
    input.type_text("Buy milk");
    h.run();
    h.get_by_label("Add").click();
    h.run();
    assert_eq!(texts(&h), ["Buy milk"]);
    assert_eq!(h.state().draft, "");
    // The new row is on screen, found by its label.
    h.get_by_role_and_label(Role::CheckBox, "Buy milk");
}

#[test]
fn enter_submits_and_an_empty_item_is_refused_with_a_message() {
    let mut h = harness(AppUi::new(Session::new()));
    h.get_by_label("Add").click();
    h.run();
    assert!(texts(&h).is_empty());
    assert!(h.state().status.contains("empty"), "status: {}", h.state().status);

    let input = h.get_by_role_and_label(Role::TextInput, "New item");
    input.focus();
    input.type_text("Call mom");
    h.run();
    h.key_press(Key::Enter);
    h.run();
    assert_eq!(texts(&h), ["Call mom"]);
}

#[test]
fn rows_toggle_and_delete_by_their_own_labels() {
    let mut h = harness(with_items(&["One", "Two"]));
    h.get_by_role_and_label(Role::CheckBox, "Two").click();
    h.run();
    assert!(h.state().session.doc().items[1].done);
    h.get_by_label("Delete One").click();
    h.run();
    assert_eq!(texts(&h), ["Two"]);
}

#[test]
fn undo_button_and_shortcut() {
    let mut h = harness(AppUi::new(Session::new()));
    assert!(h.get_by_label("Undo").accesskit_node().is_disabled());
    h.state_mut().run("item.add", json!({"text": "x"})).unwrap();
    h.run();
    assert!(!h.get_by_label("Undo").accesskit_node().is_disabled());
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run();
    assert!(texts(&h).is_empty());
    h.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
    h.run();
    assert_eq!(texts(&h), ["x"]);
}

/// The control channel, end to end but without a window or socket: requests go through the same
/// `handle` the desktop app uses, and injected input is fed to the harness frame by frame.
#[test]
fn control_channel_drives_the_ui() {
    let mut h = harness(AppUi::new(Session::new()));
    let (tx, rx) = std::sync::mpsc::channel();
    let ctx = h.ctx.clone();
    control::attach(h.state_mut(), &ctx, rx);
    h.run();

    let call = |h: &mut Harness<'_, AppUi>, method: &str, params: Value| -> Value {
        let (req, reply) = ControlRequest::new(method, params);
        tx.send(req).unwrap();
        h.run_steps(1); // `logic` drains the request
        // Feed injected input one event per frame, as the app's raw_input_hook does.
        while let Some(event) = h.state_mut().synthetic.pop_front() {
            h.event(event);
            h.run_steps(1);
        }
        h.run();
        reply.try_recv().unwrap()
    };

    // A text field is found by the label it is `labelled_by`, focused by a click, then typed into.
    assert_eq!(call(&mut h, "ui.click", json!({"label": "New item"}))["ok"], true);
    call(&mut h, "ui.text", json!({"text": "Typed"}));
    call(&mut h, "ui.key", json!({"key": "Enter"}));
    assert_eq!(texts(&h), ["Typed"]);

    let r = call(
        &mut h,
        "app.run",
        json!({"command": "item.add", "params": {"text": "Write tests"}}),
    );
    assert_eq!(r["ok"], true, "{r}");
    let widgets = call(&mut h, "ui.widgets", json!({"label": "Write tests"}));
    assert!(!widgets["result"].as_array().unwrap().is_empty(), "{widgets}");

    let r = call(&mut h, "ui.click", json!({"label": "Write tests"}));
    assert_eq!(r["ok"], true, "{r}");
    assert!(h.state().session.doc().items[1].done);

    let r = call(&mut h, "ui.drag", json!({"label": "Add", "dx": 30, "steps": 3}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(
        r["result"]["to"][0].as_f64().unwrap() - r["result"]["from"][0].as_f64().unwrap(),
        30.0
    );

    let r = call(&mut h, "ui.click", json!({"label": "No such widget"}));
    assert_eq!(r["ok"], false);
    let r = call(&mut h, "app.inspect", json!({}));
    assert_eq!(r["result"]["doc"]["items"][1]["done"], true);
}

fn snapshot_at(name: &str, size: egui::Vec2) {
    let mut app = with_items(&[
        "Buy milk",
        "Walk the dog",
        "Ship v1 of the project with a title far too long to fit in a small window",
    ]);
    app.run("item.toggle", json!({"id": 2})).unwrap();
    let mut h = Harness::builder()
        .with_size(size)
        .wgpu()
        .build_ui_state(|ui, app: &mut AppUi| app.ui(ui), app);
    h.run();
    h.snapshot(name);
}

/// Pixel snapshots: render with wgpu and compare with tests/snapshots/<name>.png.
/// `UPDATE_SNAPSHOTS=1 cargo test` writes new references; look at them before committing.
#[test]
fn todo_list_snapshot() {
    snapshot_at("todo_list", egui::vec2(480.0, 320.0));
}

/// The smallest window the app allows (`with_min_inner_size` in apps/desktop): nothing clipped.
#[test]
fn todo_list_min_size_snapshot() {
    snapshot_at("todo_list_min_size", egui::vec2(360.0, 240.0));
}

#[test]
fn a_label_finds_the_input_it_names_whatever_the_tree_order() {
    let node = |role: &str, x: f32| WidgetNode {
        role: role.into(),
        label: Some("New item".into()),
        value: None,
        rect: [x, 0.0, 10.0, 10.0],
        disabled: false,
    };
    for widgets in [
        vec![node("Label", 0.0), node("TextInput", 50.0)],
        vec![node("TextInput", 50.0), node("Label", 0.0)],
    ] {
        assert_eq!(control::find_widget(&widgets, "New item").unwrap().role, "TextInput");
        assert_eq!(control::find_widget(&widgets, "New").unwrap().role, "TextInput");
    }
}

/// Widgets that clear their own accessible label (DragValue) can still be named per row.
#[test]
fn drag_values_are_named_and_dragged() {
    let mut h = Harness::builder().with_size(egui::vec2(300.0, 100.0)).build_ui_state(
        |ui, value: &mut f32| {
            let r = ui.add(egui::DragValue::new(value).range(0.0..=100.0));
            myapp_ui::widgets::set_name(&r, "Estimate for Buy milk");
        },
        10.0_f32,
    );
    h.run();
    let at = h
        .get_by_role_and_label(Role::SpinButton, "Estimate for Buy milk")
        .rect()
        .center();
    h.event(egui::Event::PointerMoved(at));
    h.run_steps(1);
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(1);
    for i in 1..=8 {
        h.event(egui::Event::PointerMoved(at + egui::vec2(5.0 * i as f32, 0.0)));
        h.run_steps(1);
    }
    h.event(egui::Event::PointerButton {
        pos: at + egui::vec2(40.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run();
    assert!(*h.state() > 10.0, "value {}", h.state());
}
