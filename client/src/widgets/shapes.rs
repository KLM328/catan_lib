use eframe::emath::Align2;
use eframe::epaint::FontId;
use egui::{Color32, Pos2, Rect, Stroke};
use crate::theme;

pub(crate) fn card(painter: &egui::Painter, rect: Rect, fill: Color32) {
    let r = rect.width() * 0.05;
    painter.rect_filled(rect, r, fill);
    painter.rect_stroke(rect, r, Stroke::new(rect.width() * 0.02, theme::OUTLINE), egui::StrokeKind::Inside);
}

pub fn badge(painter: &egui::Painter, center: Pos2, radius: f32, text: &str, bg: Color32, fg: Color32){
    painter.circle_filled(center, radius, bg);
    painter.text(
        center,
        Align2::CENTER_CENTER,
        text,
        FontId::proportional(18.0),
        fg,
    );
}