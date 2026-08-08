use eframe::emath::Align2;
use eframe::epaint::FontId;
use egui::{Color32, Pos2, Sense, Stroke};
use crate::theme;

pub(crate) fn player_disc(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
    painter.circle_filled(center, radius, color);
    painter.circle_stroke(center, radius, Stroke::new(radius * 0.16, theme::OUTLINE));
}

pub(crate) fn disc_button(ui: &mut egui::Ui, fill: Option<Color32>, name: &str, sub: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(78.0, 104.0), Sense::click());
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    let center = egui::pos2(rect.center().x, rect.top() + 36.0);
    let radius = 27.0 + 3.0 * t;

    match fill {
        Some(color) => {
            player_disc(&painter, center, radius, color);
        }
        None => {
            player_disc(&painter, center, radius, Color32::from_gray(90));
        }
    }
    painter.text(
        egui::pos2(rect.center().x, rect.top() + 78.0),
        Align2::CENTER_CENTER,
        name,
        FontId::proportional(14.0),
        ui.visuals().text_color(),
    );

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 95.0),
        Align2::CENTER_CENTER,
        sub,
        FontId::proportional(11.0),
        ui.visuals().weak_text_color(),
    );
    response
}