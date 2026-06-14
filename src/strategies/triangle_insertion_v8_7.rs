#![allow(unused)]
/// Estrategia: Triangle Insertion V8.7 — Onion Peeling (Multi-Layer Convex Hull)
///
/// Mejora de V8.6 que usa múltiples capas de convex hull (onion peeling)
/// para procesar nodos en orden de exterioridad:
///   - Capa 1: Convex hull exterior
///   - Capa 2: Convex hull de nodos restantes
///   - Capa 3: ...
///   - Última capa: Nodos más interiores
///
/// Parámetros configurables:
///   - k_neighbors: número de vecinos a considerar (4, 6, 8, 10, 12, 16)
///   - w_angle: peso del ángulo de inserción (0.0 a 1.0)
///   - w_cost: peso de la penalización por costo (0.0 a 1.0)
use super::Strategy;
use crate::core::{Node, insertion_cost, path_distance};
use macroquad::prelude::Vec2;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// =============================================================================
// K-D Tree para Búsqueda de Vecinos
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
// Parámetros de V8.7
// =============================================================================

#[derive(Clone, Copy)]
pub struct V87Params {
    pub k_neighbors: usize,
    pub w_angle: f32,
    pub w_cost: f32,
}

impl Default for V87Params {
    fn default() -> Self {
        Self {
            k_neighbors: 8,
            w_angle: 0.25,
            w_cost: 0.25,
        }
    }
}

// =============================================================================
// Triangle Insertion V8.7 — Onion Peeling
// =============================================================================

pub struct TriangleInsertionV87 {
    initialized: bool,
    unvisited: Vec<usize>,
    layers: Vec<Vec<usize>>,
    current_layer: usize,
    params: V87Params,
}

impl TriangleInsertionV87 {
    pub fn new() -> Self {
        Self {
            initialized: false,
            unvisited: Vec::new(),
            layers: Vec::new(),
            current_layer: 0,
            params: V87Params::default(),
        }
    }

    pub fn with_params(params: V87Params) -> Self {
        Self {
            initialized: false,
            unvisited: Vec::new(),
            layers: Vec::new(),
            current_layer: 0,
            params,
        }
    }

    pub fn set_params(&mut self, params: V87Params) {
        self.params = params;
    }

    pub fn get_params(&self) -> V87Params {
        self.params
    }

    pub fn load_calibrated_params<P: AsRef<std::path::Path>>(&mut self, path: P) -> bool {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut k_neighbors = None;
            let mut w_angle = None;
            let mut w_cost = None;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once(':') {
                    match key.trim() {
                        "k_neighbors" => k_neighbors = value.trim().parse().ok(),
                        "w_angle" => w_angle = value.trim().parse().ok(),
                        "w_cost" => w_cost = value.trim().parse().ok(),
                        _ => {}
                    }
                }
            }

            if let (Some(k), Some(a), Some(c)) = (k_neighbors, w_angle, w_cost) {
                self.params = V87Params {
                    k_neighbors: k,
                    w_angle: a,
                    w_cost: c,
                };
                return true;
            }
        }
        false
    }

    // -------------------------------------------------------------------------
    // Convex Hull (Graham Scan)
    // -------------------------------------------------------------------------

    fn convex_hull(nodes: &[Node], indices: &[usize]) -> Vec<usize> {
        if indices.is_empty() {
            return Vec::new();
        }
        if indices.len() < 3 {
            return indices.to_vec();
        }

        let mut sorted = indices.to_vec();
        sorted.sort_by(|&a, &b| {
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
        for &idx in &sorted {
            while lower.len() >= 2
                && cross(lower[lower.len() - 2], lower[lower.len() - 1], idx) <= 0.0
            {
                lower.pop();
            }
            lower.push(idx);
        }

        let mut upper: Vec<usize> = Vec::new();
        for &idx in sorted.iter().rev() {
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
    // Onion Peeling: Múltiples capas de convex hull
    // -------------------------------------------------------------------------

    fn compute_onion_layers(nodes: &[Node]) -> Vec<Vec<usize>> {
        let n = nodes.len();
        let mut remaining: Vec<usize> = (0..n).collect();
        let mut layers: Vec<Vec<usize>> = Vec::new();

        while !remaining.is_empty() {
            let hull = Self::convex_hull(nodes, &remaining);
            if hull.is_empty() {
                break;
            }

            // Eliminar nodos del hull de remaining
            let hull_set: std::collections::HashSet<usize> = hull.iter().copied().collect();
            remaining.retain(|idx| !hull_set.contains(idx));

            layers.push(hull);
        }

        layers
    }

    // -------------------------------------------------------------------------
    // Scoring de Inserción
    // -------------------------------------------------------------------------

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

    fn find_best_insertion(
        &self,
        candidate: usize,
        path: &[usize],
        nodes: &[Node],
    ) -> (usize, f32) {
        let mut best_pos = 1;
        let mut best_score = f32::MIN;

        for i in 0..path.len() {
            let next = (i + 1) % path.len();

            let insertion_angle = Self::compute_insertion_angle(path[i], path[next], candidate, nodes);
            let angle_score = insertion_angle / std::f32::consts::PI;

            let cost = insertion_cost(path[i], path[next], candidate, nodes);
            let edge_len = nodes[path[i]].pos.distance(nodes[path[next]].pos);
            let cost_ratio = if edge_len > 1e-5 {
                cost / edge_len
            } else {
                1.0
            };
            let cost_penalty = 1.0 / (1.0 + cost_ratio);

            let total_score = angle_score * self.params.w_angle + cost_penalty * self.params.w_cost;

            if total_score > best_score {
                best_score = total_score;
                best_pos = i + 1;
            }
        }

        (best_pos, best_score)
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

                    if swapped < current {
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

impl Strategy for TriangleInsertionV87 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if current_path.is_empty() && self.unvisited.is_empty() && self.layers.is_empty() {
            // Calcular capas de onion peeling
            self.layers = Self::compute_onion_layers(nodes);
            self.unvisited = (0..nodes.len()).collect();
            self.current_layer = 0;
        }

        if self.unvisited.is_empty() {
            Self::optimize_2opt(current_path, nodes, 20);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
            Self::optimize_node_reinsertion(current_path, nodes);
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_2opt(current_path, nodes, 5);
            return true;
        }

        if current_path.is_empty() {
            if nodes.len() < 3 {
                current_path.extend(0..nodes.len());
                self.unvisited.clear();
                return true;
            }

            // Inicializar con la primera capa (convex hull exterior)
            if let Some(first_layer) = self.layers.first() {
                for &idx in first_layer {
                    if let Some(pos) = self.unvisited.iter().position(|&x| x == idx) {
                        self.unvisited.swap_remove(pos);
                    }
                }
                current_path.extend_from_slice(first_layer);
            }
            self.current_layer = 1;
            self.initialized = true;
            return false;
        }

        if !self.initialized {
            return true;
        }

        // Procesar nodos de la capa actual
        if self.current_layer < self.layers.len() {
            let layer = &self.layers[self.current_layer];
            
            // Insertar todos los nodos de esta capa
            for &node in layer {
                if let Some(pos) = self.unvisited.iter().position(|&x| x == node) {
                    self.unvisited.swap_remove(pos);
                }
                
                let (best_pos, _) = self.find_best_insertion(node, current_path, nodes);
                current_path.insert(best_pos, node);
            }
            
            self.current_layer += 1;
            return false;
        }

        // Si no hay más capas, insertar nodos restantes (si los hay)
        if !self.unvisited.is_empty() {
            let candidate = self.unvisited[0];
            let (best_pos, _) = self.find_best_insertion(candidate, current_path, nodes);
            if let Some(pos) = self.unvisited.iter().position(|&x| x == candidate) {
                self.unvisited.swap_remove(pos);
            }
            current_path.insert(best_pos, candidate);
            return false;
        }

        true
    }

    fn name(&self) -> &str {
        "Triangle Insertion V8.7 (Onion Peeling)"
    }

    fn reset(&mut self) {
        self.initialized = false;
        self.unvisited.clear();
        self.layers.clear();
        self.current_layer = 0;
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

    fn run_to_completion(strategy: &mut TriangleInsertionV87, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 10 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v87_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV87::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_v87_with_custom_params() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
            Node::new(5.0, 5.0),
        ];
        let params = V87Params {
            k_neighbors: 10,
            w_angle: 0.7,
            w_cost: 0.3,
        };
        let mut strategy = TriangleInsertionV87::with_params(params);
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 5, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_onion_layers() {
        // Cuadrado con centro
        let nodes = vec![
            Node::new(0.0, 0.0),   // 0: esquina
            Node::new(10.0, 0.0),  // 1: esquina
            Node::new(10.0, 10.0), // 2: esquina
            Node::new(0.0, 10.0),  // 3: esquina
            Node::new(5.0, 5.0),   // 4: centro
        ];

        let layers = TriangleInsertionV87::compute_onion_layers(&nodes);
        assert_eq!(layers.len(), 2, "Debe haber 2 capas");
        assert_eq!(layers[0].len(), 4, "Capa 1 debe tener 4 nodos (esquinas)");
        assert_eq!(layers[1].len(), 1, "Capa 2 debe tener 1 nodo (centro)");
    }
}
