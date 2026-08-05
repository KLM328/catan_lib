mod cost;
mod hand;
mod counts;
mod steal;

pub use cost::Cost;
pub use hand::{Hand, ResourceError};
pub use counts::ResourceCounts;
pub use steal::Steal;

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
    
    pub(crate) fn from_index(index: usize) -> Option<Resource> {
        match index {
            0 => Some(Resource::Wood),
            1 => Some(Resource::Stone),
            2 => Some(Resource::Brick),
            3 => Some(Resource::Wheat),
            4 => Some(Resource::Wool),
            _ => None
        }
    }
}