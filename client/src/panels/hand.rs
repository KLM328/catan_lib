use eframe::egui;
use eframe::egui::{Align2, Color32, FontId, Sense, Stroke, Ui};
use catan::{Game, GameStatus, PlayerColor, PlayerId, Resource, ResourceCounts};
use crate::theme::{player_color, resource_color, CARD_H, CARD_W, GAP};
use crate::{UiAction};

pub(crate) fn show(ui: &mut Ui, game: &Game, selection: &mut ResourceCounts) -> Vec<UiAction>{

    let mut actions = Vec::new();

    egui::Area::new(egui::Id::new("hand"))
        .anchor(Align2::LEFT_BOTTOM, egui::vec2(24.0, -24.0))
        .show(ui.ctx(), |ui| {
            let (player_id, required) = match game.status() {
                GameStatus::AwaitingDiscard { must_discard } => {
                    match must_discard.iter().position(|&n| n > 0) {
                        Some(i) => (PlayerId::new(i), Some(must_discard[i])),
                        None => return,
                    }
                }
                _ => (game.current_player(), None),
            };

            let player = game.get_player(player_id).unwrap();
            let hand = player.hand();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(required) = required {
                        if selection.count() == required && ui.button("Défausser").clicked() {
                            actions.push(UiAction::Discard(player_id, *selection));
                        }
                    }
                    let c = player_color(player);
                    let label = match required {
                        Some(n) => format!("{} doit défausser {} / {n}",
                                           PlayerColor::color_name(player.color()),
                                           selection.count()),
                        None => format!("Main de {}", PlayerColor::color_name(player.color())),
                    };
                    ui.label(egui::RichText::new(label).size(20.0).color(c));
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    for &resource in Resource::ALL.iter() {
                        let (card, response) =
                            ui.allocate_exact_size(egui::vec2(CARD_W, CARD_H), Sense::click());
                        let painter = ui.painter_at(card);
                        let count = hand.amount(resource);
                        let card = egui::Rect::from_min_size(
                            egui::pos2(card.left(), card.top()),
                            egui::vec2(CARD_W, CARD_H),
                        );
                        let color = resource_color(resource);

                        if count == 0 {
                            painter.rect_stroke(
                                card,
                                5.0,
                                Stroke::new(2.0, color.gamma_multiply(0.4)),
                                egui::StrokeKind::Inside,
                            );
                        } else {
                            painter.rect_filled(card, 5.0, color);
                            painter.rect_stroke(
                                card,
                                5.0,
                                Stroke::new(2.0, Color32::from_rgb(38, 34, 30)),
                                egui::StrokeKind::Inside,
                            );

                            let badge = egui::pos2(card.center().x, card.bottom() - 22.0);
                            painter.circle_filled(badge, 16.0, Color32::from_rgb(30, 28, 25));
                            painter.text(
                                badge,
                                Align2::CENTER_CENTER,
                                count.to_string(),
                                FontId::proportional(18.0),
                                Color32::from_gray(235),
                            );

                            let selected =
                                required.map_or(0, |_| selection.amount(resource));
                            if selected > 0 {
                                // La part défaussée s'efface : on voit ce qu'on va perdre.
                                let lost = card.height() * selected as f32 / count.max(1) as f32;
                                painter.rect_filled(
                                    egui::Rect::from_min_size(card.left_top(), egui::vec2(CARD_W, lost)),
                                    5.0,
                                    Color32::from_black_alpha(170),
                                );
                                let mark = egui::pos2(card.center().x, card.top() + 22.0);
                                painter.circle_filled(mark, 15.0, Color32::from_rgb(150, 40, 40));
                                painter.text(
                                    mark,
                                    Align2::CENTER_CENTER,
                                    format!("-{selected}"),
                                    FontId::proportional(16.0),
                                    Color32::from_gray(240),
                                );
                            }
                        }

                        if let Some(required) = required {
                            let selected = selection.amount(resource);
                            if response.clicked() && selected < count && selection.count() < required {
                                selection.add(&ResourceCounts::from_resource(resource, 1));
                            }
                            if response.secondary_clicked() && selected > 0 {
                                selection.remove(&ResourceCounts::from_resource(resource, 1));
                            }

                        }
                    }
                });
            });


        });

    actions
}