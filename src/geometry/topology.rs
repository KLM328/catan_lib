use std::collections::HashMap;
use crate::geometry::{Hex, HexCorner, HexDirection};
use crate::{EdgeId, VertexId};

pub struct Topology{
    hexes: Vec<Hex>,
    tile_vertices: Vec<[VertexId; 6]>,
    tile_edges: Vec<[EdgeId; 6]>,
    edges_endpoints: Vec<[VertexId; 2]>,
    vertex_count: usize,
    edge_count: usize,
}

impl Topology {

    fn new() -> Topology{
        Topology{hexes: Vec::new(), tile_vertices: Vec::new(), tile_edges: Vec::new(), edges_endpoints : Vec::new(), vertex_count: 0, edge_count: 0}
    }
    pub fn from_hexes(hexes : &[Hex]) -> Topology{
        let mut topology = Topology::new();
        topology.hexes = hexes.to_vec();
        let mut vertices: HashMap<[Hex; 3], VertexId> = HashMap::new();
        let mut edges: HashMap<[Hex; 2], EdgeId> = HashMap::new();
        let mut edges_endpoints : Vec<[VertexId; 2]> = Vec::new();

        for hex in hexes {
            let mut new_tile_for_vertices: [VertexId; 6] = [VertexId::new(0); 6];
            let mut new_tile_for_edges : [EdgeId; 6] = [EdgeId::new(0); 6];




            for corner in HexCorner::ALL {
                let new_index_vertices = vertices.len();

                new_tile_for_vertices[corner.value()] = *vertices.entry(hex.corner_hexes(corner)).or_insert(VertexId::new(new_index_vertices));
            }
            for dir in HexDirection::ALL {
                let new_index_edges = edges.len();
                let edge_id = *edges.entry(hex.edge_hexes(dir)).or_insert(EdgeId::new(new_index_edges));
                new_tile_for_edges[dir.value()] = edge_id;
                if edge_id.value() == edges_endpoints.len() {
                    let mut endpoints = [new_tile_for_vertices[(dir.value() + 5) % 6],
                        new_tile_for_vertices[dir.value()]];
                    endpoints.sort();
                    edges_endpoints.push(endpoints);
                }
            }
            topology.tile_vertices.push(new_tile_for_vertices);
            topology.tile_edges.push(new_tile_for_edges);
        }
        topology.vertex_count = vertices.len();
        topology.edge_count = edges.len();
        topology.edges_endpoints = edges_endpoints;
        topology

    }

    pub fn hexes(&self) -> &[Hex] {
        &self.hexes
    }

    pub fn tile_vertices(&self) -> &[[VertexId; 6]] {
        &self.tile_vertices
    }
    pub fn tile_edges(&self) -> &[[EdgeId; 6]] {
        &self.tile_edges
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub fn edge_count(&self) -> usize {
        self.edge_count
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

    pub fn standard() -> Topology{
        Self::from_hexes(&Self::hexagon(2))
    }

    #[cfg(test)]
    pub fn test_topology() -> Topology{
        let mut hexes = Vec::new();
        hexes.push(Hex::new(0, 0));
        hexes.push(hexes[0].neighbor(HexDirection::new(0).unwrap()));
        hexes.push(hexes[0].neighbor(HexDirection::new(1).unwrap()));
        Topology::from_hexes(&hexes)
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

    #[test]
    fn test_edges_endpoints() {
        let hexes = Topology::hexagon(2);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.edges_endpoints.len(), topo.edge_count());
        let topo = Topology::from_hexes(&[Hex::new(0, 0)]);
        assert_eq!(topo.edges_endpoints, vec![
            [VertexId::new(0), VertexId::new(5)],
            [VertexId::new(0), VertexId::new(1)],
            [VertexId::new(1), VertexId::new(2)],
            [VertexId::new(2), VertexId::new(3)],
            [VertexId::new(3), VertexId::new(4)],
            [VertexId::new(4), VertexId::new(5)],
        ]);
    }

    #[test]
    fn every_vertex_has_two_or_three_edges() {
        let topo = Topology::standard();
        let mut degree = vec![0usize; topo.vertex_count()];
        for e in &topo.edges_endpoints {
            assert_ne!(e[0], e[1], "une arête ne peut pas relier un sommet à lui-même");
            for v in e { degree[v.value()] += 1; }
        }
        assert!(degree.iter().all(|&d| d == 2 || d == 3));
        assert_eq!(degree.iter().filter(|&&d| d == 2).count(), 18);
        assert_eq!(degree.iter().filter(|&&d| d == 3).count(), 36);
    }

    #[test]
    fn every_tile_agrees_on_edge_endpoints() {
        let topo = Topology::standard();
        for (tile, edges) in topo.tile_edges().iter().enumerate() {
            let vertices = topo.tile_vertices()[tile];
            for dir in 0..6 {
                let mut expected = [vertices[(dir + 5) % 6], vertices[dir]];
                expected.sort();
                assert_eq!(topo.edges_endpoints[edges[dir].value()], expected,
                           "tuile {tile}, direction {dir}");
            }
        }
    }
}