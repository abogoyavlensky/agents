//! One place for colours, spacing and fonts. Panels read [`Tokens`], never hard-coded colours, so a
//! restyle (or a light theme) is one edit.

use egui::{Color32, Context, FontFamily, FontId, TextStyle};

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub accent: Color32,
    pub muted: Color32,
    pub gap: f32,
}

impl Tokens {
    pub fn get(ctx: &Context) -> Self {
        let dark = ctx.global_style().visuals.dark_mode;
        Self {
            accent: Color32::from_rgb(0x4c, 0x8d, 0xf6),
            muted: if dark {
                Color32::from_gray(150)
            } else {
                Color32::from_gray(100)
            },
            gap: 12.0,
        }
    }
}

pub fn apply(ctx: &Context) {
    // Custom fonts: bundle the files with include_bytes! and insert them into
    // egui::FontDefinitions here, so the app and the snapshot tests render the same glyphs.
    ctx.set_visuals(egui::Visuals::dark());
    let accent = Tokens::get(ctx).accent; // read before global_style_mut: it holds the context's lock
    ctx.global_style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(20.0, FontFamily::Proportional));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::new(14.0, FontFamily::Proportional));
        style.visuals.selection.bg_fill = accent;
    });
}
