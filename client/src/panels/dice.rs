use eframe::egui;
use eframe::egui::{Align2, Color32, Sense, Ui};
use catan::{Game, GameStatus};
use crate::{CatanApp, UiAction};
use crate::widgets::draw_die;

pub(crate) fn show(ui: &mut Ui, game: &Game, app : &CatanApp) -> Vec<UiAction> {
    let mut actions = Vec::new();

    egui::Area::new(egui::Id::new("dices"))
        .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-324.0, -24.0))
        .show(ui.ctx(), |ui| {
            let base = match game.status() {
                GameStatus::AwaitingRoll => 120.0,
                GameStatus::AwaitingSteal
                | GameStatus::AwaitingDiscard { .. }
                | GameStatus::PlayingActions
                | GameStatus::AwaitingNewRobberLocation => 100.0,
                _ => 0.0,
            };

            // Pulsation uniquement quand on attend le lancer
            let die = if matches!(game.status(), GameStatus::AwaitingRoll) {
                ui.ctx().request_repaint(); // ← sans ça, rien ne bouge
                let t = ui.input(|i| i.time) as f32;
                base * (1.0 + 0.08 * (t * 3.0).sin())
            } else {
                base
            };

            let gap = 8.0;
            let (response, painter) =
                ui.allocate_painter(egui::vec2(die * 2.0 + gap, die), Sense::click());

            // Retour visuel au survol
            let face = if response.hovered() {
                Color32::from_rgb(255, 250, 235)
            } else {
                Color32::from_rgb(230, 228, 222)
            };

            let (a, b) = app
                .last_roll
                .map(|r| (r.dice1(), r.dice2()))
                .unwrap_or((1, 1));

            let c = response.rect.center();
            draw_die(
                &painter,
                c - egui::vec2((die + gap) / 2.0, 0.0),
                die,
                a,
                face,
            );
            draw_die(
                &painter,
                c + egui::vec2((die + gap) / 2.0, 0.0),
                die,
                b,
                face,
            );

            match game.status() {
                GameStatus::AwaitingRoll => {
                    if response.on_hover_text("Lancer les dés").clicked() {
                        actions.push(UiAction::Roll);
                    }
                }
                _ => {}
            }
        });

    actions
}