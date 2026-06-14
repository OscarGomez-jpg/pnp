#![allow(unused)]
/// Estrategia: Triangle Insertion V8 — Outside-In Angle Optimization
///
/// Enfoque inverso a V6/V7: comienza desde afuera (casco convexo completo)
/// y envuelve hacia adentro, seleccionando puntos por:
///   1. Ángulo más abierto (cercano a 180°) en el punto de inserción
///   2. Impacto en ángulos vecinos: prioriza inserciones que mejoren
///      la suavidad global del tour circundante
///
/// Analogía: Como un elástico que abraza los puntos exteriores y luego
/// se contrae suavemente hacia los interiores, minimizando giros bruscos.
use super::Strategy;
use crate::core::{Node, insertion_cost, path_distance};
use macroquad::prelude::Vec2;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// =============================================================================
// K-D Tree para Búsqueda de Vecinos (reutilizado de V7)
// =============================================================================

#[derive(Clone)]
struct KDNode {
    point: Vec2,
    index: usize,
    left: Option<Box<KDNode>>,
    right: Option<Box<KDNode>>,
}

impl KDNode {
    fn new(point: Vec2, index: usize) -> Self {
        Self {
            point,
            index,
            left: None,
            right: None,
        }
    }
}

struct KDTree {
    root: Option<Box<KDNode>>,
}

impl KDTree {
    fn build(points: &[(Vec2, usize)]) -> Self {
        let mut pts = points.to_vec();
        Self {
            root: Self::build_recursive(&mut pts, 0),
        }
    }

    fn build_recursive(points: &mut [(Vec2, usize)], depth: u32) -> Option<Box<KDNode>> {
        if points.is_empty() {
            return None;
        }

        let axis = depth % 2;
        points.sort_unstable_by(|a, b| {
            if axis == 0 {
                a.0.x.partial_cmp(&b.0.x).unwrap_or(Ordering::Equal)
            } else {
                a.0.y.partial_cmp(&b.0.y).unwrap_or(Ordering::Equal)
            }
        });

        let mid = points.len() / 2;
        let (point, index) = points[mid];
        let mut node = KDNode::new(point, index);

        node.left = Self::build_recursive(&mut points[..mid], depth + 1);
        node.right = Self::build_recursive(&mut points[mid + 1..], depth + 1);

        Some(Box::new(node))
    }

    fn find_k_nearest(&self, query: Vec2, k: usize) -> Vec<usize> {
        let mut heap: BinaryHeap<DistanceItem> = BinaryHeap::with_capacity(k);
        if let Some(ref root) = self.root {
            Self::search_nearest(root, query, k, &mut heap, 0);
        }
        heap.into_iter().map(|item| item.index).collect()
    }

    fn search_nearest(
        node: &KDNode,
        query: Vec2,
        k: usize,
        heap: &mut BinaryHeap<DistanceItem>,
        depth: u32,
    ) {
        let dist = query.distance(node.point);
        let item = DistanceItem {
            dist,
            index: node.index,
        };

        if heap.len() < k {
            heap.push(item);
        } else if let Some(top) = heap.peek() {
            if dist < top.dist {
                heap.pop();
                heap.push(item);
            }
        }

        let axis = depth % 2;
        let diff = if axis == 0 {
            query.x - node.point.x
        } else {
            query.y - node.point.y
        };

        let (first, second) = if diff < 0.0 {
            (&node.left, &node.right)
        } else {
            (&node.right, &node.left)
        };

        if let Some(child) = first {
            Self::search_nearest(child, query, k, heap, depth + 1);
        }

        if let Some(child) = second {
            let max_dist_in_heap = heap.peek().map_or(f32::MAX, |top| top.dist);
            if heap.len() < k || diff.abs() < max_dist_in_heap {
                Self::search_nearest(child, query, k, heap, depth + 1);
            }
        }
    }
}

#[derive(Clone)]
struct DistanceItem {
    dist: f32,
    index: usize,
}

impl PartialEq for DistanceItem {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
    }
}
impl Eq for DistanceItem {}

impl PartialOrd for DistanceItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.dist.partial_cmp(&other.dist)
    }
}

impl Ord for DistanceItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist
            .partial_cmp(&other.dist)
            .unwrap_or(Ordering::Equal)
    }
}

// =============================================================================
// Triangle Insertion V8
// =============================================================================

pub struct TriangleInsertionV8 {
    initialized: bool,
    unvisited: Vec<usize>,
    k_neighbors: usize,
}

impl TriangleInsertionV8 {
    pub fn new() -> Self {
        Self {
            initialized: false,
            unvisited: Vec::new(),
            k_neighbors: 8,
        }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo Completo (Outside-In)
    // -------------------------------------------------------------------------

    fn convex_hull(nodes: &[Node]) -> Vec<usize> {
        if nodes.is_empty() {
            return Vec::new();
        }
        if nodes.len() < 3 {
            return (0..nodes.len()).collect();
        }

        let mut indexed: Vec<usize> = (0..nodes.len()).collect();
        indexed.sort_by(|&a, &b| {
            let pa = nodes[a].pos;
            let pb = nodes[b].pos;
            pa.x.partial_cmp(&pb.x)
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

    // -------------------------------------------------------------------------
    // V8 Core: Angle-Optimized Insertion from Outside-In
    // -------------------------------------------------------------------------

    /// Calcula el ángulo en el punto `u` formado por los vectores hacia `a` y `b`
    fn angle_at_point(a: usize, u: usize, b: usize, nodes: &[Node]) -> f32 {
        let p_a = nodes[a].pos;
        let p_u = nodes[u].pos;
        let p_b = nodes[b].pos;

        let v1 = p_a - p_u;
        let v2 = p_b - p_u;

        let len1 = v1.length();
        let len2 = v2.length();

        if len1 < 1e-5 || len2 < 1e-5 {
            return 0.0;
        }

        let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
        cos_theta.acos()
    }

    /// Calcula el ángulo promedio de los vecinos adyacentes a una posición en el path
    fn neighborhood_angle_score(path: &[usize], pos: usize, nodes: &[Node]) -> f32 {
        let n = path.len();
        if n < 3 {
            return std::f32::consts::PI;
        }

        let prev = (pos + n - 1) % n;
        let curr = pos % n;
        let next = (pos + 1) % n;

        let angle_prev = Self::angle_at_point(path[prev], path[curr], path[next], nodes);

        let prev2 = (pos + n - 2) % n;
        let next2 = (pos + 2) % n;

        let angle_prev2 = Self::angle_at_point(path[prev2], path[prev], path[curr], nodes);
        let angle_next2 = Self::angle_at_point(path[curr], path[next], path[next2], nodes);

        (angle_prev * 2.0 + angle_prev2 + angle_next2) / 4.0
    }

    /// Inserción Outside-In con optimización de ángulos
    ///
    /// Para cada punto no visitado:
    /// 1. Encuentra el punto más cercano al path como referencia
    /// 2. Busca k vecinos cercanos a ese punto
    /// 3. Evalúa cada candidato por:
    ///    - Ángulo de inserción (cercano a 180° = mejor)
    ///    - Impacto en ángulos vecinos después de la inserción
    /// 4. Selecciona el que maximiza la suavidad angular global
    fn outside_in_insertion(
        &self,
        path: &[usize],
        nodes: &[Node],
        kdtree: &KDTree,
    ) -> (usize, usize) {
        if self.unvisited.is_empty() {
            return (0, 0);
        }

        let mut best_node = self.unvisited[0];
        let mut best_pos = 1;
        let mut best_score = f32::MIN;

        let path_center = Self::compute_path_center(path, nodes);
        let reference_candidates = kdtree.find_k_nearest(path_center, self.k_neighbors);

        for &ref_candidate in &reference_candidates {
            if !self.unvisited.contains(&ref_candidate) {
                continue;
            }

            let local_candidates = kdtree.find_k_nearest(nodes[ref_candidate].pos, self.k_neighbors);

            for &candidate in &local_candidates {
                if !self.unvisited.contains(&candidate) {
                    continue;
                }

                for i in 0..path.len() {
                    let next = (i + 1) % path.len();

                    let insertion_angle = Self::compute_insertion_angle(
                        path[i],
                        path[next],
                        candidate,
                        nodes,
                    );

                    let angle_score = insertion_angle / std::f32::consts::PI;

                    let mut hypothetical_path = path.to_vec();
                    hypothetical_path.insert(i + 1, candidate);

                    let neighborhood_score = Self::neighborhood_angle_score(
                        &hypothetical_path,
                        i + 1,
                        nodes,
                    ) / std::f32::consts::PI;

                    let cost = insertion_cost(path[i], path[next], candidate, nodes);
                    let edge_len = nodes[path[i]].pos.distance(nodes[path[next]].pos);
                    let cost_ratio = if edge_len > 1e-5 { cost / edge_len } else { 1.0 };
                    let cost_penalty = 1.0 / (1.0 + cost_ratio);

                    let total_score = angle_score * 0.5
                        + neighborhood_score * 0.3
                        + cost_penalty * 0.2;

                    if total_score > best_score {
                        best_score = total_score;
                        best_node = candidate;
                        best_pos = i + 1;
                    }
                }
            }
        }

        (best_node, best_pos)
    }

    fn compute_path_center(path: &[usize], nodes: &[Node]) -> Vec2 {
        if path.is_empty() {
            return Vec2::ZERO;
        }

        let mut sum = Vec2::ZERO;
        for &idx in path {
            sum += nodes[idx].pos;
        }
        sum / path.len() as f32
    }

    fn compute_insertion_angle(i: usize, j: usize, u: usize, nodes: &[Node]) -> f32 {
        let p_i = nodes[i].pos;
        let p_j = nodes[j].pos;
        let p_u = nodes[u].pos;

        let v1 = p_i - p_u;
        let v2 = p_j - p_u;

        let len1 = v1.length();
        let len2 = v2.length();

        if len1 < 1e-5 || len2 < 1e-5 {
            return 0.0;
        }

        let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
        cos_theta.acos()
    }

    // -------------------------------------------------------------------------
    // Post-optimización: 2-Opt, Or-Opt, Node Reinsertion
    // -------------------------------------------------------------------------

    fn optimize_2opt(path: &mut Vec<usize>, nodes: &[Node], max_iterations: usize) -> bool {
        let mut improved = false;
        for _ in 0..max_iterations {
            let mut local_improved = false;
            for i in 0..path.len().saturating_sub(2) {
                for j in (i + 2)..path.len() {
                    if i == 0 && j == path.len() - 1 {
                        continue;
                    }
                    let p1 = nodes[path[i]].pos;
                    let p2 = nodes[path[i + 1]].pos;
                    let p3 = nodes[path[j]].pos;
                    let p4 = nodes[path[(j + 1) % path.len()]].pos;

                    let current = p1.distance(p2) + p3.distance(p4);
                    let swapped = p1.distance(p3) + p2.distance(p4);

                    if swapped < current - 0.01 {
                        path[i + 1..=j].reverse();
                        local_improved = true;
                        improved = true;
                    }
                }
            }
            if !local_improved {
                break;
            }
        }
        improved
    }

    fn optimize_or_opt(path: &mut Vec<usize>, nodes: &[Node], seg_len: usize) -> bool {
        let n = path.len();
        if n < seg_len + 2 {
            return false;
        }

        let mut improved = true;
        let mut ever_improved = false;

        while improved {
            improved = false;
            for i in 0..n {
                let seg: Vec<usize> = (0..seg_len).map(|k| path[(i + k) % n]).collect();
                let p_prev = path[(i + n - 1) % n];
                let p_first = path[i];
                let p_last = path[(i + seg_len - 1) % n];
                let p_next = path[(i + seg_len) % n];

                let removal_gain = nodes[p_prev].distance_to(&nodes[p_first])
                    + nodes[p_last].distance_to(&nodes[p_next])
                    - nodes[p_prev].distance_to(&nodes[p_next]);

                let mut reduced = Vec::with_capacity(n - seg_len);
                for k in 0..n {
                    let mut in_seg = false;
                    for offset in 0..seg_len {
                        if k == (i + offset) % n {
                            in_seg = true;
                            break;
                        }
                    }
                    if !in_seg {
                        reduced.push(path[k]);
                    }
                }

                let m = reduced.len();
                for j in 0..=m {
                    let r_prev = reduced[(j + m - 1) % m];
                    let r_next = reduced[j % m];

                    let ins_cost = nodes[r_prev].distance_to(&nodes[p_first])
                        + nodes[p_last].distance_to(&nodes[r_next])
                        - nodes[r_prev].distance_to(&nodes[r_next]);

                    if ins_cost < removal_gain - 0.01 {
                        let mut new_path = Vec::with_capacity(n);
                        new_path.extend_from_slice(&reduced[..j]);
                        new_path.extend_from_slice(&seg);
                        new_path.extend_from_slice(&reduced[j..]);
                        *path = new_path;
                        improved = true;
                        ever_improved = true;
                        break;
                    }
                }
                if improved {
                    break;
                }
            }
        }
        ever_improved
    }

    fn optimize_node_reinsertion(path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let n = path.len();
        if n < 4 {
            return false;
        }
        let mut ever_improved = false;
        let mut improved = true;

        while improved {
            improved = false;
            for i in 0..n {
                let node_idx = path[i];
                let p_prev = path[(i + n - 1) % n];
                let p_next = path[(i + 1) % n];

                let removal_gain = nodes[p_prev].distance_to(&nodes[node_idx])
                    + nodes[node_idx].distance_to(&nodes[p_next])
                    - nodes[p_prev].distance_to(&nodes[p_next]);

                let mut reduced = path.clone();
                reduced.remove(i);

                let m = reduced.len();
                for j in 0..=m {
                    let r_prev = reduced[(j + m - 1) % m];
                    let r_next = reduced[j % m];

                    let ins_cost = nodes[r_prev].distance_to(&nodes[node_idx])
                        + nodes[node_idx].distance_to(&nodes[r_next])
                        - nodes[r_prev].distance_to(&nodes[r_next]);

                    if ins_cost < removal_gain - 0.01 {
                        reduced.insert(j, node_idx);
                        *path = reduced;
                        improved = true;
                        ever_improved = true;
                        break;
                    }
                }
                if improved {
                    break;
                }
            }
        }
        ever_improved
    }
}

// =============================================================================
// Implementación del Trait Strategy
// =============================================================================

impl Strategy for TriangleInsertionV8 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if current_path.is_empty() && self.unvisited.is_empty() {
            self.unvisited = (0..nodes.len()).collect();
        }

        if self.unvisited.is_empty() {
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
            Self::optimize_node_reinsertion(current_path, nodes);
            Self::optimize_2opt(current_path, nodes, 5);
            return true;
        }

        if current_path.is_empty() {
            if nodes.len() < 3 {
                current_path.extend(0..nodes.len());
                self.unvisited.clear();
                return true;
            }

            let hull = Self::convex_hull(nodes);
            for &idx in &hull {
                if let Some(pos) = self.unvisited.iter().position(|&x| x == idx) {
                    self.unvisited.swap_remove(pos);
                }
            }
            current_path.extend_from_slice(&hull);
            self.initialized = true;
            return false;
        }

        if !self.initialized {
            return true;
        }

        let points: Vec<(Vec2, usize)> = self
            .unvisited
            .iter()
            .map(|&i| (nodes[i].pos, i))
            .collect();

        if points.is_empty() {
            return true;
        }

        let kdtree = KDTree::build(&points);

        let (best_node, best_pos) = self.outside_in_insertion(current_path, nodes, &kdtree);

        if let Some(pos) = self.unvisited.iter().position(|&x| x == best_node) {
            self.unvisited.swap_remove(pos);
        }
        current_path.insert(best_pos, best_node);

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V8 (Outside-In Angle Optimization)"
    }

    fn reset(&mut self) {
        self.initialized = false;
        self.unvisited.clear();
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    fn run_to_completion(strategy: &mut TriangleInsertionV8, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 10 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v8_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV8::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_v8_convex_hull_initialization() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
            Node::new(5.0, 5.0),
        ];
        let mut strategy = TriangleInsertionV8::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 5, "Debe visitar todos los nodos incluyendo el centro");
    }

    #[test]
    fn test_v8_outside_in_approach() {
        let nodes: Vec<Node> = (0..20)
            .map(|i| {
                let angle = i as f32 * std::f32::consts::PI * 2.0 / 20.0;
                Node::new(angle.cos() * 10.0, angle.sin() * 10.0)
            })
            .collect();

        let mut strategy = TriangleInsertionV8::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 20, "Debe visitar todos los nodos del círculo");
    }
}
