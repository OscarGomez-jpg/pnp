/// Estrategia: Triángulo + Inserción Inteligente con Look-Ahead Query
/// 
/// Algoritmo basado en query que:
/// 1. Inicia con un triángulo obligado [0, 1, 2]
/// 2. En cada paso, evalúa los 2 nodos más cercanos (P1, P2)
/// 3. Calcula costo de inserción para cada uno
/// 4. Inserta el que menos impacto tenga en el perímetro total

use super::Strategy;
use crate::core::{insertion_cost, Node};

pub struct TriangleInsertion {
    triangle_initialized: bool,
}

impl TriangleInsertion {
    pub fn new() -> Self {
        Self {
            triangle_initialized: false,
        }
    }
}

impl Strategy for TriangleInsertion {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            return true; // Finalizado
        }

        // Paso 1: Inicializar con triángulo obligado
        if current_path.is_empty() && nodes.len() >= 3 {
            current_path.extend_from_slice(&[0, 1, 2]);
            self.triangle_initialized = true;
            return false;
        }

        if !self.triangle_initialized {
            return true; // No hay suficientes nodos
        }

        // Paso 2: Query de look-ahead - obtener los 2 más cercanos
        let last_node_idx = current_path[current_path.len() - 1];
        let last_node_pos = nodes[last_node_idx].pos;

        let mut sorted_unvisited = unvisited;
        sorted_unvisited.sort_by(|&a, &b| {
            last_node_pos
                .distance(nodes[a].pos)
                .partial_cmp(&last_node_pos.distance(nodes[b].pos))
                .unwrap()
        });

        let p1 = sorted_unvisited.get(0).cloned();
        let p2 = sorted_unvisited.get(1).cloned();

        // Paso 3: Evaluar P1
        let mut best_cost_p1 = f32::MAX;
        let mut best_idx_p1 = 0;

        if let Some(target_p1) = p1 {
            for i in 0..current_path.len() {
                let cost = insertion_cost(
                    current_path[i],
                    current_path[(i + 1) % current_path.len()],
                    target_p1,
                    nodes,
                );
                if cost < best_cost_p1 {
                    best_cost_p1 = cost;
                    best_idx_p1 = i + 1;
                }
            }
        }

        // Paso 4: Comparar con P2 y decidir
        let mut candidate = p1.unwrap_or(0);
        let mut final_idx = best_idx_p1;

        if let Some(target_p2) = p2 {
            let mut best_cost_p2 = f32::MAX;
            let mut best_idx_p2 = 0;

            for i in 0..current_path.len() {
                let cost = insertion_cost(
                    current_path[i],
                    current_path[(i + 1) % current_path.len()],
                    target_p2,
                    nodes,
                );
                if cost < best_cost_p2 {
                    best_cost_p2 = cost;
                    best_idx_p2 = i + 1;
                }
            }

            // Decisión final: insertar el que tenga menor costo
            if best_cost_p2 < best_cost_p1 {
                candidate = target_p2;
                final_idx = best_idx_p2;
            }
        }

        current_path.insert(final_idx, candidate);
        false
    }

    fn name(&self) -> &str {
        "Triángulo + Inserción Inteligente (Tu Query)"
    }

    fn reset(&mut self) {
        self.triangle_initialized = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_initialization() {
        let mut strategy = TriangleInsertion::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
            Node::new(1.0, 1.0),
        ];
        let mut path = vec![];
        strategy.execute_step(&mut path, &nodes);
        
        assert_eq!(path, vec![0, 1, 2]);
        assert!(strategy.triangle_initialized);
    }

    #[test]
    fn test_triangle_requires_3_nodes() {
        let mut strategy = TriangleInsertion::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(1.0, 0.0),
        ];
        let mut path = vec![];
        let finished = strategy.execute_step(&mut path, &nodes);
        
        // Con menos de 3 nodos, debería finalizar
        assert!(finished);
        assert!(path.is_empty());
    }

    #[test]
    fn test_triangle_completes_with_4_nodes() {
        let mut strategy = TriangleInsertion::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
            Node::new(1.5, 2.0),
        ];
        let mut path = vec![];

        let mut finished = false;
        for _ in 0..10 {
            finished = strategy.execute_step(&mut path, &nodes);
            if finished {
                break;
            }
        }

        assert!(finished);
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn test_reset() {
        let mut strategy = TriangleInsertion::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(1.0, 0.0),
            Node::new(0.0, 1.0),
        ];
        let mut path = vec![];
        
        strategy.execute_step(&mut path, &nodes);
        assert!(!path.is_empty());
        
        strategy.reset();
        assert!(!strategy.triangle_initialized);
    }
}
