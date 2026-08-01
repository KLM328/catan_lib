use crate::{EdgeId, PlayerId};
use crate::{Roll, TileId, Topology, VertexId};
use std::fmt;

mod building;
mod production;
mod tile;

pub use crate::board::building::Building;
use crate::board::building::BuildingKind;
pub use crate::board::production::{Gain, Production};
pub use crate::board::tile::{NumberToken, Terrain, TerrainTokenMismatch, Tile};
use crate::game::GameStatus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidBoard {
    WrongTileCount { expected: usize, got: usize },
    WrongDistribution,
    NoDesert,
    TerrainToken(TerrainTokenMismatch),
    InvalidRobber,
}

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongTileCount { expected, got } => write!(
                f,
                "nombre de tuiles invalide : attendu {expected}, obtenu {got}"
            ),
            Self::WrongDistribution => write!(f, "la répartition ne correspond pas au scénario"),
            Self::NoDesert => write!(f, "aucun désert dans la répartition"),
            Self::TerrainToken(e) => write!(f, "{e}"),
            Self::InvalidRobber => write!(f, "Position du voleur invalide"),
        }
    }
}

impl From<TerrainTokenMismatch> for InvalidBoard {
    fn from(e: TerrainTokenMismatch) -> Self {
        Self::TerrainToken(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidAction {
    RobberOnDesert,
    UnexistingTile(TileId),
    UnexistingVertex(VertexId),
    UnexistingBuilding(VertexId),
    UnauthorizedAction,
    IsNotSettlement(VertexId),
    UnexistingEdge(EdgeId),
}

impl fmt::Display for InvalidAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidAction::RobberOnDesert => {
                write!(f, "Le voleur ne peut pas être déplacé sur un désert")
            }
            InvalidAction::UnexistingTile(tile_id) => {
                write!(f, "La tuile {} n'existe pas", tile_id.value())
            }
            InvalidAction::UnexistingVertex(vertex_id) => {
                write!(f, "Le vertex {} n'existe pas", vertex_id.value())
            }
            InvalidAction::IsNotSettlement(vertex_id) => {
                write!(
                    f,
                    "ce n'est pas une colonie sur l'emplacement {}",
                    vertex_id.value()
                )
            }
            InvalidAction::UnexistingBuilding(vertex_id) => {
                write!(
                    f,
                    "Aucune construction sur l'emplacement {}",
                    vertex_id.value()
                )
            }
            InvalidAction::UnauthorizedAction => {
                write!(f, "Vous n'avez pas la permission d'effectuer cette action")
            }
            InvalidAction::UnexistingEdge(edge_id) => {
                write!(f, "L'arête {} n'existe pas", edge_id.value())
            }
        }
    }
}
impl std::error::Error for InvalidAction {}

#[derive(Debug, PartialEq)]
pub struct Board {
    topology: Topology,
    tiles: Vec<Tile>,
    buildings: Vec<Option<Building>>,
    roads: Vec<Option<PlayerId>>,
    robber: TileId,
}
impl Board {
    pub fn new(
        topology: Topology,
        tiles: Vec<Tile>,
        robber: TileId,
    ) -> Result<Board, InvalidBoard> {
        if tiles.len() != topology.hexes().len() {
            Err(InvalidBoard::WrongTileCount {
                expected: topology.hexes().len(),
                got: tiles.len(),
            })
        } else if robber.value() >= tiles.len() || !matches!(tiles[robber.value()], Tile::Desert) {
            Err(InvalidBoard::InvalidRobber)
        } else {
            Ok(Board {
                tiles,
                buildings: vec![None; topology.vertex_count()],
                roads: vec![None; topology.edge_count()],
                robber,
                topology,
            })
        }
    }

    pub fn robber(&self) -> TileId {
        self.robber
    }

    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    pub fn topology(&self) -> &Topology {
        &self.topology
    }

    pub fn production(&self, roll: Roll) -> Production {
        let mut production = Production::default();

        for (index, tile) in self.tiles.iter().enumerate() {
            if index == self.robber.value() {
                continue;
            }
            if tile.number().map(|n| n.value()) != Some(roll.value()) {
                continue;
            }
            let Some(resource) = tile.resource() else {
                continue;
            };

            for vertex in &self.topology.tile_vertices()[index] {
                if let Some(building) = self.buildings[vertex.value()] {
                    production.add_gain(Gain {
                        player: building.owner(),
                        resource,
                        amount: building.kind().amount(),
                    });
                }
            }
        }
        production
    }

    pub fn can_place_building(
        &self,
        game_status: GameStatus,
        vertex: VertexId,
        player: PlayerId,
    ) -> bool {
        let mut vertexs = self.topology.vertex_neighbors(vertex);
        let connected_edges = self.topology.connected_edges(vertex);
        vertexs.push(vertex);
        if !vertexs.iter().all(|v| self.buildings[v.value()].is_none()) {
            false
        } else {
            matches!(game_status, GameStatus::Placement)
                || connected_edges
                    .iter()
                    .any(|&e_id| self.roads[e_id.value()] == Some(player))
        }
    }

    pub fn can_place_road(&self, game_status: GameStatus, edge: EdgeId, player: PlayerId) -> bool {
        let target_edge = self.roads.get(edge.value());
        if target_edge.is_none() {
            false
        } else {
            let target_edge = target_edge.unwrap();
            if target_edge.is_some() {
                false
            } else {
                let endpoints = self.topology.edges_endpoints()[edge.value()];
                let mut connected_edges = Vec::new();
                endpoints
                    .iter()
                    .for_each(|&v| connected_edges.extend(self.topology().connected_edges(v)));
                match game_status {
                    GameStatus::Placement => {
                        let player_buildings = endpoints
                            .iter()
                            .filter(|&&v| {
                                if let Some(building) = self.buildings[v.value()] {
                                    building.owner() == player
                                } else {
                                    false
                                }
                            })
                            .map(|&v| v)
                            .collect::<Vec<VertexId>>();

                        !player_buildings.is_empty()
                            && player_buildings.iter().any(|&v| {
                                self.topology
                                    .connected_edges(v)
                                    .iter()
                                    .all(|&e| self.roads[e.value()] != Some(player))
                            })
                    }
                    GameStatus::Playing => {
                        endpoints
                            .iter()
                            .any(|&v| matches!(self.buildings[v.value()], Some(player)))
                            || connected_edges
                                .iter()
                                .any(|&e| matches!(self.roads[e.value()], Some(player)))
                    }
                    GameStatus::End => false,
                }
            }
        }
    }

    pub fn place_road(
        &mut self,
        game_status: GameStatus,
        edge: EdgeId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if self.roads.get(edge.value()).is_none() {
            Err(InvalidAction::UnexistingEdge(edge))
        } else {
            if self.can_place_road(game_status, edge, player) {
                self.roads[edge.value()] = Some(player);
                Ok(())
            } else {
                Err(InvalidAction::UnauthorizedAction)
            }
        }
    }

    pub fn place_building(
        &mut self,
        game_status: GameStatus,
        vertex: VertexId,
        player: PlayerId,
        building_kind: BuildingKind,
    ) -> Result<(), InvalidAction> {
        if self.can_place_building(game_status, vertex, player) {
            self.buildings[vertex.value()] = Some(Building::new(building_kind, player));
            Ok(())
        } else {
            Err(InvalidAction::UnexistingVertex(vertex))
        }
    }

    pub fn move_robber(&mut self, tile_id: TileId) -> Result<(), InvalidAction> {
        let option_tile = self.tiles.get(tile_id.value());
        match option_tile {
            Some(tile) => {
                if matches!(tile, Tile::Desert) {
                    Err(InvalidAction::RobberOnDesert)
                } else {
                    self.robber = tile_id;
                    Ok(())
                }
            }
            None => Err(InvalidAction::UnexistingTile(tile_id)),
        }
    }

    pub fn upgrade_settlement(
        &mut self,
        vertex: VertexId,
        player: PlayerId,
    ) -> Result<(), InvalidAction> {
        if let Some(building) = self.buildings[vertex.value()] {
            if matches!(building.kind(), BuildingKind::Settlement) {
                if building.owner() == player {
                    self.buildings[vertex.value()] =
                        Some(Building::new(BuildingKind::City, player));
                    Ok(())
                } else {
                    Err(InvalidAction::UnauthorizedAction)
                }
            } else {
                Err(InvalidAction::IsNotSettlement(vertex))
            }
        } else {
            Err(InvalidAction::UnexistingBuilding(vertex))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::NumberToken;
    use crate::board::production::{Gain, Production};
    use crate::player::PlayerId;
    use crate::resource::Resource;

    pub fn init_board_without_buildings() -> Board {
        let topology = Topology::test_topology();
        Board::new(
            topology,
            vec![
                Tile::Forest(NumberToken::new(6).unwrap()),
                Tile::Hills(NumberToken::new(8).unwrap()),
                Tile::Desert,
            ],
            TileId::new(2),
        )
        .unwrap()
    }

    pub fn init_board() -> Board {
        let mut board = init_board_without_buildings();
        board
            .place_building(
                GameStatus::Placement,
                VertexId::new(4),
                PlayerId::new(1),
                BuildingKind::City,
            )
            .unwrap();
        board
            .place_building(
                GameStatus::Placement,
                VertexId::new(8),
                PlayerId::new(1),
                BuildingKind::Settlement,
            )
            .unwrap();
        board.move_robber(TileId::new(1)).unwrap();

        board
            .place_road(GameStatus::Placement, EdgeId::new(5), PlayerId::new(1))
            .unwrap();
        board
    }

    #[test]
    fn test_production() {
        let board = init_board();
        let mut expected_production = Production::default();
        expected_production.add_gain(Gain {
            player: PlayerId::new(1),
            resource: Resource::Wood,
            amount: 2,
        });
        assert_eq!(
            board.production(Roll::new(2, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_production_with_robber() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(
            board.production(Roll::new(4, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_production_with_no_buildings() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(
            board.production(Roll::new(4, 4).unwrap()),
            expected_production
        )
    }

    #[test]
    fn test_can_place_building_ok() {
        let board = init_board_without_buildings();
        assert!(board.can_place_building(
            GameStatus::Placement,
            VertexId::new(0),
            PlayerId::new(1)
        ));
    }

    #[test]
    fn test_can_place_building_ko_road() {
        let board = init_board_without_buildings();
        assert!(!board.can_place_building(GameStatus::Playing, VertexId::new(0), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_building_ko_neighbor() {
        let board = init_board();
        assert!(!board.can_place_building(
            GameStatus::Placement,
            VertexId::new(3),
            PlayerId::new(0)
        ));
    }

    #[test]
    fn test_upgrade_settlement_ok() {
        let mut board = init_board();
        board
            .upgrade_settlement(VertexId::new(8), PlayerId::new(1))
            .unwrap();
        assert_eq!(board.buildings[8].unwrap().kind(), BuildingKind::City);
    }

    #[test]
    fn test_upgrade_settlement_ko_wrong_player() {
        let mut board = init_board();
        assert!(
            board
                .upgrade_settlement(VertexId::new(8), PlayerId::new(0))
                .is_err()
        );
    }

    #[test]
    fn test_upgrade_settlement_ko_wrong_kind() {
        let mut board = init_board();
        assert!(
            board
                .upgrade_settlement(VertexId::new(4), PlayerId::new(1))
                .is_err()
        )
    }

    #[test]
    fn test_can_place_road_during_placement_ok() {
        let mut board = init_board();
        assert!(board.can_place_road(GameStatus::Placement, EdgeId::new(9), PlayerId::new(1)));
        assert!(board.can_place_road(GameStatus::Placement, EdgeId::new(10), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_road_during_placement_ko() {
        let mut board = init_board();
        assert!(!board.can_place_road(GameStatus::Placement, EdgeId::new(9), PlayerId::new(0)));
        assert!(!board.can_place_road(GameStatus::Placement, EdgeId::new(11), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_road_during_placement_ko_two_road_to_the_same_building() {
        let mut board = init_board();
        assert!(!board.can_place_road(GameStatus::Placement, EdgeId::new(4), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_road_during_placement_ko_second_road_consecutive_to_first_road() {
        let mut board = init_board();
        assert!(!board.can_place_road(GameStatus::Placement, EdgeId::new(0), PlayerId::new(1)));
        assert!(!board.can_place_road(GameStatus::Placement, EdgeId::new(4), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_road_during_playing_ko() {
        let mut board = init_board();
        assert!(!board.can_place_road(GameStatus::Playing, EdgeId::new(13), PlayerId::new(1)));
    }

    #[test]
    fn test_can_place_road_during_playing_ok() {
        let mut board = init_board();
        assert!(board.can_place_road(GameStatus::Playing, EdgeId::new(9), PlayerId::new(1)));
        assert!(board.can_place_road(GameStatus::Playing, EdgeId::new(4), PlayerId::new(1)));
        assert!(board.can_place_road(GameStatus::Playing, EdgeId::new(10), PlayerId::new(1)));
    }

    #[test]
    fn test_place_road_during_playing_ok() {
        let mut board = init_board();
        board
            .place_road(GameStatus::Playing, EdgeId::new(9), PlayerId::new(1))
            .unwrap();
        board
            .place_road(GameStatus::Playing, EdgeId::new(10), PlayerId::new(1))
            .unwrap();
        assert_eq!(board.roads[9].unwrap(), PlayerId::new(1));
        assert_eq!(board.roads[10].unwrap(), PlayerId::new(1));
    }

    #[test]
    fn test_place_road_during_playing_ko() {
        let mut board = init_board();
        assert!(
            board
                .place_road(GameStatus::Playing, EdgeId::new(3), PlayerId::new(1))
                .is_err()
        );
        assert!(
            board
                .place_road(GameStatus::Playing, EdgeId::new(7), PlayerId::new(1))
                .is_err()
        );
        assert_eq!(board.roads[3], None);
        assert_eq!(board.roads[7], None);
    }

    #[test]
    fn test_place_road_during_placement_ko() {
        let mut board = init_board();
        assert!(
            board
                .place_road(GameStatus::Placement, EdgeId::new(0), PlayerId::new(1))
                .is_err()
        );
        assert!(
            board
                .place_road(GameStatus::Placement, EdgeId::new(4), PlayerId::new(1))
                .is_err()
        );
        assert_eq!(board.roads[9], None);
        assert_eq!(board.roads[4], None);
    }

    #[test]
    fn test_place_road_during_placement_ok() {
        let mut board = init_board();
        assert!(
            board
                .place_road(GameStatus::Placement, EdgeId::new(10), PlayerId::new(1))
                .is_ok()
        );
        assert_eq!(board.roads[10].unwrap(), PlayerId::new(1));
    }
}
