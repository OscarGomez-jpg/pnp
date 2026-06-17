/// Constructor V9 (Recursive Edge Insertion) optimizado para ILS.
///
/// Diferencias con V9 base:
/// - Usa un array `visited` en lugar de `Vec::contains` (O(1) vs O(n)).
/// - Construye el K-D tree una sola vez.
/// - No implementa `Strategy`; devuelve un tour completo.
use crate::core::{Node, insertion_cost};
use crate::strategies::triangle_insertion_v9::{KDTree, V9Params};
use macroquad::prelude::Vec2;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Clone, Copy)]
pub struct V9ConstructionParams {
    pub k_neighbors: usize,
    pub w_angle: f32,
    pub w_cost: f32,
    pub w_density: f32,
}

impl From<V9Params> for V9ConstructionParams {
    fn from(p: V9Params) -> Self {
        Self {
            k_neighbors: p.k_neighbors,
            w_angle: p.w_angle,
            w_cost: p.w_cost,
            w_density: p.w_density,
        }
    }
}

#[derive(Clone)]
struct EdgeCandidate {
    score: f32,
    edge_i: usize,
    node: usize,
}

impl PartialEq for EdgeCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}
impl Eq for EdgeCandidate {}

impl PartialOrd for EdgeCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl Ord for EdgeCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.partial_cmp(&other.score).unwrap_or(Ordering::Equal)
    }
}

pub struct V9Constructor {
    params: V9ConstructionParams,
}

impl V9Constructor {
    pub fn new(params: V9ConstructionParams) -> Self {
        Self { params }
    }

    pub fn build(&self, nodes: &[Node]) -> Vec<usize> {
        if nodes.len() < 3 {
            return (0..nodes.len()).collect();
        }

        let mut path = self.convex_hull(nodes);
        if path.len() == nodes.len() {
            return path;
        }

        let mut visited = vec![false; nodes.len()];
        for &idx in &path {
            visited[idx] = true;
        }

        let points: Vec<(Vec2, usize)> = nodes.iter().enumerate().map(|(i, n)| (n.pos, i)).collect();
        let kdtree = KDTree::build(&points);
        let k = self.params.k_neighbors.max(1);

        while path.len() < nodes.len() {
            if let Some((node, pos)) = self.find_best_insertion(&path, nodes, &visited, &kdtree, k) {
                visited[node] = true;
                path.insert(pos, node);
            } else {
                // Fallback: insertar el primer no visitado.
                if let Some(node) = (0..nodes.len()).find(|&i| !visited[i]) {
                    visited[node] = true;
                    path.insert(1, node);
                }
            }
        }

        path
    }

    fn convex_hull(&self, nodes: &[Node]) -> Vec<usize> {
        let mut indexed: Vec<usize> = (0..nodes.len()).collect();
        indexed.sort_by(|&a, &b| {
            let pa = nodes[a].pos;
            let pb = nodes[b].pos;
            pa.x
                .partial_cmp(&pb.x)
                .unwrap_or(Ordering::Equal)
                .then(pa.y.partial_cmp(&pb.y).unwrap_or(Ordering::Equal))
        });

        let cross = |o: usize, a: usize, b: usize| -> f32 {
            let po = nodes[o].pos;
            let pa = nodes[a].pos;
            let pb = nodes[b].pos;
            (pa.x - po.x) * (pb.y - po.y) - (pa.y - po.y) * (pb.x - po.x)
        };

        let mut lower: Vec<usize> = Vec::new();
        for &idx in &indexed {
            while lower.len() >= 2
                && cross(lower[lower.len() - 2], lower[lower.len() - 1], idx) <= 0.0
            {
                lower.pop();
            }
            lower.push(idx);
        }

        let mut upper: Vec<usize> = Vec::new();
        for &idx in indexed.iter().rev() {
            while upper.len() >= 2
                && cross(upper[upper.len() - 2], upper[upper.len() - 1], idx) <= 0.0
            {
                upper.pop();
            }
            upper.push(idx);
        }

        lower.pop();
        upper.pop();
        lower.extend(upper);
        lower
    }

    fn find_best_insertion(
        &self,
        path: &[usize],
        nodes: &[Node],
        visited: &[bool],
        kdtree: &KDTree,
        k: usize,
    ) -> Option<(usize, usize)> {
        let n = path.len();
        if n < 2 {
            return (0..nodes.len()).find(|&i| !visited[i]).map(|i| (i, 0));
        }

        let mut heap: BinaryHeap<EdgeCandidate> = BinaryHeap::new();

        for i in 0..n {
            let j = (i + 1) % n;
            let p_i = nodes[path[i]].pos;
            let p_j = nodes[path[j]].pos;
            let midpoint = (p_i + p_j) * 0.5;

            let nearby = kdtree.find_k_nearest(midpoint, k);
            let unvisited_nearby = nearby.iter().filter(|&&c| !visited[c]).count();
            let density_ratio = unvisited_nearby as f32 / k as f32;

            for &candidate in &nearby {
                if visited[candidate] {
                    continue;
                }

                let score = self.insertion_score(path[i], path[j], candidate, nodes, density_ratio);
                heap.push(EdgeCandidate {
                    score,
                    edge_i: i,
                    node: candidate,
                });
            }
        }

        while let Some(best) = heap.pop() {
            if !visited[best.node] {
                let insert_pos = (best.edge_i + 1).min(n);
                return Some((best.node, insert_pos));
            }
        }

        None
    }

    fn insertion_score(
        &self,
        i: usize,
        j: usize,
        u: usize,
        nodes: &[Node],
        density_ratio: f32,
    ) -> f32 {
        let insertion_angle = Self::compute_insertion_angle(i, j, u, nodes);
        let angle_score = insertion_angle / std::f32::consts::PI;

        let cost = insertion_cost(i, j, u, nodes);
        let edge_len = nodes[i].pos.distance(nodes[j].pos);
        let cost_ratio = if edge_len > 1e-5 { cost / edge_len } else { 1.0 };
        let cost_penalty = 1.0 / (1.0 + cost_ratio);

        let density_score = density_ratio.clamp(0.0, 1.0);

        angle_score * self.params.w_angle
            + cost_penalty * self.params.w_cost
            + density_score * self.params.w_density
    }

    fn compute_insertion_angle(i: usize, j: usize, u: usize, nodes: &[Node]) -> f32 {
        let v1 = nodes[i].pos - nodes[u].pos;
        let v2 = nodes[j].pos - nodes[u].pos;
        let len1 = v1.length();
        let len2 = v2.length();
        if len1 < 1e-5 || len2 < 1e-5 {
            return 0.0;
        }
        let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
        cos_theta.acos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    #[test]
    fn test_constructor_visits_all() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
            Node::new(5.0, 5.0),
        ];

        let ctor = V9Constructor::new(V9ConstructionParams::from(V9Params::default()));
        let path = ctor.build(&nodes);
        assert_eq!(path.len(), nodes.len());
    }
}
