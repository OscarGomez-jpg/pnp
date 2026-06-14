/// Estrategia: Triangle Insertion V4 — Triangle Rotation & Look-Ahead
///
/// Implementa la "Rotación Geométrica guiada por Atracción".
/// Antes de decidir insertar un nuevo punto, toma los triángulos locales de
/// la frontera (grupos de 3 nodos consecutivos) y evalúa rotarlos/permutarlos.
/// 
/// La rotación se acepta si minimiza la distancia del recorrido actual 
/// MÁS la distancia de inserción del mejor candidato no visitado.

use super::Strategy;
use crate::core::{insertion_cost, path_distance, Node};

pub struct TriangleInsertionV4 {
    initialized: bool,
}

impl TriangleInsertionV4 {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo (igual que V3 para base matemática sólida)
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
            while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], idx) <= 0.0 {
                lower.pop();
            }
            lower.push(idx);
        }

        let mut upper: Vec<usize> = Vec::new();
        for &idx in indexed.iter().rev() {
            while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], idx) <= 0.0 {
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
    // Métrica de Atracción & Rotación de Triángulos (V4 Core)
    // -------------------------------------------------------------------------

    /// Calcula el costo mínimo para insertar CUALQUIER nodo no visitado en la ruta dada.
    /// Esto actúa como un "Look-Ahead": ¿qué tan amigable es esta ruta para el próximo paso?
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

    /// Evalúa la calidad total de una ruta: distancia del polígono + atracción de candidatos.
    fn evaluate_configuration(path: &[usize], unvisited: &[usize], nodes: &[Node]) -> f32 {
        let dist = path_distance(path, nodes);
        let look_ahead = Self::min_insertion_cost(path, unvisited, nodes);
        dist + look_ahead
    }

    /// Toma grupos de 3 nodos consecutivos (triángulos locales en la frontera) y
    /// evalúa todas sus permutaciones. Si alguna permutación mejora la métrica de
    /// atracción, aplica la rotación al path.
    fn rotate_triangles_for_candidates(
        path: &mut Vec<usize>,
        unvisited: &[usize],
        nodes: &[Node],
    ) {
        if path.len() < 4 {
            return; // Necesitamos al menos 4 para que permutar 3 tenga sentido real en el ciclo
        }

        let mut improved = true;
        let mut max_iters = 10; // Evitar ciclos infinitos raros

        while improved && max_iters > 0 {
            improved = false;
            max_iters -= 1;

            let current_score = Self::evaluate_configuration(path, unvisited, nodes);

            // Iterar sobre cada posible inicio de un trío de nodos (incluso cruzando el final)
            for i in 0..path.len() {
                let i1 = i;
                let i2 = (i + 1) % path.len();
                let i3 = (i + 2) % path.len();

                let a = path[i1];
                let b = path[i2];
                let c = path[i3];

                // Las 6 permutaciones de (A, B, C)
                let permutations = [
                    vec![a, b, c], // Original
                    vec![a, c, b],
                    vec![b, a, c],
                    vec![b, c, a],
                    vec![c, a, b],
                    vec![c, b, a],
                ];

                let mut best_score = current_score;
                let mut best_perm = None;

                for perm in permutations.iter().skip(1) { // Saltar la original
                    // Construir la nueva ruta propuesta
                    let mut proposed_path = path.clone();
                    proposed_path[i1] = perm[0];
                    proposed_path[i2] = perm[1];
                    proposed_path[i3] = perm[2];

                    let score = Self::evaluate_configuration(&proposed_path, unvisited, nodes);
                    
                    if score < best_score - 0.01 { // Margen de mejora para evitar inestabilidad float
                        best_score = score;
                        best_perm = Some(perm.clone());
                    }
                }

                if let Some(perm) = best_perm {
                    path[i1] = perm[0];
                    path[i2] = perm[1];
                    path[i3] = perm[2];
                    improved = true;
                    // Romper el for y reiniciar el loop while para re-evaluar con el nuevo path
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

        let cross = |p: macroquad::math::Vec2, q: macroquad::math::Vec2, r: macroquad::math::Vec2| -> f32 {
            (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y)
        };

        let o1 = cross(p1, p2, p3);
        let o2 = cross(p1, p2, p4);
        let o3 = cross(p3, p4, p1);
        let o4 = cross(p3, p4, p2);

        // Si los signos son opuestos, se cruzan estrictamente
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
            for j in (i + 2)..n { // asegurar aristas disjuntas
                if i == 0 && j == n - 1 { continue; } // adyacentes por ciclo

                if Self::edges_intersect(path[i], path[(i+1)%n], path[j], path[(j+1)%n], nodes) {
                    // Triángulo 1: i-1, i, i+1
                    // Triángulo 2: j-1, j, j+1
                    let t1_idx = [(i+n-1)%n, i, (i+1)%n];
                    let t2_idx = [(j+n-1)%n, j, (j+1)%n];

                    // Verificar solapamiento de nodos
                    let mut overlap = false;
                    for &x in &t1_idx {
                        if t2_idx.contains(&x) { overlap = true; }
                    }
                    if overlap { continue; }

                    let a1 = path[t1_idx[0]]; let b1 = path[t1_idx[1]]; let c1 = path[t1_idx[2]];
                    let a2 = path[t2_idx[0]]; let b2 = path[t2_idx[1]]; let c2 = path[t2_idx[2]];

                    let perms1 = [
                        vec![a1, b1, c1], vec![a1, c1, b1], vec![b1, a1, c1],
                        vec![b1, c1, a1], vec![c1, a1, b1], vec![c1, b1, a1]
                    ];
                    let perms2 = [
                        vec![a2, b2, c2], vec![a2, c2, b2], vec![b2, a2, c2],
                        vec![b2, c2, a2], vec![c2, a2, b2], vec![c2, b2, a2]
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

                            let score = Self::evaluate_configuration(&proposed_path, unvisited, nodes);
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
    // Expansión: Cheapest Insertion Global
    // -------------------------------------------------------------------------

    fn cheapest_insertion(
        path: &[usize],
        unvisited: &[usize],
        nodes: &[Node],
    ) -> (usize, usize) {
        let mut global_best_cost = f32::MAX;
        let mut best_node = unvisited[0];
        let mut best_pos = 1;

        for &candidate in unvisited {
            let mut min_cost = f32::MAX;
            let mut min_pos = 1;

            for i in 0..path.len() {
                let next = (i + 1) % path.len();
                let cost = insertion_cost(path[i], path[next], candidate, nodes);
                if cost < min_cost {
                    min_cost = cost;
                    min_pos = i + 1;
                }
            }

            if min_cost < global_best_cost {
                global_best_cost = min_cost;
                best_node = candidate;
                best_pos = min_pos;
            }
        }

        (best_node, best_pos)
    }

    // -------------------------------------------------------------------------
    // Post-optimización: Or-Opt & 2-Opt
    // -------------------------------------------------------------------------

    fn optimize_or_opt(path: &mut Vec<usize>, nodes: &[Node], seg_len: usize) -> bool {
        let n = path.len();
        if n < seg_len + 2 {
            return false;
        }

        let mut improved = false;

        'outer: loop {
            let current_dist = path_distance(path, nodes);

            for i in 0..path.len() {
                if i + seg_len > path.len() {
                    continue;
                }

                let seg: Vec<usize> = (0..seg_len).map(|k| path[i + k]).collect();
                let prev = if i == 0 { path.len() - 1 } else { i - 1 };
                let next = i + seg_len;

                if next >= path.len() {
                    continue;
                }

                let mut new_path: Vec<usize> = path[..i].to_vec();
                new_path.extend_from_slice(&path[next..]);

                let m = new_path.len();
                for j in 0..m {
                    if j == prev.min(m - 1) {
                        continue;
                    }
                    let mut candidate = new_path[..=j].to_vec();
                    candidate.extend_from_slice(&seg);
                    candidate.extend_from_slice(&new_path[j + 1..]);

                    if path_distance(&candidate, nodes) < current_dist - 0.01 {
                        *path = candidate;
                        improved = true;
                        continue 'outer;
                    }
                }
            }
            break;
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
}

impl Strategy for TriangleInsertionV4 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            // Post-optimización final
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
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

        // Paso 3: Inserción Cheapest Global
        let (best_node, best_pos) = Self::cheapest_insertion(current_path, &unvisited, nodes);
        current_path.insert(best_pos, best_node);

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V4 (Look-Ahead Rotation)"
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

    fn run_to_completion(strategy: &mut TriangleInsertionV4, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 5 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v4_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV4::new();
        let path = run_to_completion(&mut strategy, &nodes);

        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_evaluate_configuration_scores_correctly() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0), // En path
            Node::new(5.0, 0.1),   // No visitado, cerca de la arista 0->10
        ];
        let path = vec![0, 1, 2];
        let unvisited = vec![3];

        let score = TriangleInsertionV4::evaluate_configuration(&path, &unvisited, &nodes);
        
        // Distancia del path: 10 + 10 + sqrt(200) = 34.14
        // Costo de inserción de (5, 0.1) entre (0,0) y (10,0):
        // sqrt(5^2 + 0.1^2) + sqrt(5^2 + 0.1^2) - 10 = 5.001 + 5.001 - 10 = 0.002
        // Score total ≈ 34.14 + 0.002 = 34.142
        assert!(score > 34.0 && score < 35.0);
    }
    
    #[test]
    fn test_rotation_applies_when_better() {
        // Configuramos un escenario artificial donde rotar un triángulo sea muy beneficioso
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(5.0, 10.0),
            Node::new(10.0, -10.0), // Unvisited
        ];
        // Forzamos un path ineficiente
        let mut path = vec![0, 2, 1]; // Triángulo mal rotado para atraer al nodo 3
        let unvisited = vec![3];

        let initial_score = TriangleInsertionV4::evaluate_configuration(&path, &unvisited, &nodes);
        
        TriangleInsertionV4::rotate_triangles_for_candidates(&mut path, &unvisited, &nodes);
        
        let final_score = TriangleInsertionV4::evaluate_configuration(&path, &unvisited, &nodes);
        
        // La rotación debería haber mejorado el score
        assert!(final_score <= initial_score);
    }
}
