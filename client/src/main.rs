mod app;
mod panels;
mod theme;
mod widgets;

pub use app::UiAction;
use panels::{dice, hand, infos, next_player, end};
pub(crate) use theme::{player_color, resource_color, terrain_color};
pub(crate) use widgets::{action_button, disc_button, draw_die, hand_over_button, player_row};

use catan::{
    BuildingKind, EdgeId, Game, GameError, GameStatus, Layout, Player, PlayerColor, PlayerId, ResourceCounts, Roll, Scenario, Steal, TileId, VertexId,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Shape, Stroke};
use crate::panels::actions;

#[derive(PartialEq, Clone, Copy)]
enum StealChoice {
    Pending,
    Nobody,
    Victim(PlayerId),
}

#[derive(PartialEq, Clone, Copy)]
enum BuildMode {
    None,
    Road,
    Settlement,
    City,
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };

    eframe::run_native(
        "Catan",
        options,
        Box::new(|_cc| Ok(Box::new(CatanApp::new()))),
    )
}

struct CatanApp {
    game: Game,
    hex_size: f32,
    last_roll: Option<Roll>,
    message: String,
    build_mode: BuildMode,
    discard_selection: ResourceCounts,
}

impl CatanApp {
    fn new() -> Self {
        let scenario = Scenario::standard();
        let terrains = scenario.terrains().to_vec();

        let mut game = Game::new(
            scenario,
            vec![
                Player::new(PlayerColor::Orange),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::White),
            ],
        )
        .expect("création de la partie");

        while let Err(GameError::TiedRolls) =
            game.set_players_order(vec![Roll::random(), Roll::random(), Roll::random()])
        {}

        game.start(&terrains).expect("mise en place du plateau");

        Self {
            game,
            hex_size: 80.0,
            last_roll: None,
            message: String::new(),
            build_mode: BuildMode::None,
            discard_selection: ResourceCounts::default(),
        }
    }

    fn apply(&mut self, action: UiAction) {
        let player = self.game.current_player();
        let result = match action {
            UiAction::Roll => {
                let roll = Roll::random();
                self.last_roll = Some(roll);
                self.game.apply_roll(roll).map(|outcome| {
                    self.message = format!("{outcome:?}");
                })
            }
            UiAction::NextPlayer => self.game.next_player(),
            UiAction::BuildSettlement(vertex_id) => {
                let result = self.game.build_settlement(player, vertex_id);
                if let Ok(()) = result {
                    self.build_mode = BuildMode::None;
                }
                result
            }
            UiAction::BuildRoad(edge_id) => {

                let result = self.game.build_road(player, edge_id);
                if let Ok(()) = result {
                    self.build_mode = BuildMode::None;
                }
                result
            }
            UiAction::UpgradeCity(vertex_id) => {

                let result = self.game.upgrade_settlement_to_city(player, vertex_id);
                if let Ok(()) = result {
                    self.build_mode = BuildMode::None;
                }
                result
            }
            UiAction::MoveRobber(tile_id) => self.game.move_robber(player, tile_id),
            UiAction::Steal(steal_option) => self.game.steal(player, steal_option),
            UiAction::Discard(player, resources) => {
                self.discard_selection = ResourceCounts::default();
                self.game.discard(player, resources)
            }
        };
        if let Err(e) = result {
            self.message = format!("{e:?}");
        }
    }
}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut actions = Vec::new();
        let ctx = ui.ctx().clone();

        // Taille physique de la fenêtre, indépendante du zoom courant :
        // screen_rect rétrécit quand pixels_per_point augmente, le produit est stable.
        let physical_h = ctx.content_rect().height() * ctx.pixels_per_point();
        let native_ppp = ctx.native_pixels_per_point().unwrap_or(1.0);
        let target = (physical_h / native_ppp / 1080.0).clamp(0.5, 2.0);

        if (ctx.zoom_factor() - target).abs() > 0.01 {
            ctx.set_zoom_factor(target);
        }

        if ui.input(|i| i.key_pressed(egui::Key::F11)) {
            let full = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!full));
        }

        infos::show(ui, &self.game);

        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(format!("Statut : {:?}", self.game.status()));

            // On réserve toute la place restante et on récupère un pinceau.
            let available = ui.available_size();
            let (response, painter) = ui.allocate_painter(available, Sense::click());

            // Le plateau est centré sur (0,0) en coordonnées hexagonales :
            // il suffit de placer l'origine au centre de la zone de dessin.
            let center = response.rect.center();
            let layout = Layout {
                hex_size: self.hex_size,
                origin: (center.x, center.y),
            };

            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.hex_size = (self.hex_size * (1.0 + scroll * 0.002)).clamp(20.0, 200.0);
            }

            let board = self.game.board().expect("plateau initialisé");
            let topo = board.topology();

            for (index, tile) in board.tiles().iter().enumerate() {
                let tile_id = TileId::new(index);

                // Les 6 coins de la tuile sont les positions de ses sommets :
                // aucune fonction de coins supplémentaire n'est nécessaire.
                let corners: Vec<Pos2> = topo.tile_vertices()[index]
                    .iter()
                    .map(|&v| {
                        let (x, y) = layout.vertex_position(topo, v);
                        Pos2::new(x, y)
                    })
                    .collect();

                painter.add(Shape::convex_polygon(
                    corners,
                    terrain_color(tile.terrain()),
                    Stroke::new(2.0, Color32::from_rgb(40, 40, 50)),
                ));

                // Le jeton numéroté, au centre de la tuile.
                if let Some(token) = tile.number() {
                    let (cx, cy) = layout.tile_position(topo, tile_id);
                    let pos = Pos2::new(cx, cy);
                    painter.circle_filled(
                        pos,
                        self.hex_size * 0.35,
                        Color32::from_rgb(240, 235, 220),
                    );

                    // 6 et 8 sont les numéros "rouges" : les plus probables.
                    let color = match token.value() {
                        6 | 8 => Color32::from_rgb(180, 30, 30),
                        _ => Color32::from_rgb(30, 30, 30),
                    };

                    let size = match token.value() {
                        6 | 8 => 0.4,
                        5 | 9 => 0.36,
                        4 | 10 => 0.32,
                        3 | 11 => 0.28,
                        _ => 0.24,
                    };

                    painter.text(
                        pos,
                        Align2::CENTER_CENTER,
                        token.value().to_string(),
                        FontId::proportional(self.hex_size * size),
                        color,
                    );
                }

                // Le voleur.
                if board.robber() == tile_id {
                    let (cx, cy) = layout.tile_position(topo, tile_id);
                    painter.circle_filled(
                        Pos2::new(cx, cy - self.hex_size * 0.50),
                        self.hex_size * 0.30,
                        Color32::from_rgb(20, 20, 20),
                    );
                }
            }

            // Les routes d'abord (elles passent sous les bâtiments)
            for (index, owner) in board.roads().iter().enumerate() {
                if let Some(player) = owner {
                    let ((ax, ay), (bx, by)) = layout.edge_position(topo, EdgeId::new(index));
                    painter.line_segment(
                        [Pos2::new(ax, ay), Pos2::new(bx, by)],
                        Stroke::new(
                            self.hex_size * 0.15,
                            player_color(self.game.get_player(*player).unwrap()),
                        ),
                    );
                }
            }

            // Puis les colonies et les villes
            for (index, building) in board.buildings().iter().enumerate() {
                if let Some(b) = building {
                    let (x, y) = layout.vertex_position(topo, VertexId::new(index));
                    let pos = Pos2::new(x, y);
                    let color = player_color(self.game.get_player(b.owner()).unwrap());
                    match b.kind() {
                        BuildingKind::Settlement => {
                            painter.circle_filled(pos, self.hex_size * 0.3, color)
                        }
                        BuildingKind::City => painter.rect_filled(
                            egui::Rect::from_center_size(
                                pos,
                                egui::vec2(self.hex_size * 0.6, self.hex_size * 0.6),
                            ),
                            2.0,
                            color,
                        ),
                    };
                }
            }

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let radius = self.hex_size * 0.30;
                    match self.game.status() {
                        GameStatus::Starting => {}
                        GameStatus::FirstPlacementSettlement
                        | GameStatus::SecondPlacementSettlement => {
                            if let Some(vertex_location) =
                                layout.pick_vertex(topo, (pos.x, pos.y), radius)
                            {
                                actions.push(UiAction::BuildSettlement(vertex_location));
                            }
                        }
                        GameStatus::FirstPlacementRoad | GameStatus::SecondPlacementRoad => {
                            if let Some(edge_location) =
                                layout.pick_edge(topo, (pos.x, pos.y), radius)
                            {
                                actions.push(UiAction::BuildRoad(edge_location));
                            }
                        }
                        GameStatus::AwaitingRoll => {}
                        GameStatus::AwaitingDiscard { .. } => {}
                        GameStatus::AwaitingSteal => {}
                        GameStatus::AwaitingNewRobberLocation => {
                            if let Some(tile_location) = layout.pick_tile(topo, (pos.x, pos.y)) {
                                actions.push(UiAction::MoveRobber(tile_location));
                            }
                        }
                        GameStatus::PlayingActions => match self.build_mode {
                            BuildMode::None => {}
                            BuildMode::Road => {
                                if let Some(edge_location) =
                                    layout.pick_edge(topo, (pos.x, pos.y), radius)
                                {
                                    actions.push(UiAction::BuildRoad(edge_location));
                                }
                            }
                            BuildMode::Settlement => {
                                if let Some(vertex_location) =
                                    layout.pick_vertex(topo, (pos.x, pos.y), radius)
                                {
                                    actions.push(UiAction::BuildSettlement(vertex_location));
                                }
                            }
                            BuildMode::City => {
                                if let Some(vertex_location) =
                                    layout.pick_vertex(topo, (pos.x, pos.y), radius)
                                {
                                    actions.push(UiAction::UpgradeCity(vertex_location))
                                }
                            }
                        },
                        GameStatus::End { .. } => {}
                    }
                }
            }

            if matches!(self.game.status(), GameStatus::AwaitingSteal) {
                // 1. On extrait les données AVANT le closure
                let mut chosen: StealChoice = StealChoice::Pending;

                if let Ok(victims) = self.game.steal_victims(self.game.current_player()) {
                    const DISC_W: f32 = 78.0;
                    const GAP: f32 = 10.0;

                    let n = victims.len() as f32;
                    let total = n.max(1.0) * DISC_W + (n - 1.0).max(0.0) * GAP;

                    egui::Modal::new(egui::Id::new("steal")).show(ui.ctx(), |ui| {
                        ui.set_width((total + 48.0).max(240.0));
                        ui.vertical_centered(|ui| {
                            ui.heading("À qui volez-vous une carte ?");
                            ui.add_space(14.0);

                            if victims.is_empty() {
                                if disc_button(ui, None, "Personne", "à voler").clicked() {
                                    chosen = StealChoice::Nobody;
                                }
                            } else {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = GAP;
                                    ui.add_space(((ui.available_width() - total) * 0.5).max(0.0));
                                    for &v in &victims {
                                        let p = &self.game.get_player(v).unwrap();
                                        let cards = p.hand().count();
                                        let sub = if cards > 1 {
                                            format!("{cards} cartes")
                                        } else {
                                            format!("{cards} carte")
                                        };
                                        if disc_button(
                                            ui,
                                            Some(player_color(p)),
                                            PlayerColor::color_name(p.color()),
                                            sub.as_str(),
                                        )
                                        .clicked()
                                        {
                                            chosen = StealChoice::Victim(v);
                                        }
                                    }
                                });
                            }
                        });
                    });
                }

                // 2. On applique APRÈS, hors du closure
                let steal = match chosen {
                    StealChoice::Nobody => None,
                    StealChoice::Victim(victim) => Some(Steal::new(
                        victim,
                        self.game.get_player(victim).unwrap().hand().random_pick(),
                    )),
                    StealChoice::Pending => return,
                };
                actions.push(UiAction::Steal(steal));
            }
        });

        actions.extend(dice::show(ui, &self.game, self));
        actions.extend(next_player::show(ui, &self.game));
        actions.extend(hand::show(ui, &self.game, &mut self.discard_selection));

        actions::show(ui, &self.game, &mut self.build_mode);

        end::show(ui, &self.game);

        for action in actions {
            self.apply(action);
        }
    }
}

fn cell(ui: &mut egui::Ui, text: egui::RichText) {
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        |ui| {
            ui.label(text);
        },
    );
}
