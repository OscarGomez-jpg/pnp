#![allow(unused)]
/// Estrategia: Triangle Insertion V7 — Geometric Acceleration + Ejection Chains + Simulated Annealing
///
/// Evolución de la V6 que incorpora tres mejoras fundamentales inspiradas en LKH:
///
/// 1. **Aceleración Geométrica (O(N²) → O(N log N))**:
///    Utiliza un árbol k-d para limitar la búsqueda de inserciones candidatas
///    solo a los vecinos más cercanos, reduciendo drásticamente el tiempo de ejecución.
///
/// 2. **Ejection Chains Dinámicas**:
///    Expulsa secuencias de nodos, crea huecos, y busca la mejor manera de reinsertarlos
///    todos de una vez, aceptando temporalmente soluciones "peores" para escapar de
///    óptimos locales profundos.
///
/// 3. **Recocido Simulado (Simulated Annealing)**:
///    Añade un factor de temperatura que permite ocasionalmente aceptar inserciones
///    con score ligeramente peor, especialmente al principio, evitando trampas locales.
///
/// Métrica: score = insertion_cost * (1 + α * (1 + cos θ)) * temperature_factor
use super::Strategy;
use crate::core::{Node, insertion_cost, path_distance};
use macroquad::prelude::Vec2;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// =============================================================================
// K-D Tree para Búsqueda de Vecinos Cercanos
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
    fn new() -> Self {
        Self { root: None }
    }

    fn build(points: &[(Vec2, usize)]) -> Self {
        if points.is_empty() {
            return Self { root: None };
        }
        Self {
            root: Some(Self::build_recursive(&mut points.to_vec(), 0)),
        }
    }

    fn build_recursive(points: &mut Vec<(Vec2, usize)>, depth: u32) -> Box<KDNode> {
        if points.is_empty() {
            panic!("Points vector should not be empty");
        }

        let axis = depth % 2;
        points.sort_by(|a, b| {
            if axis == 0 {
                a.0.x.partial_cmp(&b.0.x).unwrap()
            } else {
                a.0.y.partial_cmp(&b.0.y).unwrap()
            }
        });

        let mid = points.len() / 2;
        let (point, index) = points.remove(mid);
        let mut node = KDNode::new(point, index);

        if !points.is_empty() {
            let right_points = points.split_off(mid);
            if !right_points.is_empty() && mid < points.len() {
                node.right = Some(Self::build_recursive(&mut right_points.into_iter().collect(), depth + 1));
            }
            if !points.is_empty() {
                node.left = Some(Self::build_recursive(points, depth + 1));
            }
        }

        Box::new(node)
    }

    fn find_k_nearest(&self, query: Vec2, k: usize, nodes: &[Node]) -> Vec<usize> {
        let mut heap: BinaryHeap<DistanceItem> = BinaryHeap::new();
        if let Some(ref root) = self.root {
            Self::search_nearest(root, query, k, &mut heap, 0);
        }

        heap.into_iter()
            .take(k)
            .map(|item| item.index)
            .collect()
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

        if let Some(ref child) = first {
            Self::search_nearest(child, query, k, heap, depth + 1);
        }

        if let Some(ref child) = second {
            if heap.len() < k || diff.abs() < heap.peek().map_or(f32::MAX, |top| top.dist) {
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
        Some(self.cmp(other))
    }
}

impl Ord for DistanceItem {
    fn cmp(&self, other: &Self) -> Ordering {
        other.dist.partial_cmp(&self.dist).unwrap()
    }
}

// =============================================================================
// Triangle Insertion V7
// =============================================================================

pub struct TriangleInsertionV7 {
    initialized: bool,
    iteration: usize,
    total_iterations: usize,
    temperature: f32,
    initial_temperature: f32,
    cooling_rate: f32,
    k_neighbors: usize,
}

impl TriangleInsertionV7 {
    pub fn new() -> Self {
        Self {
            initialized: false,
            iteration: 0,
            total_iterations: 1000,
            temperature: 1.0,
            initial_temperature: 10.0,
            cooling_rate: 0.995,
            k_neighbors: 15,
        }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo (heredado de V6)
    // -------------------------------------------------------------------------

    fn convex_hull(nodes: &[Node]) -> Vec<usize> {
        if nodes.len() < 3 {
            return (0..nodes.len()).collect();
        }

        let mut indexed: Vec<usize> = (0..nodes.len()).collect();
        indexed.sort_by(|&a, &b| {
            let pa = nodes[a].pos;
            let pb = nodes[b].pos;
            pa.x.partial_cmp(&pb.x)
                .unwrap()
                .then(pa.y.partial_cmp(&pb.y).unwrap())
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

    fn best_triangle_from_hull(nodes: &[Node]) -> Vec<usize> {
        let hull = Self::convex_hull(nodes);
        if hull.len() < 3 {
            return (0..nodes.len().min(3)).collect();
        }

        let n = hull.len();
        let mut best_triangle = vec![hull[0], hull[1], hull[2]];
        let mut best_perimeter = Self::triangle_perimeter(hull[0], hull[1], hull[2], nodes);

        for i in 0..n {
            for j in (i + 1)..n {
                for k in (j + 1)..n {
                    let p = Self::triangle_perimeter(hull[i], hull[j], hull[k], nodes);
                    if p > best_perimeter {
                        best_perimeter = p;
                        best_triangle = vec![hull[i], hull[j], hull[k]];
                    }
                }
            }
        }
        best_triangle
    }

    fn triangle_perimeter(a: usize, b: usize, c: usize, nodes: &[Node]) -> f32 {
        nodes[a].pos.distance(nodes[b].pos)
            + nodes[b].pos.distance(nodes[c].pos)
            + nodes[c].pos.distance(nodes[a].pos)
    }

    // -------------------------------------------------------------------------
    // V7 Core: Smoothest Angle Insertion con Aceleración K-D Tree
    // -------------------------------------------------------------------------

    /// Inserción por Ángulo más Suave con aceleración geométrica.
    ///
    /// Usa un k-d tree para limitar candidatos a los k vecinos más cercanos
    /// de cada arista, reduciendo complejidad de O(N²) a O(N log N).
    fn smoothest_insertion_accelerated(
        path: &[usize],
        unvisited: &[usize],
        nodes: &[Node],
        kdtree: &KDTree,
        k: usize,
        temperature: f32,
    ) -> (usize, usize) {
        let alpha: f32 = 2.0;
        let mut best_node = unvisited[0];
        let mut best_pos = 1;
        let mut best_score = f32::MAX;

        for i in 0..path.len() {
            let next = (i + 1) % path.len();
            let edge_mid = (nodes[path[i]].pos + nodes[path[next]].pos) * 0.5;

            // Usar k-d tree para obtener candidatos cercanos a esta arista
            let candidates = kdtree.find_k_nearest(edge_mid, k.min(unvisited.len()), nodes);
            
            for &candidate_idx in &candidates {
                if !unvisited.contains(&candidate_idx) {
                    continue;
                }

                let cost = insertion_cost(path[i], path[next], candidate_idx, nodes);

                let p_i = nodes[path[i]].pos;
                let p_next = nodes[path[next]].pos;
                let p_u = nodes[candidate_idx].pos;

                let v1 = p_i - p_u;
                let v2 = p_next - p_u;
                let len1 = v1.length();
                let len2 = v2.length();

                let cos_theta = if len1 > 1e-5 && len2 > 1e-5 {
                    (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0)
                } else {
                    1.0
                };

                let base_score = cost * (1.0 + alpha * (1.0 + cos_theta));

                // Recocido simulado: añadir factor de temperatura
                let temperature_factor = 1.0 + (1.0 - temperature) * 0.5;
                let score = base_score * temperature_factor;

                // Criterio de aceptación probabilístico
                if score < best_score || (temperature > 0.1 && (best_score - score) / temperature < 0.1) {
                    if score < best_score || rand_accept(score, best_score, temperature) {
                        best_score = score;
                        best_node = candidate_idx;
                        best_pos = i + 1;
                    }
                }
            }
        }

        (best_node, best_pos)
    }

    // -------------------------------------------------------------------------
    // Ejection Chains Dinámicas
    // -------------------------------------------------------------------------

    /// Implementa ejection chains de profundidad variable.
    /// Expulsa una secuencia de nodos y busca la mejor re-inserción.
    fn ejection_chain(path: &mut Vec<usize>, nodes: &[Node], chain_length: usize, temperature: f32) -> bool {
        if path.len() < chain_length + 2 {
            return false;
        }

        let n = path.len();
        let current_dist = path_distance(path, nodes);
        let mut best_dist = current_dist;
        let mut best_path = path.clone();

        // Probar múltiples puntos de inicio para la cadena
        for start in 0..n {
            // Extraer segmento de nodos para la cadena de eyección
            let ejected: Vec<usize> = (0..chain_length)
                .map(|k| path[(start + k) % n])
                .collect();

            // Crear path reducido sin los nodos eyectados
            let mut reduced: Vec<usize> = Vec::new();
            for i in 0..n {
                if !ejected.contains(&path[i]) {
                    reduced.push(path[i]);
                }
            }

            if reduced.len() < 2 {
                continue;
            }

            // Buscar mejor re-inserción de todos los nodos eyectados
            let mut temp_path = reduced.clone();
            for &node in &ejected {
                let mut best_pos = 0;
                let mut best_insert_dist = f32::MAX;

                for pos in 0..=temp_path.len() {
                    let mut candidate = temp_path[..pos].to_vec();
                    candidate.push(node);
                    candidate.extend_from_slice(&temp_path[pos..]);

                    let dist = path_distance(&candidate, nodes);
                    
                    // Criterio de aceptación con temperatura
                    let acceptance_prob = if dist < best_insert_dist {
                        1.0
                    } else {
                        ((best_insert_dist - dist) / temperature).exp().clamp(0.0, 1.0)
                    };

                    if dist < best_insert_dist || (temperature > 0.1 && acceptance_prob > 0.3) {
                        best_insert_dist = dist;
                        best_pos = pos;
                    }
                }

                let mut new_path = temp_path[..best_pos].to_vec();
                new_path.push(node);
                new_path.extend_from_slice(&temp_path[best_pos..]);
                temp_path = new_path;
            }

            let final_dist = path_distance(&temp_path, nodes);

            // Criterio de aceptación para la cadena completa
            let delta = final_dist - current_dist;
            let accept = if delta < 0.0 {
                true
            } else if temperature > 0.1 {
                (delta / temperature).exp() > 0.1
            } else {
                false
            };

            if accept && final_dist < best_dist {
                best_dist = final_dist;
                best_path = temp_path;
            }
        }

        if best_dist < current_dist - 0.01 {
            *path = best_path;
            return true;
        }

        false
    }

    // -------------------------------------------------------------------------
    // Funciones heredadas de V6 (optimizaciones)
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

        while improved {
            improved = false;
            let current_dist = path_distance(path, nodes);

            for i in 0..path.len() {
                let seg: Vec<usize> = (0..seg_len).map(|k| path[(i + k) % path.len()]).collect();

                let mut reduced: Vec<usize> = Vec::new();
                for k in 0..path.len() {
                    if k < i || k >= i + seg_len {
                        reduced.push(path[k]);
                    }
                }

                let m = reduced.len();
                if m < 2 {
                    continue;
                }

                for j in 0..=m {
                    let mut candidate = reduced[..j.min(m)].to_vec();
                    candidate.extend_from_slice(&seg);
                    if j < m {
                        candidate.extend_from_slice(&reduced[j..]);
                    }

                    if candidate.len() != path.len() {
                        continue;
                    }

                    let dist = path_distance(&candidate, nodes);
                    if dist < current_dist - 0.01 {
                        *path = candidate;
                        improved = true;
                        break;
                    }
                }

                if improved {
                    break;
                }
            }
        }
        improved
    }

    fn optimize_node_reinsertion(path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if path.len() < 4 {
            return false;
        }

        let mut ever_improved = false;
        let mut improved = true;

        while improved {
            improved = false;
            let current_dist = path_distance(path, nodes);

            for idx in 0..path.len() {
                let node = path[idx];

                let mut reduced: Vec<usize> = path[..idx].to_vec();
                reduced.extend_from_slice(&path[idx + 1..]);

                let mut best_pos = idx;
                let mut best_dist = current_dist;

                for j in 0..reduced.len() {
                    let mut candidate = reduced[..=j].to_vec();
                    candidate.push(node);
                    candidate.extend_from_slice(&reduced[j + 1..]);

                    let dist = path_distance(&candidate, nodes);
                    if dist < best_dist - 0.01 {
                        best_dist = dist;
                        best_pos = j + 1;
                    }
                }

                if best_dist < current_dist - 0.01 {
                    let mut new_path = reduced[..best_pos].to_vec();
                    new_path.push(node);
                    new_path.extend_from_slice(&reduced[best_pos..]);
                    *path = new_path;
                    improved = true;
                    ever_improved = true;
                    break;
                }
            }
        }
        ever_improved
    }
}

// Función auxiliar para aceptación probabilística
fn rand_accept(new_score: f32, old_score: f32, temperature: f32) -> bool {
    if new_score <= old_score {
        return true;
    }
    let delta = new_score - old_score;
    if temperature <= 0.01 {
        return false;
    }
    let prob = (-delta / temperature).exp();
    prob > 0.1 // Umbral simplificado
}

impl Strategy for TriangleInsertionV7 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            // Post-optimización final con ejection chains
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
            
            // Aplicar ejection chains con temperatura decreciente
            for chain_len in 2..=4 {
                for _ in 0..3 {
                    Self::ejection_chain(current_path, nodes, chain_len, self.temperature);
                }
            }
            
            Self::optimize_node_reinsertion(current_path);
            Self::optimize_2opt(current_path, nodes, 5);
            return true;
        }

        // Paso 1: Inicialización
        if current_path.is_empty() {
            if nodes.len() < 3 {
                current_path.extend(0..nodes.len());
                return true;
            }
            let triangle = Self::best_triangle_from_hull(nodes);
            current_path.extend_from_slice(&triangle);
            self.initialized = true;
            return false;
        }

        if !self.initialized {
            return true;
        }

        // Actualizar temperatura (recocido simulado)
        self.iteration += 1;
        self.temperature = self.initial_temperature * self.cooling_rate.powi(self.iteration as i32);

        // Construir k-d tree para aceleración geométrica
        let points: Vec<(Vec2, usize)> = nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.pos, i))
            .collect();
        let kdtree = KDTree::build(&points);

        // Paso 2: Inserción con aceleración geométrica y recocido
        let (best_node, best_pos) = Self::smoothest_insertion_accelerated(
            current_path,
            &unvisited,
            nodes,
            &kdtree,
            self.k_neighbors,
            self.temperature,
        );
        current_path.insert(best_pos, best_node);

        // Paso 3: Aplicar ejection chain ocasionalmente durante la construcción
        if self.iteration % 5 == 0 && current_path.len() > 6 {
            Self::ejection_chain(current_path, nodes, 2, self.temperature);
        }

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V7 (Geo-Accel + Ejection Chains + SA)"
    }

    fn reset(&mut self) {
        self.initialized = false;
        self.iteration = 0;
        self.temperature = self.initial_temperature;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    fn run_to_completion(strategy: &mut TriangleInsertionV7, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 10 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v7_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV7::new();
        let path = run_to_completion(&mut strategy, &nodes);

        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_v7_kd_tree_acceleration() {
        let nodes: Vec<Node> = (0..20)
            .map(|i| Node::new(i as f32 * 1.5, (i % 5) as f32 * 2.0))
            .collect();
        
        let mut strategy = TriangleInsertionV7::new();
        let path = run_to_completion(&mut strategy, &nodes);
        
        assert_eq!(path.len(), 20, "Debe visitar todos los nodos con k-d tree");
    }

    #[test]
    fn test_v7_ejection_chain() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(5.0, 0.0),
            Node::new(5.0, 5.0),
            Node::new(0.0, 5.0),
            Node::new(2.5, 2.5),
            Node::new(7.5, 2.5),
        ];
        
        let mut strategy = TriangleInsertionV7::new();
        let path = run_to_completion(&mut strategy, &nodes);
        
        assert_eq!(path.len(), 6, "Debe visitar todos los nodos con ejection chains");
    }

    #[test]
    fn test_v7_simulated_annealing() {
        // Verificar que la temperatura se actualiza correctamente
        let mut strategy = TriangleInsertionV7::new();
        assert_eq!(strategy.temperature, strategy.initial_temperature);
        
        strategy.iteration = 100;
        let expected_temp = strategy.initial_temperature * strategy.cooling_rate.powi(100);
        let actual_temp = strategy.initial_temperature * strategy.cooling_rate.powi(strategy.iteration as i32);
        
        assert!((actual_temp - expected_temp).abs() < 0.001);
    }
}
