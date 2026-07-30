mod token;

pub use token::NumberToken;
use crate::resource::Resource;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tile {
    Desert,
    Forest(NumberToken),
    Mountain(NumberToken),
    Hills(NumberToken),
    Pasture(NumberToken),
    Fields(NumberToken),
}


impl Tile {
    pub fn resource(self) -> Option<Resource> {
        match self {
            Tile::Desert => None,
            Tile::Forest(_) => Some(Resource::Wood),
            Tile::Mountain(_) => Some(Resource::Stone),
            Tile::Hills(_) => Some(Resource::Brick),
            Tile::Pasture(_) => Some(Resource::Wool),
            Tile::Fields(_) => Some(Resource::Wheat),
        }
    }

    pub fn number(self) -> Option<NumberToken> {
        match self {
            Tile::Forest(n) => Some(n),
            Tile::Mountain(n) => Some(n),
            Tile::Hills(n) => Some(n),
            Tile::Pasture(n) => Some(n),
            Tile::Fields(n) => Some(n),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_terrain_maps_to_its_resource() {
        let t = NumberToken::new(2).unwrap();
        let cases = [
            (Tile::Desert, None),
            (Tile::Forest(t), Some(Resource::Wood)),
            (Tile::Mountain(t), Some(Resource::Stone)),
            (Tile::Hills(t), Some(Resource::Brick)),
            (Tile::Pasture(t), Some(Resource::Wool)),
            (Tile::Fields(t), Some(Resource::Wheat)),
        ];
        for (terrain, expected) in cases {
            assert_eq!(terrain.resource(), expected, "fail for {terrain:?}");
        }
    }

    #[test]
    fn test_number_token() {
        assert_eq!(Tile::Fields(NumberToken::new(8).unwrap()).number(), NumberToken::new(8).ok());
        assert_eq!(Tile::Desert.number(), None);

    }
}