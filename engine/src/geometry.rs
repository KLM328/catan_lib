mod topology;
mod hex;
mod ids;

pub use topology::Topology;

pub use hex::{Hex, HexCorner};

pub use ids::{EdgeId, VertexId, TileId, ConnectedEdges};

use std::fmt;

const DIRS: [(i8, i8); 6] = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDirection(pub usize);

impl fmt::Display for InvalidDirection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} n'est pas une direction valide (0-5)", self.0)
    }
}
impl std::error::Error for InvalidDirection {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HexDirection(usize);

impl HexDirection {
    pub(crate) fn new(dir : usize) -> Result<HexDirection, InvalidDirection> {
        if matches!(dir, 0..=5) {
            Ok(HexDirection(dir))
        }else{
            Err(InvalidDirection(dir))
        }
    }

    pub const ALL : [HexDirection; 6] = [HexDirection(0), HexDirection(1), HexDirection(2), HexDirection(3), HexDirection(4), HexDirection(5)];

    pub(crate) fn value(self) -> usize {
        self.0
    }
}