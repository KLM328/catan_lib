use catan::{
    BuildingKind, EdgeId, Game, GameError, GameStatus, Layout, Player, PlayerColor, PlayerId, Roll,
    Scenario, Steal, Terrain, TileId, VertexId,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Shape, Stroke};

enum StealChoice { Pending, Nobody, Victim(PlayerId) }

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
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
}

impl CatanApp {
    fn new() -> Self {
        let scenario = Scenario::standard();
        let terrains = scenario.terrains().to_vec();

        let mut game = Game::new(
            scenario,
            vec![
                Player::new(PlayerColor::Blue),
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
        }
    }
}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.label(format!("Statut : {:?}", self.game.status()));
            ui.label(format!(
                "Joueur courant : {}",
                PlayerColor::color_name(
                    self.game
                        .get_player(self.game.current_player())
                        .unwrap()
                        .color()
                )
            ));

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
                                egui::vec2(self.hex_size * 0.3, self.hex_size * 0.3),
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
                                let _ = self
                                    .game
                                    .build_settlement(self.game.current_player(), vertex_location);
                            }
                        }
                        GameStatus::FirstPlacementRoad | GameStatus::SecondPlacementRoad => {
                            if let Some(edge_location) =
                                layout.pick_edge(topo, (pos.x, pos.y), radius)
                            {
                                let _ = self
                                    .game
                                    .build_road(self.game.current_player(), edge_location);
                            }
                        }
                        GameStatus::AwaitingRoll => {}
                        GameStatus::AwaitingDiscard { .. } => {}
                        GameStatus::AwaitingSteal => {}
                        GameStatus::AwaitingNewRobberLocation => {
                            let test = layout.pick_tile(topo, (pos.x, pos.y));
                            if let Some(tile_location) = test {
                                let _ = self
                                    .game
                                    .move_robber(self.game.current_player(), tile_location);
                            } else {
                                println!("test : {:?}", test);
                            }
                        }
                        GameStatus::PlayingActions => {}
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
                    StealChoice::Victim(victim) => Some(Steal::new(victim, self.game.get_player(victim).unwrap().hand().random_pick())),
                    StealChoice::Pending => return
                };
                if let Err(e) = self.game.steal(self.game.current_player(), steal) {
                    self.message = format!("{e:?}");
                }
            }
        });

        egui::Area::new(egui::Id::new("dices"))
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-24.0, -24.0))
            .show(ui.ctx(), |ui| {
                let base = match self.game.status() {
                    GameStatus::AwaitingRoll => 100.0,
                    GameStatus::AwaitingSteal
                    | GameStatus::AwaitingDiscard { .. }
                    | GameStatus::PlayingActions
                    | GameStatus::AwaitingNewRobberLocation => 60.0,
                    _ => 0.0,
                };

                // Pulsation uniquement quand on attend le lancer
                let die = if matches!(self.game.status(), GameStatus::AwaitingRoll) {
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

                let (a, b) = self
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

                match self.game.status() {
                    GameStatus::AwaitingRoll => {
                        if response.on_hover_text("Lancer les dés").clicked() {
                            let roll = Roll::random();
                            self.last_roll = Some(roll);
                            self.message = match self.game.apply_roll(roll) {
                                Ok(outcome) => format!("{outcome:?}"),
                                Err(e) => format!("{e:?}"),
                            };
                        }
                    }
                    _ => {}
                }
            });

        egui::Area::new(egui::Id::new("next_player"))
            .anchor(Align2::RIGHT_TOP, egui::vec2(-24.0, 24.0))
            .show(ui.ctx(), |ui| match self.game.status() {
                GameStatus::PlayingActions => {
                    let next_player = self.game.get_player(self.game.get_nex_player()).unwrap();
                    if hand_over_button(
                        ui,
                        player_color(next_player),
                        next_player.color().color_name(),
                    )
                    .clicked()
                    {
                        if let Err(e) = self.game.next_player() {
                            self.message = format!("{e:?}");
                        }
                    }
                }
                _ => {}
            });
    }
}

fn terrain_color(terrain: Terrain) -> Color32 {
    match terrain {
        Terrain::Desert => Color32::from_rgb(224, 201, 138),
        Terrain::Forest => Color32::from_rgb(45, 90, 39),
        Terrain::Mountain => Color32::from_rgb(122, 122, 140),
        Terrain::Hills => Color32::from_rgb(181, 90, 48),
        Terrain::Pasture => Color32::from_rgb(143, 193, 93),
        Terrain::Fields => Color32::from_rgb(232, 193, 74),
    }
}

fn player_color(player: &Player) -> Color32 {
    match player.color() {
        PlayerColor::Blue => Color32::from_rgb(0, 0, 250),
        PlayerColor::Red => Color32::from_rgb(185, 5, 20),
        PlayerColor::White => Color32::from_rgb(255, 255, 210),
        PlayerColor::Orange => Color32::from_rgb(255, 110, 0),
    }
}

fn draw_die(painter: &egui::Painter, center: Pos2, size: f32, value: u8, face: Color32) {
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

fn disc_button(ui: &mut egui::Ui, fill: Option<Color32>, name: &str, sub: &str) -> egui::Response {
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

fn hand_over_button(ui: &mut egui::Ui, next: Color32, name: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(180.0, 48.0), Sense::click());
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    let bg = Color32::from_gray(38 + (14.0 * t) as u8);
    painter.rect_filled(rect, rect.height() / 2.0, bg);

    painter.text(
        egui::pos2(rect.left() + 20.0, rect.center().y),
        Align2::LEFT_CENTER,
        format!("Au tour de {name}"),
        FontId::proportional(14.0),
        Color32::from_gray(220),
    );

    // Le disque glisse vers la droite au survol : le geste de passer la main.
    let c = egui::pos2(rect.right() - 26.0 + 4.0 * t, rect.center().y);
    painter.circle_filled(c, 14.0, next);
    painter.circle_stroke(c, 14.0, Stroke::new(2.0, Color32::from_rgb(38, 34, 30)));

    response
}
