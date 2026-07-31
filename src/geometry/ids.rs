#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileId(usize);

impl TileId {
    pub fn new(index: usize) -> TileId {
        TileId(index)
    }
    pub fn value(&self) -> usize {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct VertexId(usize);

impl VertexId {
    pub fn new(index: usize) -> VertexId {
        VertexId(index)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(usize);

impl EdgeId {
    pub(crate) fn value(&self) -> usize {
        self.0
    }
}

impl EdgeId {
    pub fn new(index: usize) -> EdgeId {
        EdgeId(index)
    }
}