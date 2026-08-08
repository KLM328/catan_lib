use eframe::egui;
use eframe::egui::{Align2, Color32, FontId, Sense, Stroke};
use catan::{Player, PlayerColor};
use crate::theme::player_color;

pub(crate) fn disc_button(ui: &mut egui::Ui, fill: Option<Color32>, name: &str, sub: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(78.0, 104.0), Sense::click());
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    let center = egui::pos2(rect.center().x, rect.top() + 36.0);
    let radius = 27.0 + 3.0 * t;

    match fill {
        Some(color) => {
            if t > 0.0 {
                painter.circle_filled(center, radius + 7.0, color.gamma_multiply(0.3 * t));
            }
            painter.circle_filled(center, radius, color);
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(2.5, Color32::from_rgb(38, 34, 30)),
            );
        }
        None => {
            let a = 90 + (60.0 * t) as u8;
            painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::from_gray(a)));
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


pub(crate) fn player_row(ui: &mut egui::Ui, player: &Player, is_current: bool) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 90.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    let color = player_color(player);

    if is_current {
        painter.rect_filled(rect, 8.0, Color32::from_gray(52));
        painter.rect_stroke(rect, 8.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    }

    // Ligne 1 : le joueur
    let line1 = rect.top() + 24.0;
    let disc = egui::pos2(rect.left() + 32.0, line1);
    painter.circle_filled(disc, 15.0, color);
    painter.circle_stroke(disc, 15.0, Stroke::new(2.5, Color32::from_rgb(38, 34, 30)));
    painter.text(
        egui::pos2(rect.left() + 58.0, line1),
        Align2::LEFT_CENTER,
        PlayerColor::color_name(player.color()),
        FontId::proportional(19.0),
        Color32::from_gray(230),
    );

    // Ligne 2 : cartes à gauche, points à droite
    let line2 = rect.bottom() - 24.0;

    let resource_card = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 24.0, line2),
        egui::vec2(14.0, 19.0),
    );
    painter.rect_filled(resource_card, 2.0, Color32::from_gray(185));
    painter.rect_stroke(resource_card, 2.0, Stroke::new(1.5, Color32::from_gray(60)),
                        egui::StrokeKind::Inside);
    painter.text(
        egui::pos2(rect.left() + 42.0, line2),
        Align2::LEFT_CENTER,
        player.hand().count().to_string(),
        FontId::proportional(16.0),
        Color32::from_gray(200),
    );

    let dev_card = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, line2),
        egui::vec2(14.0, 19.0),
    );
    painter.rect_filled(dev_card, 2.0, Color32::from_rgb(75,0,130));
    painter.rect_stroke(dev_card, 2.0, Stroke::new(1.5, Color32::from_gray(60)),
                        egui::StrokeKind::Inside);
    painter.text(
        egui::pos2(rect.center().x + 24.0, line2),
        Align2::LEFT_CENTER,
        0.to_string() ,
        FontId::proportional(16.0),
        Color32::from_gray(200),
    );

    painter.text(
        egui::pos2(rect.right() - 20.0, line2),
        Align2::RIGHT_CENTER,
        player.score().to_string(),
        FontId::proportional(26.0),
        Color32::from_gray(245),
    );
    painter.text(
        egui::pos2(rect.right() - 42.0, line2),
        Align2::RIGHT_CENTER,
        "pts",
        FontId::proportional(12.0),
        Color32::from_gray(150),
    );
}
