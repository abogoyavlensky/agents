//! Small helpers shared by panels.

/// Gives a widget its own accessible name (the one AccessKit, kittest queries and the control
/// channel's `ui.widgets` see), for widgets whose visible text repeats, like a "Delete" per row.
pub fn set_label(response: &egui::Response, kind: egui::WidgetType, label: impl Into<String>) {
    let label = label.into();
    let enabled = response.enabled();
    response.widget_info(|| egui::WidgetInfo::labeled(kind, enabled, label.clone()));
}

/// Names a widget that sets (or clears) its own accessible label, such as a `DragValue`, whose
/// node is a `Role::SpinButton` with no label. Use it for per-row inputs, where `labelled_by` a
/// shared column header would give every row the same name.
pub fn set_name(response: &egui::Response, name: impl Into<String>) {
    let name = name.into();
    response
        .ctx
        .accesskit_node_builder(response.id, |node| node.set_label(name));
}
