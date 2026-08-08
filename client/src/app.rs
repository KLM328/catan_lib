use catan::{EdgeId, PlayerId, ResourceCounts, Steal, TileId, VertexId};

pub enum UiAction {
    Roll,
    NextPlayer,
    BuildSettlement(VertexId),
    BuildRoad(EdgeId),
    UpgradeCity(VertexId),
    MoveRobber(TileId),
    Steal(Option<Steal>),
    Discard(PlayerId, ResourceCounts), //playerId à retiré quand client-serveur
}

