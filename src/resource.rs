mod cost;
mod hand;
mod counts;

pub use cost::Cost;
pub use hand::Hand;
pub use counts::ResourceCounts; 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Wood,
    Stone,
    Brick,
    Wheat,
    Wool
}

impl Resource {
    pub(crate) fn index(self) -> usize {
        match self {
            Resource::Wood => 0,
            Resource::Stone => 1,
            Resource::Brick => 2,
            Resource::Wheat => 3,
            Resource::Wool => 4
        }
    }
}