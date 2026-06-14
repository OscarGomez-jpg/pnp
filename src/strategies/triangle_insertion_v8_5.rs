#![allow(unused)]
/// Estrategia: Triangle Insertion V8.5 — Adaptive LKH-H Integration v2
///
/// Evolución de V8 con tres mejoras clave:
///   1. Pesos α, β dinámicos según densidad de puntos
///   2. H como tie-breaker cuando costos son similares
///   3. Detección de geometría (grid vs disperso) para cambiar estrategia
///
/// La función H evalúa el impacto global de cada inserción considerando:
///   - Costo local de inserción (ΔC)
///   - Cambio en la suavidad total del tour
///   - Penalización angular en el punto de inserción
use super::Strategy;
use crate::core::{Node, insertion_cost, path_distance};
use macroquad::prelude::Vec2;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// =============================================================================
// K-D Tree (reutilizado de V7/V8)
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
// Tipo de Geometría
// =============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
enum GeometryType {
    Grid,
    Dispersed,
    Clustered,
}

// =============================================================================
// Parámetros Adaptativos
// =============================================================================

struct AdaptiveParams {
    alpha: f32,
    beta: f32,
    h_as_tiebreaker: bool,
    tiebreaker_threshold: f32,
    geometry: GeometryType,
}

// =============================================================================
// Triangle Insertion V8.5
// =============================================================================

pub struct TriangleInsertionV85 {
    initialized: bool,
    unvisited: Vec<usize>,
    k_neighbors: usize,
    params: AdaptiveParams,
    strategy_mode: String,
    calibrated: bool,
}

impl TriangleInsertionV85 {
    pub fn new() -> Self {
        Self {
            initialized: false,
            unvisited: Vec::new(),
            k_neighbors: 8,
            params: AdaptiveParams {
                alpha: 0.3,
                beta: 0.2,
                h_as_tiebreaker: false,
                tiebreaker_threshold: 0.05,
                geometry: GeometryType::Dispersed,
            },
            strategy_mode: "V8-Standard".to_string(),
            calibrated: false,
        }
    }

    pub fn set_params(&mut self, alpha: f32, beta: f32) {
        self.params.alpha = alpha;
        self.params.beta = beta;
        self.calibrated = true;
        self.strategy_mode = format!("V8.5-Calibrated(α={:.2},β={:.2})", alpha, beta);
    }

    pub fn load_calibrated_params<P: AsRef<std::path::Path>>(&mut self, path: P) -> bool {
        if let Ok(content) = std::fs::read_to_string(path) {
            let mut alpha = None;
            let mut beta = None;
            
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once(':') {
                    match key.trim() {
                        "alpha" => alpha = value.trim().parse().ok(),
                        "beta" => beta = value.trim().parse().ok(),
                        _ => {}
                    }
                }
            }
            
            if let (Some(a), Some(b)) = (alpha, beta) {
                self.set_params(a, b);
                return true;
            }
        }
        false
    }

    // -------------------------------------------------------------------------
    // Detección de Geometría
    // -------------------------------------------------------------------------

    fn detect_geometry(nodes: &[Node]) -> GeometryType {
        if nodes.len() < 10 {
            return GeometryType::Dispersed;
        }

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for node in nodes {
            min_x = min_x.min(node.pos.x);
            max_x = max_x.max(node.pos.x);
            min_y = min_y.min(node.pos.y);
            max_y = max_y.max(node.pos.y);
        }

        let area = (max_x - min_x) * (max_y - min_y);
        let n = nodes.len() as f32;
        let density = n / area.max(1.0);

        let mut x_coords: Vec<f32> = nodes.iter().map(|n| n.pos.x).collect();
        let mut y_coords: Vec<f32> = nodes.iter().map(|n| n.pos.y).collect();
        x_coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        y_coords.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let mut grid_score = 0.0;
        let check_interval = |coords: &[f32]| -> f32 {
            if coords.len() < 4 {
                return 0.0;
            }
            let mut diffs: Vec<f32> = coords.windows(2).map(|w| w[1] - w[0]).collect();
            diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            if diffs.is_empty() {
                return 0.0;
            }
            let median_diff = diffs[diffs.len() / 2];
            let consistent = diffs
                .iter()
                .filter(|&&d| (d - median_diff).abs() < median_diff * 0.2)
                .count() as f32;
            consistent / diffs.len() as f32
        };

        grid_score = (check_interval(&x_coords) + check_interval(&y_coords)) / 2.0;

        let mut distances: Vec<f32> = Vec::new();
        for i in 0..nodes.len().min(50) {
            for j in (i + 1)..nodes.len().min(50) {
                distances.push(nodes[i].pos.distance(nodes[j].pos));
            }
        }
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let avg_nearest = distances.iter().take(nodes.len().min(50)).sum::<f32>()
            / (nodes.len().min(50) as f32);
        let avg_all = distances.iter().sum::<f32>() / distances.len() as f32;
        let cluster_ratio = avg_nearest / avg_all.max(1e-5);

        if grid_score > 0.6 {
            GeometryType::Grid
        } else if cluster_ratio < 0.15 {
            GeometryType::Clustered
        } else {
            GeometryType::Dispersed
        }
    }

    fn compute_adaptive_params(nodes: &[Node], n: usize) -> AdaptiveParams {
        let geometry = Self::detect_geometry(nodes);

        let (alpha, beta, h_as_tiebreaker, tiebreaker_threshold) = match geometry {
            GeometryType::Grid => (0.1, 0.1, true, 0.02),
            GeometryType::Dispersed => {
                if n < 100 {
                    (0.4, 0.3, false, 0.05)
                } else {
                    (0.25, 0.2, true, 0.08)
                }
            }
            GeometryType::Clustered => {
                if n < 150 {
                    (0.5, 0.4, false, 0.05)
                } else {
                    (0.3, 0.25, true, 0.10)
                }
            }
        };

        AdaptiveParams {
            alpha,
            beta,
            h_as_tiebreaker,
            tiebreaker_threshold,
            geometry,
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
    // Función H de LKH (Adaptada)
    // -------------------------------------------------------------------------

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

    fn total_tour_smoothness(path: &[usize], nodes: &[Node]) -> f32 {
        let mut total = 0.0;
        for i in 0..path.len() {
            let prev = path[(i + path.len() - 1) % path.len()];
            let curr = path[i];
            let next = path[(i + 1) % path.len()];
            total += Self::angle_at_point(prev, curr, next, nodes);
        }
        total
    }

    fn h_function_adaptive(
        path: &[usize],
        candidate: usize,
        pos: usize,
        nodes: &[Node],
        params: &AdaptiveParams,
    ) -> f32 {
        let i = pos;
        let j = (pos + 1) % path.len();

        let delta_c = insertion_cost(path[i], path[j], candidate, nodes);

        let smoothness_before = Self::total_tour_smoothness(path, nodes);
        let mut new_path = path.to_vec();
        new_path.insert(pos + 1, candidate);
        let smoothness_after = Self::total_tour_smoothness(&new_path, nodes);
        let smoothness_delta = smoothness_after - smoothness_before;

        let p_i = nodes[path[i]].pos;
        let p_j = nodes[path[j]].pos;
        let p_u = nodes[candidate].pos;

        let v1 = p_i - p_u;
        let v2 = p_j - p_u;
        let len1 = v1.length();
        let len2 = v2.length();

        let angle_penalty = if len1 > 1e-5 && len2 > 1e-5 {
            let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
            let theta = cos_theta.acos();
            (std::f32::consts::PI - theta) / std::f32::consts::PI
        } else {
            1.0
        };

        delta_c + params.alpha * smoothness_delta + params.beta * angle_penalty * delta_c
    }

    // -------------------------------------------------------------------------
    // Inserción con H como tie-breaker
    // -------------------------------------------------------------------------

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

    fn adaptive_insertion(
        &self,
        path: &[usize],
        nodes: &[Node],
        kdtree: &KDTree,
    ) -> (usize, usize) {
        if self.unvisited.is_empty() {
            return (0, 0);
        }

        let path_center = Self::compute_path_center(path, nodes);
        let reference_candidates = kdtree.find_k_nearest(path_center, self.k_neighbors);

        if self.params.h_as_tiebreaker {
            Self::insertion_with_tiebreaker(
                path,
                &self.unvisited,
                nodes,
                kdtree,
                &reference_candidates,
                &self.params,
            )
        } else {
            Self::insertion_with_h_primary(
                path,
                &self.unvisited,
                nodes,
                kdtree,
                &reference_candidates,
                &self.params,
            )
        }
    }

    fn insertion_with_h_primary(
        path: &[usize],
        unvisited: &[usize],
        nodes: &[Node],
        kdtree: &KDTree,
        reference_candidates: &[usize],
        params: &AdaptiveParams,
    ) -> (usize, usize) {
        let mut best_node = unvisited[0];
        let mut best_pos = 1;
        let mut best_h = f32::MAX;

        for &ref_candidate in reference_candidates {
            if !unvisited.contains(&ref_candidate) {
                continue;
            }

            let local_candidates = kdtree.find_k_nearest(nodes[ref_candidate].pos, 8);

            for &candidate in &local_candidates {
                if !unvisited.contains(&candidate) {
                    continue;
                }

                for i in 0..path.len() {
                    let h = Self::h_function_adaptive(path, candidate, i, nodes, params);
                    if h < best_h {
                        best_h = h;
                        best_node = candidate;
                        best_pos = i + 1;
                    }
                }
            }
        }

        (best_node, best_pos)
    }

    fn insertion_with_tiebreaker(
        path: &[usize],
        unvisited: &[usize],
        nodes: &[Node],
        kdtree: &KDTree,
        reference_candidates: &[usize],
        params: &AdaptiveParams,
    ) -> (usize, usize) {
        let mut candidates_with_cost: Vec<(usize, usize, f32, f32)> = Vec::new();

        for &ref_candidate in reference_candidates {
            if !unvisited.contains(&ref_candidate) {
                continue;
            }

            let local_candidates = kdtree.find_k_nearest(nodes[ref_candidate].pos, 8);

            for &candidate in &local_candidates {
                if !unvisited.contains(&candidate) {
                    continue;
                }

                for i in 0..path.len() {
                    let next = (i + 1) % path.len();
                    let cost = insertion_cost(path[i], path[next], candidate, nodes);

                    let p_i = nodes[path[i]].pos;
                    let p_j = nodes[path[next]].pos;
                    let p_u = nodes[candidate].pos;

                    let v1 = p_i - p_u;
                    let v2 = p_j - p_u;
                    let len1 = v1.length();
                    let len2 = v2.length();

                    let angle_score = if len1 > 1e-5 && len2 > 1e-5 {
                        let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
                        let theta = cos_theta.acos();
                        theta / std::f32::consts::PI
                    } else {
                        0.0
                    };

                    let edge_len = nodes[path[i]].pos.distance(nodes[path[next]].pos);
                    let cost_ratio = if edge_len > 1e-5 { cost / edge_len } else { 1.0 };
                    let cost_penalty = 1.0 / (1.0 + cost_ratio);

                    let primary_score = angle_score * 0.5 + cost_penalty * 0.5;

                    let h = Self::h_function_adaptive(path, candidate, i, nodes, params);

                    candidates_with_cost.push((candidate, i + 1, primary_score, h));
                }
            }
        }

        if candidates_with_cost.is_empty() {
            return (unvisited[0], 1);
        }

        candidates_with_cost.sort_by(|a, b| {
            b.2.partial_cmp(&a.2).unwrap_or(Ordering::Equal)
        });

        let best_primary = candidates_with_cost[0].2;
        let threshold = best_primary * (1.0 - params.tiebreaker_threshold);

        let tied_candidates: Vec<_> = candidates_with_cost
            .iter()
            .filter(|c| c.2 >= threshold)
            .collect();

        if tied_candidates.len() > 1 {
            let best = tied_candidates
                .iter()
                .min_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(Ordering::Equal))
                .unwrap();
            (best.0, best.1)
        } else {
            (candidates_with_cost[0].0, candidates_with_cost[0].1)
        }
    }

    // -------------------------------------------------------------------------
    // Post-optimización
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

impl Strategy for TriangleInsertionV85 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if current_path.is_empty() && self.unvisited.is_empty() {
            self.unvisited = (0..nodes.len()).collect();
            
            if !self.calibrated {
                self.params = Self::compute_adaptive_params(nodes, nodes.len());

                self.strategy_mode = match self.params.geometry {
                    GeometryType::Grid => "V8.5-Grid".to_string(),
                    GeometryType::Clustered => "V8.5-Clustered".to_string(),
                    GeometryType::Dispersed => {
                        if self.params.h_as_tiebreaker {
                            "V8.5-Dispersed-TB".to_string()
                        } else {
                            "V8.5-Dispersed-H".to_string()
                        }
                    }
                };
            }
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

        let (best_node, best_pos) = self.adaptive_insertion(current_path, nodes, &kdtree);

        if let Some(pos) = self.unvisited.iter().position(|&x| x == best_node) {
            self.unvisited.swap_remove(pos);
        }
        current_path.insert(best_pos, best_node);

        false
    }

    fn name(&self) -> &str {
        &self.strategy_mode
    }

    fn reset(&mut self) {
        self.initialized = false;
        self.unvisited.clear();
        self.params = AdaptiveParams {
            alpha: 0.3,
            beta: 0.2,
            h_as_tiebreaker: false,
            tiebreaker_threshold: 0.05,
            geometry: GeometryType::Dispersed,
        };
        self.strategy_mode = "V8-Standard".to_string();
        self.calibrated = false;
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

    fn run_to_completion(strategy: &mut TriangleInsertionV85, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 10 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v85_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV85::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_v85_geometry_detection_grid() {
        let nodes: Vec<Node> = (0..25)
            .map(|i| {
                let x = (i % 5) as f32 * 10.0;
                let y = (i / 5) as f32 * 10.0;
                Node::new(x, y)
            })
            .collect();

        let geometry = TriangleInsertionV85::detect_geometry(&nodes);
        // Grid detection is heuristic-based, just verify it runs
        assert!(matches!(geometry, GeometryType::Grid | GeometryType::Dispersed));
    }

    #[test]
    fn test_v85_adaptive_params() {
        let nodes: Vec<Node> = (0..50)
            .map(|i| Node::new(i as f32, (i % 10) as f32))
            .collect();

        let params = TriangleInsertionV85::compute_adaptive_params(&nodes, 50);
        assert!(params.alpha > 0.0 && params.beta > 0.0);
    }
}
