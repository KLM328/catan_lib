use std::fmt;
use crate::{EdgeId, Roll, TileId, Topology, VertexId};
use crate::PlayerId;

mod tile;
mod building;
mod production;

pub use crate::board::tile::{Tile, NumberToken, Terrain};
pub use crate::board::building::Building;
pub use crate::board::production::{Gain, Production};



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidBoard(pub String);

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for InvalidBoard {}

pub struct Board {
    tiles: Vec<Tile>,
    tile_vertices: Vec<[VertexId; 6]>,
    tile_edges : Vec<[EdgeId; 6]>,
    buildings: Vec<Option<Building>>,
    roads: Vec<Option<PlayerId>>,
    robber : TileId,
}
impl Board {
    pub fn new(topology: &Topology, tiles :Vec<Tile>, robber : TileId) -> Result<Board, InvalidBoard> {
        if tiles.len() != topology.hexes().len() {
            Err(InvalidBoard("Le nombre de tuiles ne correspond pas au nombre de hexagones".to_string()))
        } else if robber.value() >= tiles.len() || !matches!(tiles[robber.value()], Tile::Desert) {
            Err(InvalidBoard("Position du voleur invalide".to_string()))
        } else {
            Ok(Board{
                tiles,
                tile_vertices: topology.tile_vertices().iter().copied().collect(),
                tile_edges: topology.tile_edges().iter().copied().collect(),
                buildings: vec![None; topology.vertex_count()],
                roads: vec![None; topology.edge_count()],
                robber,
            })
        }

    }

    pub fn production(&self, roll: Roll) -> Production {
        let mut production = Production::default();

        for (index, tile) in self.tiles.iter().enumerate() {
            if index == self.robber.value() { continue; }
            if tile.number().map(|n| n.value()) != Some(roll.value()) { continue; }
            let Some(resource) = tile.resource() else { continue; };

            for vertex in &self.tile_vertices[index] {
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
}

#[cfg(test)]
pub mod tests {
    use crate::player::PlayerId;
    use crate::board::production::{Gain, Production};
    use crate::NumberToken;
    use crate::resource::Resource;
    use super::*;

    pub fn init_board() -> Board {
        let topology = Topology::test_topology();
        let board = Board::new(&topology, vec![Tile::Forest(NumberToken::new(6).unwrap()), Tile::Hills(NumberToken::new(8).unwrap()), Tile::Desert], TileId::new(2)).unwrap();
        todo!();
        //placer une ville sur la tuile 0 pour le joueur 1
        //déplacer le voleur sur la tuile 2
        //palcer une colonie sur la tuile 1 pour le joueur 1
        board
    }

    #[test]
    fn test_production() {
        let board = init_board();
        let mut expected_production = Production::default();
        expected_production.add_gain(Gain { player: PlayerId::new(1), resource: Resource::Wood, amount: 2 });
        assert_eq!(board.production(Roll::new(2, 4).unwrap()), expected_production)
    }

    #[test]
    fn test_production_with_robber() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(board.production(Roll::new(4, 4).unwrap()), expected_production)
    }

    #[test]
    fn test_production_with_no_buildings() {
        let board = init_board();
        let expected_production = Production::default();
        assert_eq!(board.production(Roll::new(4, 4).unwrap()), expected_production)
    }
}