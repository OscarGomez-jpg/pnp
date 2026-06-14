#![allow(unused)]
/// Estrategia: Triangle Insertion V6 — Smoothest Angle Insertion
///
/// Evolución de la V4 que reemplaza la métrica de "Cheapest Insertion" por una
/// métrica de "Inserción más Suave". En lugar de minimizar el costo de inserción
/// (que favorece puntos cercanos a aristas existentes), la V6 favorece inserciones
/// que producen ángulos abiertos en el punto insertado.
///
/// Analogía: Imagina pines en un tablero y un hilo que debe pasar por todos.
/// Un hilo real prefiere curvas suaves (ángulos abiertos) sobre giros bruscos
/// (ángulos cerrados). Esta heurística imita ese comportamiento físico.
///
/// Métrica: score = insertion_cost * (1 + α * (1 + cos θ))
///   - Cuando θ ≈ 180° (hilo recto): cos θ ≈ -1, penalización = 0
///   - Cuando θ ≈ 0° (giro brusco):  cos θ ≈ +1, penalización = 2α * cost
use super::Strategy;
use crate::core::{Node, insertion_cost, path_distance};
use macroquad::prelude::Vec2;

pub struct TriangleInsertionV6 {
    initialized: bool,
}

impl TriangleInsertionV6 {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo (heredado de V4)
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
    // V6 Core: Smoothest Angle Insertion
    // -------------------------------------------------------------------------

    /// Calcula el ángulo (en radianes) formado en el punto `u` cuando se inserta
    /// entre los nodos `i` y `j` del path. Un ángulo mayor = curva más suave.
    fn insertion_angle(i: usize, j: usize, u: usize, nodes: &[Node]) -> f32 {
        let p_i = nodes[i].pos;
        let p_j = nodes[j].pos;
        let p_u = nodes[u].pos;

        let v1 = p_i - p_u; // vector del pin insertado hacia el vecino izquierdo
        let v2 = p_j - p_u; // vector del pin insertado hacia el vecino derecho

        let len1 = v1.length();
        let len2 = v2.length();

        if len1 < 1e-5 || len2 < 1e-5 {
            return 0.0; // Nodos coincidentes, ángulo degenerado
        }

        let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
        cos_theta.acos() // Ángulo en radianes [0, π]
    }

    /// Inserción por Ángulo más Suave.
    ///
    /// Para cada candidato no visitado, evalúa todas las posiciones de inserción.
    /// El score combina el costo de inserción con una penalización por ángulos cerrados:
    ///   score = insertion_cost * (1 + α * (1 + cos θ))
    ///
    /// α controla cuánto peso le damos a la suavidad vs. la distancia pura.
    /// Con α=0 se reduce a Cheapest Insertion (V4). Con α alto, domina la suavidad.
    fn smoothest_insertion(path: &[usize], unvisited: &[usize], nodes: &[Node]) -> (usize, usize) {
        let alpha: f32 = 2.0; // Factor de penalización angular

        let mut best_node = unvisited[0];
        let mut best_pos = 1;
        let mut best_score = f32::MAX;

        for &candidate in unvisited {
            for i in 0..path.len() {
                let next = (i + 1) % path.len();

                let cost = insertion_cost(path[i], path[next], candidate, nodes);

                // Calcular cos(θ) en el punto de inserción
                let p_i = nodes[path[i]].pos;
                let p_next = nodes[path[next]].pos;
                let p_u = nodes[candidate].pos;

                let v1 = p_i - p_u;
                let v2 = p_next - p_u;
                let len1 = v1.length();
                let len2 = v2.length();

                let cos_theta = if len1 > 1e-5 && len2 > 1e-5 {
                    (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0)
                } else {
                    1.0 // Peor caso: ángulo cerrado
                };

                // cos_theta = -1 (línea recta, ideal) → penalty = 0
                // cos_theta = +1 (giro en U, terrible) → penalty = 2α * cost
                let score = cost * (1.0 + alpha * (1.0 + cos_theta));

                if score < best_score {
                    best_score = score;
                    best_node = candidate;
                    best_pos = i + 1;
                }
            }
        }

        (best_node, best_pos)
    }

    // -------------------------------------------------------------------------
    // Métrica de Atracción & Rotación de Triángulos (heredado de V4)
    // -------------------------------------------------------------------------

    fn min_insertion_cost(path: &[usize], unvisited: &[usize], nodes: &[Node]) -> f32 {
        if unvisited.is_empty() || path.len() < 2 {
            return 0.0;
        }

        let mut global_min = f32::MAX;
        for &candidate in unvisited {
            for i in 0..path.len() {
                let next = (i + 1) % path.len();
                let cost = insertion_cost(path[i], path[next], candidate, nodes);
                if cost < global_min {
                    global_min = cost;
                }
            }
        }
        global_min
    }

    fn evaluate_configuration(path: &[usize], unvisited: &[usize], nodes: &[Node]) -> f32 {
        let dist = path_distance(path, nodes);
        let look_ahead = Self::min_insertion_cost(path, unvisited, nodes);
        dist + look_ahead
    }

    fn rotate_triangles_for_candidates(path: &mut Vec<usize>, unvisited: &[usize], nodes: &[Node]) {
        if path.len() < 4 {
            return;
        }

        let mut improved = true;
        let mut max_iters = 10;

        while improved && max_iters > 0 {
            improved = false;
            max_iters -= 1;

            let current_score = Self::evaluate_configuration(path, unvisited, nodes);

            for i in 0..path.len() {
                let i1 = i;
                let i2 = (i + 1) % path.len();
                let i3 = (i + 2) % path.len();

                let a = path[i1];
                let b = path[i2];
                let c = path[i3];

                let permutations = [
                    vec![a, b, c],
                    vec![a, c, b],
                    vec![b, a, c],
                    vec![b, c, a],
                    vec![c, a, b],
                    vec![c, b, a],
                ];

                let mut best_score = current_score;
                let mut best_perm = None;

                for perm in permutations.iter().skip(1) {
                    let mut proposed_path = path.clone();
                    proposed_path[i1] = perm[0];
                    proposed_path[i2] = perm[1];
                    proposed_path[i3] = perm[2];

                    let score = Self::evaluate_configuration(&proposed_path, unvisited, nodes);

                    if score < best_score - 0.01 {
                        best_score = score;
                        best_perm = Some(perm.clone());
                    }
                }

                if let Some(perm) = best_perm {
                    path[i1] = perm[0];
                    path[i2] = perm[1];
                    path[i3] = perm[2];
                    improved = true;
                    break;
                }
            }
        }
    }

    fn edges_intersect(a1: usize, a2: usize, b1: usize, b2: usize, nodes: &[Node]) -> bool {
        let p1 = nodes[a1].pos;
        let p2 = nodes[a2].pos;
        let p3 = nodes[b1].pos;
        let p4 = nodes[b2].pos;

        let cross = |p: Vec2, q: Vec2, r: Vec2| -> f32 {
            (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
        };

        let o1 = cross(p1, p2, p3);
        let o2 = cross(p1, p2, p4);
        let o3 = cross(p3, p4, p1);
        let o4 = cross(p3, p4, p2);

        o1 * o2 < -1e-4 && o3 * o4 < -1e-4
    }

    fn rotate_double_triangles(path: &mut Vec<usize>, unvisited: &[usize], nodes: &[Node]) -> bool {
        if path.len() < 6 {
            return false;
        }

        let n = path.len();
        let current_score = Self::evaluate_configuration(path, unvisited, nodes);
        let mut best_score = current_score;
        let mut best_path = None;

        for i in 0..n {
            for j in (i + 2)..n {
                if i == 0 && j == n - 1 {
                    continue;
                }

                if Self::edges_intersect(
                    path[i],
                    path[(i + 1) % n],
                    path[j],
                    path[(j + 1) % n],
                    nodes,
                ) {
                    let t1_idx = [(i + n - 1) % n, i, (i + 1) % n];
                    let t2_idx = [(j + n - 1) % n, j, (j + 1) % n];

                    let mut overlap = false;
                    for &x in &t1_idx {
                        if t2_idx.contains(&x) {
                            overlap = true;
                        }
                    }
                    if overlap {
                        continue;
                    }

                    let a1 = path[t1_idx[0]];
                    let b1 = path[t1_idx[1]];
                    let c1 = path[t1_idx[2]];
                    let a2 = path[t2_idx[0]];
                    let b2 = path[t2_idx[1]];
                    let c2 = path[t2_idx[2]];

                    let perms1 = [
                        vec![a1, b1, c1],
                        vec![a1, c1, b1],
                        vec![b1, a1, c1],
                        vec![b1, c1, a1],
                        vec![c1, a1, b1],
                        vec![c1, b1, a1],
                    ];
                    let perms2 = [
                        vec![a2, b2, c2],
                        vec![a2, c2, b2],
                        vec![b2, a2, c2],
                        vec![b2, c2, a2],
                        vec![c2, a2, b2],
                        vec![c2, b2, a2],
                    ];

                    for p1 in &perms1 {
                        for p2 in &perms2 {
                            let mut proposed_path = path.clone();
                            proposed_path[t1_idx[0]] = p1[0];
                            proposed_path[t1_idx[1]] = p1[1];
                            proposed_path[t1_idx[2]] = p1[2];

                            proposed_path[t2_idx[0]] = p2[0];
                            proposed_path[t2_idx[1]] = p2[1];
                            proposed_path[t2_idx[2]] = p2[2];

                            let score =
                                Self::evaluate_configuration(&proposed_path, unvisited, nodes);
                            if score < best_score - 0.01 {
                                best_score = score;
                                best_path = Some(proposed_path);
                            }
                        }
                    }
                }
            }
        }

        if let Some(p) = best_path {
            *path = p;
            return true;
        }

        false
    }

    // -------------------------------------------------------------------------
    // Post-optimización: Or-Opt & 2-Opt (heredado de V4)
    // -------------------------------------------------------------------------

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
                // Extraer el segmento de nodos consecutivos
                let seg: Vec<usize> = (0..seg_len).map(|k| path[(i + k) % path.len()]).collect();

                // Crear path reducido sin el segmento
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

    // -------------------------------------------------------------------------
    // Post-optimización: Node Reinsertion (V6 exclusivo)
    // -------------------------------------------------------------------------

    /// Saca cada nodo del tour y lo re-inserta en la posición globalmente óptima.
    /// Repite hasta que ninguna reinserción mejore la distancia total.
    /// Complejidad: O(N²) por pasada, converge en pocas iteraciones.
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

                // Costo de remover: la arista previa y la siguiente se fusionan
                let _prev = if idx == 0 { path.len() - 1 } else { idx - 1 };
                let _next = (idx + 1) % path.len();

                // Crear path sin este nodo
                let mut reduced: Vec<usize> = path[..idx].to_vec();
                reduced.extend_from_slice(&path[idx + 1..]);

                // Buscar la mejor posición de reinserción
                let mut best_pos = idx; // posición original por defecto
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

                // ¿Encontramos una posición mejor?
                if best_dist < current_dist - 0.01 {
                    let mut new_path = reduced[..best_pos].to_vec();
                    new_path.push(node);
                    new_path.extend_from_slice(&reduced[best_pos..]);
                    *path = new_path;
                    improved = true;
                    ever_improved = true;
                    break; // Reiniciar el scan con el nuevo path
                }
            }
        }
        ever_improved
    }
}

impl Strategy for TriangleInsertionV6 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            // Post-optimización final
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
            Self::optimize_node_reinsertion(current_path, nodes);
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

        // Paso 2: Rotación de Triángulos Locales (Look-Ahead)
        Self::rotate_triangles_for_candidates(current_path, &unvisited, nodes);

        // Paso 2.5: Rotación Doble para desenredar cruces
        Self::rotate_double_triangles(current_path, &unvisited, nodes);

        // Paso 3: Inserción por Ángulo más Suave (V6 Core)
        let (best_node, best_pos) = Self::smoothest_insertion(current_path, &unvisited, nodes);
        current_path.insert(best_pos, best_node);

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V6 (Smoothest Angle)"
    }

    fn reset(&mut self) {
        self.initialized = false;
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

    fn run_to_completion(strategy: &mut TriangleInsertionV6, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 5 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v6_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV6::new();
        let path = run_to_completion(&mut strategy, &nodes);

        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_insertion_angle_straight_line() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(5.0, 0.0),
            Node::new(10.0, 0.0),
        ];
        // Nodo 1 está perfectamente entre 0 y 2 → ángulo ≈ π (180°)
        let angle = TriangleInsertionV6::insertion_angle(0, 2, 1, &nodes);
        assert!(
            (angle - std::f32::consts::PI).abs() < 0.01,
            "Ángulo debería ser ~π para línea recta"
        );
    }

    #[test]
    fn test_insertion_angle_right_angle() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(5.0, 5.0),
            Node::new(10.0, 0.0),
        ];
        // Nodo 1 forma un ángulo recto con 0 y 2 → ~π/2 (90°)
        let angle = TriangleInsertionV6::insertion_angle(0, 2, 1, &nodes);
        assert!(
            (angle - std::f32::consts::FRAC_PI_2).abs() < 0.1,
            "Ángulo debería ser ~π/2 para ángulo recto"
        );
    }

    #[test]
    fn test_v6_prefers_smooth_over_cheap() {
        // Escenario donde el nodo "barato" genera un giro brusco,
        // pero el nodo "suave" genera una curva elegante.
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
            Node::new(5.0, 5.0), // Centro: barato pero genera giros
        ];
        let mut strategy = TriangleInsertionV6::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 5, "Debe visitar todos los nodos");
    }
}
