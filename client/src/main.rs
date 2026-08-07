use catan::{
    BuildingKind, Cost, EdgeId, Game, GameError, GameStatus, Layout, Player, PlayerColor, PlayerId,
    Resource, ResourceCounts, Roll, Scenario, Steal, Terrain, TileId, VertexId,
};
use eframe::egui::{self, Align2, Color32, FontId, Pos2, Sense, Shape, Stroke};

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
        viewport: egui::ViewportBuilder::default()
            .with_fullscreen(true),
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
}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Taille physique de la fenêtre, indépendante du zoom courant :
        // screen_rect rétrécit quand pixels_per_point augmente, le produit est stable.
        let physical_h = ctx.content_rect().height() * ctx.pixels_per_point();
        let native_ppp = ctx.native_pixels_per_point().unwrap_or(1.0);
        let target = (physical_h / native_ppp / 1080.0).clamp(0.5, 2.0);

        if (ctx.zoom_factor() - target).abs() > 0.01 {
            ctx.set_zoom_factor(target);
        }


        //discard
        let mut validate = false;
        let mut discarding: Option<PlayerId> = None;

        if ui.input(|i| i.key_pressed(egui::Key::F11)) {
            let full = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Fullscreen(!full));
        }


        egui::Panel::right("info")
            .exact_size(300.0)
            .show(ui, |ui| {
                ui.add_space(12.0);

                ui.heading("Joueurs");
                ui.add_space(8.0);

                let current = self.game.current_player();
                self.game.turn_order().iter().map(|&id| (id, self.game.get_player(id).unwrap())).for_each(|(id, player)| {
                    player_row(ui, player, id == current);
                    ui.add_space(6.0);
                });
            });

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
                        GameStatus::AwaitingDiscard { .. } => {

                        }
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
                        GameStatus::PlayingActions => match self.build_mode {
                            BuildMode::None => {}
                            BuildMode::Road => {
                                if let Some(edge_location) =
                                    layout.pick_edge(topo, (pos.x, pos.y), radius)
                                {
                                    let _ = self
                                        .game
                                        .build_road(self.game.current_player(), edge_location);
                                    self.build_mode = BuildMode::None;
                                }
                            }
                            BuildMode::Settlement => {
                                if let Some(vertex_location) =
                                    layout.pick_vertex(topo, (pos.x, pos.y), radius)
                                {
                                    let _ = self.game.build_settlement(
                                        self.game.current_player(),
                                        vertex_location,
                                    );
                                    self.build_mode = BuildMode::None;
                                }
                            }
                            BuildMode::City => {
                                if let Some(vertex_location) =
                                    layout.pick_vertex(topo, (pos.x, pos.y), radius)
                                {
                                    let _ = self.game.upgrade_settlement_to_city(
                                        self.game.current_player(),
                                        vertex_location,
                                    );
                                    self.build_mode = BuildMode::None;
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
                if let Err(e) = self.game.steal(self.game.current_player(), steal) {
                    self.message = format!("{e:?}");
                }
            }
        });

        egui::Area::new(egui::Id::new("dices"))
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-324.0, -24.0))
            .show(ui.ctx(), |ui| {
                let base = match self.game.status() {
                    GameStatus::AwaitingRoll => 120.0,
                    GameStatus::AwaitingSteal
                    | GameStatus::AwaitingDiscard { .. }
                    | GameStatus::PlayingActions
                    | GameStatus::AwaitingNewRobberLocation => 100.0,
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
            .anchor(Align2::RIGHT_TOP, egui::vec2(-324.0, 24.0))
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
                        } else {
                            self.build_mode = BuildMode::None;
                        }
                    }
                }
                GameStatus::AwaitingDiscard { must_discard} => {
                    let required = must_discard.iter().find(|&&n| n > 0).copied().unwrap_or(0);
                    if self.discard_selection.count() == required
                        && ui.button("Défausser").clicked()
                    {
                        validate = true;
                    }
                }
                _ => {}
            });

        egui::Area::new(egui::Id::new("hand"))
            .anchor(Align2::LEFT_BOTTOM, egui::vec2(24.0, -24.0))
            .show(ui.ctx(), |ui| {
                let (player_id, required) = match self.game.status() {
                    GameStatus::AwaitingDiscard { must_discard } => {
                        match must_discard.iter().position(|&n| n > 0) {
                            Some(i) => (PlayerId::new(i), Some(must_discard[i])),
                            None => return,
                        }
                    }
                    _ => (self.game.current_player(), None),
                };
                if required.is_some() {
                    discarding = Some(player_id);
                }

                let player = self.game.get_player(player_id).unwrap();
                let hand = player.hand();

                const W: f32 = 100.0;
                const H: f32 = 155.0;
                const GAP: f32 = 10.0;

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let c = player_color(player);
                        let label = match required {
                            Some(n) => format!("{} doit défausser {} / {n}",
                                               PlayerColor::color_name(player.color()),
                                               self.discard_selection.count()),
                            None => format!("Main de {}", PlayerColor::color_name(player.color())),
                        };
                        ui.label(egui::RichText::new(label).size(20.0).color(c));
                    });
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = GAP;
                        for &resource in Resource::ALL.iter() {
                            let (card, response) =
                                ui.allocate_exact_size(egui::vec2(W, H), Sense::click());
                            let painter = ui.painter_at(card);
                            let count = hand.amount(resource);
                            let card = egui::Rect::from_min_size(
                                egui::pos2(card.left(), card.top()),
                                egui::vec2(W, H),
                            );
                            let color = resource_color(resource);

                            if count == 0 {
                                // Emplacement vide : le contour garde la teinte de la ressource,
                                // pour qu'on sache de quelle carte il s'agit.
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
                                    required.map_or(0, |_| self.discard_selection.amount(resource));
                                if selected > 0 {
                                    // La part défaussée s'efface : on voit ce qu'on va perdre.
                                    let lost = card.height() * selected as f32 / count.max(1) as f32;
                                    painter.rect_filled(
                                        egui::Rect::from_min_size(card.left_top(), egui::vec2(W, lost)),
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
                                let selected = self.discard_selection.amount(resource);
                                if response.clicked() && selected < count && self.discard_selection.count() < required {
                                    self.discard_selection.add(&ResourceCounts::from_resource(resource, 1));
                                }
                                if response.secondary_clicked() && selected > 0 {
                                    self.discard_selection.remove(&ResourceCounts::from_resource(resource, 1));
                                }
                            }
                        }
                    });
                });


            });

        egui::Area::new(egui::Id::new("actions"))
            .anchor(Align2::RIGHT_BOTTOM, egui::vec2(-542.0, -24.0))
            .show(ui.ctx(), |ui| match self.game.status() {
                GameStatus::PlayingActions => {
                    let player = self.game.get_player(self.game.current_player()).unwrap();
                    let color = player_color(player);

                    ui.horizontal(|ui| {
                        // ui.spacing_mut().item_spacing.x = 10.0;
                        for (mode, cost) in [
                            (BuildMode::Road, &Cost::ROAD),
                            (BuildMode::Settlement, &Cost::SETTLEMENT),
                            (BuildMode::City, &Cost::CITY),
                        ] {
                            let ok = player.can_pay(cost).is_ok();
                            if action_button(ui, mode, self.build_mode, cost, ok, color).clicked()
                                && ok
                            {
                                self.build_mode = if self.build_mode == mode {
                                    BuildMode::None
                                } else {
                                    mode
                                };
                            }
                        }
                    });
                }

                _ => {}
            });

        if validate {
            if let Some(player_id) = discarding {
                if let Err(e) = self.game.discard(player_id, self.discard_selection) {
                    self.message = format!("{e:?}");
                }
                self.discard_selection = ResourceCounts::default();
            }
        }
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

fn resource_color(r: Resource) -> Color32 {
    match r {
        Resource::Wood => terrain_color(Terrain::Forest),
        Resource::Brick => terrain_color(Terrain::Hills),
        Resource::Stone => terrain_color(Terrain::Mountain),
        Resource::Wheat => terrain_color(Terrain::Fields),
        Resource::Wool => terrain_color(Terrain::Pasture),
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
    const PAD: f32 = 10.0; // marge gauche et droite
    const GAP: f32 = 12.0; // entre le texte et le disque
    const RADIUS: f32 = 18.0;
    const SLIDE: f32 = 10.0; // course du disque au survol
    let text_color = Color32::from_gray(220);

    // 1. Mesurer
    let galley = ui.painter().layout_no_wrap(
        format!("Au tour de {name}"),
        FontId::proportional(25.0),
        text_color,
    );

    // 2. Allouer d'après la mesure
    let width = PAD * 2.0 + galley.size().x + GAP + RADIUS * 2.0 + SLIDE + PAD;
    let height = (galley.size().y + 24.0).max(RADIUS * 2.0 + 16.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());

    // 3. Dessiner
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    painter.rect_filled(
        rect,
        height / 2.0,
        Color32::from_gray(50 + (20.0 * t) as u8),
    );

    let text_pos = egui::pos2(
        rect.left() + PAD * 2.0,
        rect.center().y - galley.size().y / 2.0,
    );
    painter.galley(text_pos, galley, text_color);

    let c = egui::pos2(
        rect.right() - PAD - RADIUS - SLIDE + SLIDE * t,
        rect.center().y,
    );
    painter.circle_filled(c, RADIUS, next);
    painter.circle_stroke(c, RADIUS, Stroke::new(2.0, Color32::from_rgb(38, 34, 30)));

    response
}

fn action_button(
    ui: &mut egui::Ui,
    mode: BuildMode,
    current: BuildMode,
    cost: &Cost,
    affordable: bool,
    color: Color32,
) -> egui::Response {
    const S: f32 = 100.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(S, S + 18.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool(response.id, response.hovered() && affordable);
    let painter = ui.painter_at(rect);

    let selected = mode == current;
    let bg = if selected {
        Color32::from_gray(80)
    } else {
        Color32::from_gray(38 + (14.0 * t) as u8)
    };
    let body =
        egui::Rect::from_min_size(egui::pos2(rect.left(), rect.bottom() - S), egui::vec2(S, S));
    painter.rect_filled(body, 8.0, bg);
    if selected {
        painter.rect_stroke(body, 8.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    }

    // La pièce elle-même, dans la couleur du joueur — la même forme que sur le plateau.
    let c = body.center();
    let piece = if affordable {
        color
    } else {
        color.gamma_multiply(0.35)
    };
    match mode {
        BuildMode::Road => {
            painter.line_segment(
                [c + egui::vec2(-16.0, 8.0), c + egui::vec2(16.0, -8.0)],
                Stroke::new(7.0, piece),
            );
            ()
        }
        BuildMode::Settlement => {
            painter.circle_filled(c, 12.0, piece);
            ()
        }
        BuildMode::City => {
            painter.rect_filled(
                egui::Rect::from_center_size(c, egui::vec2(24.0, 24.0)),
                3.0,
                piece,
            );
            ()
        }
        BuildMode::None => (),
    }

    // Le coût, en pastilles de la couleur des terrains.
    let dots: Vec<Resource> = Resource::ALL
        .iter()
        .flat_map(|&r| std::iter::repeat(r).take(cost.amount(r) as usize))
        .collect();
    let dw = 11.0;
    let start = rect.center().x - (dots.len() as f32 - 1.0) * dw * 0.5;
    for (i, &r) in dots.iter().enumerate() {
        let p = egui::pos2(start + i as f32 * dw, rect.top() + 8.0);
        let col = if affordable {
            resource_color(r)
        } else {
            resource_color(r).gamma_multiply(0.4)
        };
        painter.circle_filled(p, 4.5, col);
    }

    response
}

fn player_row(ui: &mut egui::Ui, player: &Player, is_current: bool) {
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