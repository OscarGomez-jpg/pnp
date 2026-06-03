/// Estrategia: Triangle Insertion V2 con Optimización Local 2-Opt
///
/// Mejoras respecto a V1:
/// 1. Triangle initialization inteligente (encuentra el mejor triángulo inicial)
/// 2. Multi-point query: evalúa 3 candidatos en lugar de 2
/// 3. Optimización 2-Opt local después de cada inserción
/// 4. Smart position selection: considera tanto costo como geometría
use super::Strategy;
use crate::core::{Node, insertion_cost};

pub struct TriangleInsertionV2 {
    triangle_initialized: bool,
    last_improvement: usize, // Rastrear si 2-Opt hizo mejoras
}

impl TriangleInsertionV2 {
    pub fn new() -> Self {
        Self {
            triangle_initialized: false,
            last_improvement: 0,
        }
    }

    /// Encuentra el mejor triángulo inicial considerando área y perímetro
    fn find_best_triangle(nodes: &[Node]) -> Vec<usize> {
        if nodes.len() < 3 {
            return (0..nodes.len()).collect();
        }

        let mut best_triangle = vec![0, 1, 2];
        let mut best_metric = Self::triangle_metric(&best_triangle, nodes);

        // Probar todos los triángulos posibles (máximo 10 nodos)
        let limit = nodes.len().min(10);
        for i in 0..limit {
            for j in (i + 1)..limit {
                for k in (j + 1)..limit {
                    let triangle = vec![i, j, k];
                    let metric = Self::triangle_metric(&triangle, nodes);
                    // Preferimos triángulos más grandes y bien formados
                    if metric > best_metric {
                        best_metric = metric;
                        best_triangle = triangle;
                    }
                }
            }
        }

        best_triangle
    }

    /// Calcula métrica de calidad del triángulo (perímetro / área)
    fn triangle_metric(triangle: &[usize], nodes: &[Node]) -> f32 {
        if triangle.len() != 3 {
            return 0.0;
        }

        let p0 = nodes[triangle[0]].pos;
        let p1 = nodes[triangle[1]].pos;
        let p2 = nodes[triangle[2]].pos;

        // Calcular perímetro
        let d01 = p0.distance(p1);
        let d12 = p1.distance(p2);
        let d20 = p2.distance(p0);
        let perimeter = d01 + d12 + d20;

        // Calcular área usando producto cruz
        let v1 = (p1.x - p0.x, p1.y - p0.y);
        let v2 = (p2.x - p0.x, p2.y - p0.y);
        let area = (v1.0 * v2.1 - v1.1 * v2.0).abs() / 2.0;

        // Métrica: triángulos grandes y bien formados
        if area > 0.1 { perimeter / area } else { 0.0 }
    }

    /// Optimización 2-Opt: intenta intercambiar pares de aristas
    /// para reducir el costo total sin perder visitación
    fn optimize_2opt(path: &mut Vec<usize>, nodes: &[Node], max_iterations: usize) -> bool {
        let mut improved = false;

        for _iteration in 0..max_iterations {
            let mut local_improved = false;

            for i in 0..path.len() - 2 {
                for j in (i + 2)..path.len() {
                    if i == 0 && j == path.len() - 1 {
                        continue; // No intercambiar los extremos del ciclo
                    }

                    // Calcular costo actual: (i -> i+1) + (j -> j+1 mod n)
                    let current_cost = {
                        let p1 = nodes[path[i]].pos;
                        let p2 = nodes[path[(i + 1) % path.len()]].pos;
                        let p3 = nodes[path[j]].pos;
                        let p4 = nodes[path[(j + 1) % path.len()]].pos;

                        p1.distance(p2) + p3.distance(p4)
                    };

                    // Calcular costo si intercambiamos: (i -> j) + (i+1 -> j+1 mod n)
                    let new_cost = {
                        let p1 = nodes[path[i]].pos;
                        let p2 = nodes[path[j]].pos;
                        let p3 = nodes[path[(i + 1) % path.len()]].pos;
                        let p4 = nodes[path[(j + 1) % path.len()]].pos;

                        p1.distance(p2) + p3.distance(p4)
                    };

                    // Si es mejor, realizar el intercambio
                    if new_cost < current_cost - 0.01 {
                        // Invertir el segmento entre i+1 y j
                        let mut start = i + 1;
                        let mut end = j;
                        while start < end {
                            path.swap(start, end);
                            start += 1;
                            end -= 1;
                        }
                        local_improved = true;
                        improved = true;
                    }
                }
            }

            if !local_improved {
                break; // No hay más mejoras posibles
            }
        }

        improved
    }
}

impl Strategy for TriangleInsertionV2 {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            // Optimización final antes de terminar
            Self::optimize_2opt(current_path, nodes, 5);
            return true;
        }

        // Paso 1: Inicializar con triángulo óptimo
        if current_path.is_empty() && nodes.len() >= 3 {
            let triangle = Self::find_best_triangle(nodes);
            current_path.extend_from_slice(&triangle);
            self.triangle_initialized = true;
            return false;
        }

        if !self.triangle_initialized {
            return true;
        }

        // Paso 2: Query multi-point - obtener los 3 más cercanos
        let last_node_idx = current_path[current_path.len() - 1];
        let last_node_pos = nodes[last_node_idx].pos;

        let mut sorted_unvisited = unvisited;
        sorted_unvisited.sort_by(|&a, &b| {
            last_node_pos
                .distance(nodes[a].pos)
                .partial_cmp(&last_node_pos.distance(nodes[b].pos))
                .unwrap()
        });

        let candidates: Vec<usize> = sorted_unvisited.iter().take(3).cloned().collect();

        // Paso 3: Evaluar cada candidato y seleccionar el mejor
        let mut best_cost = f32::MAX;
        let mut best_candidate = candidates[0];
        let mut best_idx = 0;

        for &candidate in &candidates {
            for i in 0..current_path.len() {
                let cost = insertion_cost(
                    current_path[i],
                    current_path[(i + 1) % current_path.len()],
                    candidate,
                    nodes,
                );
                if cost < best_cost {
                    best_cost = cost;
                    best_candidate = candidate;
                    best_idx = i + 1;
                }
            }
        }

        current_path.insert(best_idx, best_candidate);

        // Paso 4: Aplicar optimización 2-Opt local (2 iteraciones máximo)
        Self::optimize_2opt(current_path, nodes, 2);

        false
    }

    fn name(&self) -> &str {
        "Triangle Insertion V2 (Smart + 2-Opt)"
    }

    fn reset(&mut self) {
        self.triangle_initialized = false;
        self.last_improvement = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    #[test]
    fn test_smart_triangle_insertion_v2_simple() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];

        let mut strategy = TriangleInsertionV2::new();
        let mut path = vec![];

        loop {
            let finished = strategy.execute_step(&mut path, &nodes);
            if finished {
                break;
            }
        }

        // Verificar que visitó todos los nodos
        assert_eq!(path.len(), 4);
        for i in 0..4 {
            assert!(path.contains(&i));
        }
    }

    #[test]
    fn test_smart_triangle_insertion_v2_reset() {
        let mut strategy = TriangleInsertionV2::new();
        strategy.triangle_initialized = true;

        strategy.reset();
        assert!(!strategy.triangle_initialized);
    }

    #[test]
    fn test_smart_triangle_selection() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(100.0, 0.0),
            Node::new(50.0, 86.6), // triángulo equilátero ~ 100
            Node::new(50.0, 50.0),
        ];

        let triangle = TriangleInsertionV2::find_best_triangle(&nodes);
        assert_eq!(triangle.len(), 3);
        // El triángulo seleccionado debe ser uno válido
        assert!(triangle[0] < nodes.len());
        assert!(triangle[1] < nodes.len());
        assert!(triangle[2] < nodes.len());
    }
}
