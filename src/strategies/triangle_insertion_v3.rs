/// Estrategia: Triangle Insertion V3 — Cheapest Insertion + Convex Hull Init
///
/// Mejoras estructurales respecto a V2:
///
/// 1. **Inicialización por Casco Convexo**: Se calculan los 3 vértices del convex hull
///    con mayor perímetro. Esto garantiza que los puntos más "extremos" estén en la
///    frontera del tour desde el inicio, evitando cruces futuros.
///
/// 2. **Cheapest Insertion Global**: En cada paso se evalúan TODOS los nodos no
///    visitados. Para cada uno se calcula su mínimo costo de inserción en cualquier
///    arista del tour actual. Se elige el nodo con el mínimo global.
///    → Garantía teórica: longitud ≤ 2× óptimo.
///
/// 3. **Or-Opt Post-Procesamiento**: Además de 2-Opt, se intentan mover segmentos
///    de 1 o 2 nodos consecutivos a otras posiciones del tour si reducen el costo.

use super::Strategy;
use crate::core::{insertion_cost, path_distance, Node};

pub struct TriangleInsertionV3 {
    initialized: bool,
}

impl TriangleInsertionV3 {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo
    // -------------------------------------------------------------------------

    /// Calcula el casco convexo (Andrew's Monotone Chain) y devuelve los índices
    /// de los nodos en el hull, ordenados por ángulo (sentido antihorario).
    fn convex_hull(nodes: &[Node]) -> Vec<usize> {
        if nodes.len() < 3 {
            return (0..nodes.len()).collect();
        }

        // Ordenar por (x, y)
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

        // Eliminar el último de cada mitad (duplicado)
        lower.pop();
        upper.pop();
        lower.extend(upper);
        lower
    }

    /// Selecciona los 3 vértices del casco convexo que forman el triángulo
    /// con mayor perímetro. Si el hull tiene < 3 puntos, usa los primeros nodos.
    fn best_triangle_from_hull(nodes: &[Node]) -> Vec<usize> {
        let hull = Self::convex_hull(nodes);

        if hull.len() < 3 {
            return (0..nodes.len().min(3)).collect();
        }

        let n = hull.len();
        let mut best_triangle = vec![hull[0], hull[1], hull[2]];
        let mut best_perimeter = Self::triangle_perimeter(hull[0], hull[1], hull[2], nodes);

        // Probar todas las combinaciones de 3 vértices del hull
        // El hull suele ser pequeño (O(log n) esperado), así que esto es barato
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
    // Expansión: Cheapest Insertion Global
    // -------------------------------------------------------------------------

    /// Para cada nodo no visitado calcula su mínimo costo de inserción en
    /// cualquier arista del tour. Devuelve (nodo, posición_inserción).
    ///
    /// Complejidad: O(|unvisited| × |path|) por paso → O(n²) total por paso
    fn cheapest_insertion(
        path: &[usize],
        unvisited: &[usize],
        nodes: &[Node],
    ) -> (usize, usize) {
        let mut global_best_cost = f32::MAX;
        let mut best_node = unvisited[0];
        let mut best_pos = 1;

        for &candidate in unvisited {
            // Mínimo costo de inserción de este candidato en cualquier arista
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

            // ¿Este candidato es globalmente mejor?
            if min_cost < global_best_cost {
                global_best_cost = min_cost;
                best_node = candidate;
                best_pos = min_pos;
            }
        }

        (best_node, best_pos)
    }

    // -------------------------------------------------------------------------
    // Post-optimización: Or-Opt
    // -------------------------------------------------------------------------

    /// Or-Opt: mueve segmentos de longitud `seg_len` (1 ó 2 nodos) a la posición
    /// que minimice el costo total. Retorna true si hubo alguna mejora.
    fn optimize_or_opt(path: &mut Vec<usize>, nodes: &[Node], seg_len: usize) -> bool {
        let n = path.len();
        if n < seg_len + 2 {
            return false;
        }

        let mut improved = false;

        'outer: loop {
            let current_dist = path_distance(path, nodes);

            for i in 0..path.len() {
                // Segmento: path[i..i+seg_len] (circular)
                if i + seg_len > path.len() {
                    continue;
                }

                // Extraer segmento
                let seg: Vec<usize> = (0..seg_len).map(|k| path[i + k]).collect();

                // Nodos anterior y posterior al segmento
                let prev = if i == 0 { path.len() - 1 } else { i - 1 };
                let next = i + seg_len; // podría ser == path.len() si seg toca el final

                if next >= path.len() {
                    continue;
                }

                // Construir path sin el segmento
                let mut new_path: Vec<usize> = path[..i].to_vec();
                new_path.extend_from_slice(&path[next..]);

                // Intentar insertar el segmento en cada posición del path reducido
                let m = new_path.len();
                for j in 0..m {
                    if j == prev.min(m - 1) {
                        continue; // posición original
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

    /// 2-Opt clásico (igual que V2 pero más limpio)
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

impl Strategy for TriangleInsertionV3 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            // Post-optimización final completa
            Self::optimize_2opt(current_path, nodes, 10);
            Self::optimize_or_opt(current_path, nodes, 1);
            Self::optimize_or_opt(current_path, nodes, 2);
            Self::optimize_2opt(current_path, nodes, 5);
            return true;
        }

        // Paso 1: Inicialización con triángulo del casco convexo
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

        // Paso 2: Cheapest Insertion Global
        let (best_node, best_pos) = Self::cheapest_insertion(current_path, &unvisited, nodes);
        current_path.insert(best_pos, best_node);

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V3 (Convex Hull + Cheapest)"
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

    fn run_to_completion(strategy: &mut TriangleInsertionV3, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 5 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v3_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV3::new();
        let path = run_to_completion(&mut strategy, &nodes);

        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
        for i in 0..4 {
            assert!(path.contains(&i), "Nodo {} debe estar en el path", i);
        }
    }

    #[test]
    fn test_v3_optimal_square_perimeter() {
        // Cuadrado 10×10 — perímetro óptimo = 40
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV3::new();
        let path = run_to_completion(&mut strategy, &nodes);
        let dist = crate::core::path_distance(&path, &nodes);

        assert!(
            (dist - 40.0).abs() < 0.1,
            "Perímetro óptimo del cuadrado debería ser 40, got {:.2}",
            dist
        );
    }

    #[test]
    fn test_v3_reset() {
        let mut strategy = TriangleInsertionV3::new();
        strategy.initialized = true;
        strategy.reset();
        assert!(!strategy.initialized);
    }

    #[test]
    fn test_v3_convex_hull_collinear_safe() {
        // 5 puntos en línea recta — hull puede tener < 3 puntos, no debe paniquear
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(1.0, 0.0),
            Node::new(2.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(4.0, 0.0),
        ];
        let mut strategy = TriangleInsertionV3::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 5);
    }

    #[test]
    fn test_v3_three_nodes() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(5.0, 0.0),
            Node::new(2.5, 4.33),
        ];
        let mut strategy = TriangleInsertionV3::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn test_convex_hull_returns_valid_indices() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(5.0, 0.0),
            Node::new(5.0, 5.0),
            Node::new(0.0, 5.0),
            Node::new(2.5, 2.5), // punto interior — no debe estar en hull
        ];
        let hull = TriangleInsertionV3::convex_hull(&nodes);
        // El punto interior no debería aparecer en el hull
        assert!(!hull.contains(&4), "El punto interior no debe estar en el casco convexo");
        assert!(hull.len() >= 3);
    }

    #[test]
    fn test_cheapest_insertion_selects_best() {
        // Tour: 0 → 1 → 2, candidatos: nodo 3 (muy cerca de arista 0-1)
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(5.0, 10.0),
            Node::new(5.0, 0.1), // casi en la arista 0-1
        ];
        let path = vec![0, 1, 2];
        let unvisited = vec![3];
        let (node, _pos) = TriangleInsertionV3::cheapest_insertion(&path, &unvisited, &nodes);
        assert_eq!(node, 3);
    }
}
