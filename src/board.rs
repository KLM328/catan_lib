use crate::Roll;


mod tile;
mod building;
mod production;

pub use crate::board::tile::{Tile, NumberToken};
pub use crate::board::building::Building;
pub use crate::board::production::{Gain, Production};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId(u8);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(u8);

pub struct Board {
    tiles: Vec<Tile>,
    tile_vertices: Vec<[VertexId; 6]>,
    buildings: Vec<Option<Building>>,
    robber: TileId,
}
impl Board {
    pub fn production(&self, roll: Roll) -> Production {
        let mut production = Production::default();

        for (index, tile) in self.tiles.iter().enumerate() {
            if TileId(index as u8) == self.robber { continue; }
            if tile.number().map(|n| n.value()) != Some(roll.value()) { continue; }
            let Some(resource) = tile.resource() else { continue; };

            for vertex in &self.tile_vertices[index] {
                if let Some(building) = self.buildings[vertex.0 as usize] {
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
mod tests {
    use crate::board::building::BuildingKind;
    use crate::player::PlayerId;
    use crate::board::production::{Gain, Production};
    use crate::NumberToken;
    use crate::resource::Resource;
    use super::*;

    fn init_board() -> Board {
        Board {
            tiles: vec![Tile::Forest(NumberToken::new(6).unwrap()) , Tile::Hills(NumberToken::new(8).unwrap())],
            tile_vertices: vec![[VertexId(0), VertexId(1), VertexId(2), VertexId(3), VertexId(4), VertexId(5)], [VertexId(2), VertexId(3), VertexId(6), VertexId(7), VertexId(8), VertexId(9)]],
            buildings: vec![Some(Building::new(BuildingKind::City, PlayerId::new(1))), None, None, None, None, None, None, None, Some(Building::new(BuildingKind::Settlement, PlayerId::new(1))), None],
            robber: TileId(1),
        }
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