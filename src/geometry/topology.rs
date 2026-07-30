use std::collections::HashMap;
use crate::geometry::{Hex, HexCorner, HexDirection};
use crate::{EdgeId, VertexId};

pub struct Topology{
    hexes: Vec<Hex>,
    tile_vertices: Vec<[VertexId; 6]>,
    tile_edges: Vec<[EdgeId; 6]>,
    vertex_count: usize,
    edge_count: usize,
}

impl Topology {

    fn new() -> Topology{
        Topology{hexes: Vec::new(), tile_vertices: Vec::new(), tile_edges: Vec::new(), vertex_count: 0, edge_count: 0}
    }
    pub fn from_hexes(hexes : &[Hex]) -> Topology{
        let mut topology = Topology::new();
        topology.hexes = hexes.to_vec();
        let mut vertices: HashMap<[Hex; 3], VertexId> = HashMap::new();
        let mut edges: HashMap<[Hex; 2], EdgeId> = HashMap::new();

        for hex in hexes {
            let mut new_tile_for_vertices: [VertexId; 6] = [VertexId::new(0); 6];
            let mut new_tile_for_edges : [EdgeId; 6] = [EdgeId::new(0); 6];




            for corner in HexCorner::ALL {
                let new_index_vertices = vertices.len();

                new_tile_for_vertices[corner.value()] = *vertices.entry(hex.corner_hexes(corner)).or_insert(VertexId::new(new_index_vertices));
            }
            for dir in HexDirection::ALL {
                let new_index_edges = edges.len();
                new_tile_for_edges[dir.value()] = *edges.entry(hex.edge_hexes(dir)).or_insert(EdgeId::new(new_index_edges));

            }
            topology.tile_vertices.push(new_tile_for_vertices);
            topology.tile_edges.push(new_tile_for_edges);
        }
        topology.vertex_count = vertices.len();
        topology.edge_count = edges.len();
        topology

    }

    pub fn hexagon(radius: i8) -> Vec<Hex> {
        let mut hexes = Vec::new();
        for q in -radius..=radius {
            for r in -radius..=radius {
                if q.abs().max(r.abs()).max((q + r).abs()) <= radius {
                    hexes.push(Hex::new(q, r));
                }
            }
        }
        hexes
    }


    // fn hexagon(radius : u8) -> Vec<Hex>{
    //     let mut hexes : HashSet<Hex> = HashSet::new();
    //     hexes.insert(Hex{q: 0, r: 0});
    //     for _ in 0..radius {
    //         let mut temp : HashSet<Hex> = HashSet::new();
    //         for hex in hexes.iter() {
    //             DIRS.iter().enumerate().for_each(|(i, _)| {temp.insert(hex.neighbor(i));});
    //         }
    //         for hex in temp.iter() {
    //             hexes.insert(hex.clone());
    //         }
    //
    //     }
    //     hexes.into_iter().collect::<Vec<Hex>>()
    // }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_from_hexes_radius_2() {
        let hexes = Topology::hexagon(2);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.tile_vertices.len(), 19);
        assert_eq!(topo.vertex_count, 54);
        assert_eq!(topo.edge_count, 72);
    }

    #[test]
    fn test_from_hexes_radius_3() {
        let hexes = Topology::hexagon(3);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.tile_vertices.len(), 37);
        assert_eq!(topo.vertex_count, 96);
        assert_eq!(topo.edge_count, 132);
    }

    #[test]
    fn test_hexagon_radius_2() {
        let hexes = Topology::hexagon(2);
        println!("{:?}", hexes);
        assert_eq!(hexes.len(), 19);

    }

    #[test]
    fn test_hexagon_radius_1() {
        let hexes = Topology::hexagon(1);
        println!("{:?}", hexes);
        assert_eq!(hexes.len(), 7);
    }
}