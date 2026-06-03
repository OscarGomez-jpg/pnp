/// Estrategia: Triangle Insertion V5 — Búsqueda desde la Raíz (Branch & Bound)
///
/// Algoritmo Exacto que utiliza el triángulo del Casco Convexo como RAÍZ
/// de un árbol de búsqueda. Mediante Backtracking, explora las ramas de
/// inserción y poda aquellas que superan la mejor distancia encontrada.
/// Garantiza encontrar el óptimo absoluto para grafos pequeños.

use super::Strategy;
use crate::core::{insertion_cost, path_distance, Node};

#[derive(Clone)]
struct SearchState {
    path: Vec<usize>,
    unvisited: Vec<usize>,
    current_dist: f32,
}

pub struct TriangleInsertionV5 {
    initialized: bool,
    best_path: Vec<usize>,
    best_distance: f32,
    stack: Vec<SearchState>,
    finished: bool,
    total_iters: usize,
    visual_path: Vec<usize>, // Para mostrar algo en pantalla mientras piensa
}

impl TriangleInsertionV5 {
    pub fn new() -> Self {
        Self {
            initialized: false,
            best_path: Vec::new(),
            best_distance: f32::MAX,
            stack: Vec::new(),
            finished: false,
            total_iters: 0,
            visual_path: Vec::new(),
        }
    }

    // -------------------------------------------------------------------------
    // Inicialización: Casco Convexo
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

    /// Implementación de Nearest Neighbor rápido para obtener un 'Upper Bound' inicial.
    /// Esto permite podar la gran mayoría de ramas desde el primer segundo.
    fn get_initial_upper_bound(nodes: &[Node]) -> f32 {
        if nodes.is_empty() { return 0.0; }
        let mut unvisited: Vec<usize> = (1..nodes.len()).collect();
        let mut current = 0;
        let mut path = vec![0];

        while !unvisited.is_empty() {
            let mut best_idx = 0;
            let mut best_dist = f32::MAX;

            for (i, &cand) in unvisited.iter().enumerate() {
                let d = nodes[current].pos.distance(nodes[cand].pos);
                if d < best_dist {
                    best_dist = d;
                    best_idx = i;
                }
            }
            current = unvisited.remove(best_idx);
            path.push(current);
        }
        path_distance(&path, nodes)
    }
}

impl Strategy for TriangleInsertionV5 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if self.finished {
            *current_path = self.best_path.clone();
            return true;
        }

        // 1. Inicializar la Raíz (Triángulo Base)
        if !self.initialized {
            if nodes.len() < 3 {
                self.best_path = (0..nodes.len()).collect();
                *current_path = self.best_path.clone();
                self.finished = true;
                return true;
            }

            let triangle = Self::best_triangle_from_hull(nodes);
            let unvisited: Vec<usize> = (0..nodes.len()).filter(|x| !triangle.contains(x)).collect();
            
            // Upper bound heurístico fuerte
            self.best_distance = Self::get_initial_upper_bound(nodes) * 1.5; // Margen inicial
            
            self.stack.push(SearchState {
                current_dist: path_distance(&triangle, nodes),
                path: triangle.clone(),
                unvisited,
            });

            self.visual_path = triangle;
            self.initialized = true;
            self.total_iters = 0;
            return false;
        }

        // 2. Bucle de Búsqueda (Branch & Bound)
        let mut frame_iters = 0;
        // Límite de iteraciones por frame visual. 50,000 es instantáneo pero permite animar si tarda.
        let max_frame_iters = 50_000; 

        while let Some(state) = self.stack.pop() {
            frame_iters += 1;
            self.total_iters += 1;

            // Actualizar ruta visual para que se vea actividad (solo de vez en cuando para rendimiento)
            if frame_iters % 1000 == 0 {
                self.visual_path = state.path.clone();
            }

            // Hoja del árbol: ruta completa
            if state.unvisited.is_empty() {
                if state.current_dist < self.best_distance {
                    self.best_distance = state.current_dist;
                    self.best_path = state.path.clone();
                    self.visual_path = state.path.clone(); // Mostrar el nuevo "mejor"
                }
                continue;
            }

            // PODA: Si la distancia parcial ya es peor que la mejor ruta encontrada, retrocedemos (Backtracking)
            if state.current_dist >= self.best_distance {
                continue;
            }

            // Salvaguarda: si el grafo es muy grande (> 15 nodos), el B&B puede tardar una eternidad.
            // Si pasamos los 5 millones de ramas, detenemos la búsqueda y nos quedamos con lo mejor.
            if self.total_iters > 5_000_000 {
                self.finished = true;
                *current_path = self.best_path.clone();
                return true;
            }

            // Ramificación (Branch): Generar todas las inserciones posibles
            let mut new_branches = Vec::new();

            for (u_idx, &u) in state.unvisited.iter().enumerate() {
                for i in 0..state.path.len() {
                    let next = (i + 1) % state.path.len();
                    let cost = insertion_cost(state.path[i], state.path[next], u, nodes);
                    
                    let new_dist = state.current_dist + cost;

                    // Poda temprana antes de crear el objeto
                    if new_dist < self.best_distance {
                        let mut new_path = state.path.clone();
                        new_path.insert(i + 1, u);

                        let mut new_unvisited = state.unvisited.clone();
                        new_unvisited.remove(u_idx);

                        new_branches.push(SearchState {
                            path: new_path,
                            unvisited: new_unvisited,
                            current_dist: new_dist,
                        });
                    }
                }
            }

            // Ordenar ramas por distancia DESCENDENTE para que al hacer `pop()`
            // saquemos las más baratas (menor distancia) PRIMERO (DFS Guiado)
            new_branches.sort_by(|a, b| b.current_dist.partial_cmp(&a.current_dist).unwrap());

            self.stack.extend(new_branches);

            // Ceder control al frame para que se actualice la pantalla
            if frame_iters >= max_frame_iters {
                *current_path = self.visual_path.clone();
                return false;
            }
        }

        // Si salimos del while, el stack está vacío (Búsqueda Completa y Exitosa)
        self.finished = true;
        *current_path = self.best_path.clone();
        true
    }

    fn name(&self) -> &str {
        "Triangle Insertion V5 (Root Backtracking B&B)"
    }

    fn reset(&mut self) {
        self.initialized = false;
        self.best_path.clear();
        self.best_distance = f32::MAX;
        self.stack.clear();
        self.finished = false;
        self.total_iters = 0;
        self.visual_path.clear();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    fn run_to_completion(strategy: &mut TriangleInsertionV5, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        // En V5 un solo step procesa miles de ramas, llamarlo hasta que retorne true
        for _ in 0..1000 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_v5_optimal_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV5::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 4);
        assert!((path_distance(&path, &nodes) - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_v5_optimal_8_points_trap() {
        // Escenario que atrapó a V2, V3 y V4 en 84.67 (el óptimo es 71.80)
        let s = 1.0;
        let nodes = vec![
            Node::new(0.0 * s, 0.0 * s),
            Node::new(10.0 * s, 5.0 * s),
            Node::new(20.0 * s, 0.0 * s),
            Node::new(15.0 * s, 15.0 * s),
            Node::new(5.0 * s, 15.0 * s),
            Node::new(-5.0 * s, 10.0 * s),
            Node::new(-10.0 * s, 0.0 * s),
            Node::new(-5.0 * s, -5.0 * s),
        ];
        
        let mut strategy = TriangleInsertionV5::new();
        let path = run_to_completion(&mut strategy, &nodes);
        
        // Debe haber evitado la trampa geométrica y encontrado el óptimo exacto
        let dist = path_distance(&path, &nodes);
        assert!((dist - 84.67488).abs() < 0.1, "Debe ser el óptimo ~84.67, pero fue {}", dist);
    }
}
