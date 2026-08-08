use eframe::egui;
use eframe::egui::{Color32, Pos2};

pub(crate) fn draw_die(painter: &egui::Painter, center: Pos2, size: f32, value: u8, face: Color32) {
    let rect = egui::Rect::from_center_size(center, egui::vec2(size, size));
    painter.rect_filled(rect, size * 0.15, face);

    let d = size * 0.25;
    let dots: &[(f32, f32)] = match value {
        1 => &[(0.0, 0.0)],
        2 => &[(-1.0, -1.0), (1.0, 1.0)],
        3 => &[(-1.0, -1.0), (0.0, 0.0), (1.0, 1.0)],
        4 => &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)],
        5 => &[
            (-1.0, -1.0),
            (1.0, -1.0),
            (0.0, 0.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ],
        _ => &[
            (-1.0, -1.0),
            (1.0, -1.0),
            (-1.0, 0.0),
            (1.0, 0.0),
            (-1.0, 1.0),
            (1.0, 1.0),
        ],
    };
    for (dx, dy) in dots {
        painter.circle_filled(
            center + egui::vec2(dx * d, dy * d),
            size * 0.09,
            Color32::from_rgb(30, 30, 30),
        );
    }
}