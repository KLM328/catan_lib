use crate::{EdgeId, Hex, TileId, Topology, VertexId};

const SQRT_3: f32 = 1.732_050_8;
pub struct Layout {
    pub hex_size: f32,
    pub origin: (f32, f32),
}

impl Layout {
    pub fn tile_position(&self, topo: &Topology, t: TileId) -> (f32, f32) {
        self.hex_to_pixel(topo.hexes()[t.value()])
    }
    pub fn vertex_position(&self, topo: &Topology, v: VertexId) -> (f32, f32) {
        let (x, y) = topo.vertex_hexes()[v.value()]
            .iter()
            .map(|h| self.hex_to_pixel(*h))
            .fold((0.0, 0.0), |(ax, ay), (hx, hy)| (ax + hx, ay + hy));
        (x / 3.0, y / 3.0)
    }

    pub fn edge_position(&self, topo: &Topology, e: EdgeId) -> ((f32, f32), (f32, f32)) {
        let [a, b] = topo.edges_endpoints()[e.value()];
        (self.vertex_position(topo, a), self.vertex_position(topo, b))
    }

    // pub fn pick_tile(&self, topo: &Topology, p: (f32, f32)) -> Option<TileId>;
    pub fn pick_vertex(&self, topo: &Topology, p: (f32, f32), radius: f32) -> Option<VertexId> {
        let (x, y) = p;
        (0..topo.vertex_count() - 1)
            .into_iter()
            .map(|index| {
                let v = VertexId::new(index);
                let (a, b) = self.vertex_position(topo, v);
                (v, (x - a).powi(2) + (y - b).powi(2))
            })
            .filter(|(_, d)| d < &radius.powi(2))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(v, _)| v)
    }

    pub fn pick_edge(&self, topo: &Topology, p: (f32, f32), radius: f32) -> Option<EdgeId> {
        let (x, y) = p;
        (0..topo.edge_count() - 1)
            .into_iter()
            .map(|index| {
                let e = EdgeId::new(index);
                (e, self.edge_position(topo, e))
            })
            .map(|(e, ((a, b), (c, d)))| (e, ((a + c) / 2.0, (b + d) / 2.0)))
            .map(|(e, (a, b))| (e, (x - a).powi(2) + (y - b).powi(2)))
            .filter(|(_, d)| d < &radius.powi(2))
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(v, _)| v)
    }

    fn hex_to_pixel(&self, h: Hex) -> (f32, f32) {
        let x = self.hex_size * SQRT_3 * (h.q() as f32 + h.r() as f32 / 2.0);
        let y = self.hex_size * 1.5 * h.r() as f32;
        (x + self.origin.0, y + self.origin.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Topology;

    #[test]
    fn les_sommets_sont_a_hex_size_du_centre_de_leur_tuile() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        let layout = Layout {
            hex_size: 40.0,
            origin: (0.0, 0.0),
        };

        for (i, vertices) in topo.tile_vertices().iter().enumerate() {
            let center = layout.tile_position(&topo, TileId::new(i));
            for &v in vertices {
                let (x, y) = layout.vertex_position(&topo, v);
                let d = ((x - center.0).powi(2) + (y - center.1).powi(2)).sqrt();
                assert!(
                    (d - 40.0).abs() < 0.01,
                    "sommet {v:?} à {d} du centre de la tuile {i}"
                );
            }
        }
    }

    #[test]
    fn tous_les_sommets_ont_une_position_distincte() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        let layout = Layout {
            hex_size: 40.0,
            origin: (0.0, 0.0),
        };
        let mut points: Vec<(i64, i64)> = (0..topo.vertex_count())
            .map(|v| layout.vertex_position(&topo, VertexId::new(v)))
            .map(|(x, y)| ((x * 1000.0) as i64, (y * 1000.0) as i64))
            .collect();
        points.sort();
        let before = points.len();
        points.dedup();
        assert_eq!(points.len(), before);
    }
}
