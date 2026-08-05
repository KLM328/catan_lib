use catan::{Game, Layout, Player, PlayerColor, Roll, Scenario, Terrain, TileId};
use eframe::egui::{self, Color32, FontId, Pos2, Sense, Shape, Stroke, Align2};

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

        game.set_players_order(vec![Roll::random(), Roll::random(), Roll::random()])
            .expect("ordre des joueurs");
        game.start(&terrains).expect("mise en place du plateau");

        Self { game, hex_size: 80.0 }
    }
}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Catan");
            ui.label(format!("Statut : {:?}", self.game.status()));
            ui.label(format!("Joueur courant : {:?}", self.game.current_player()));
            ui.separator();

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
                    painter.circle_filled(pos, self.hex_size*0.35, Color32::from_rgb(240, 235, 220));

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
                        FontId::proportional(self.hex_size*size),
                        color,
                    );
                }

                // Le voleur.
                if board.robber() == tile_id {
                    let (cx, cy) = layout.tile_position(topo, tile_id);
                    painter.circle_filled(
                        Pos2::new(cx, cy - self.hex_size*0.50),
                        self.hex_size*0.30,
                        Color32::from_rgb(20, 20, 20),
                    );
                }
            }
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