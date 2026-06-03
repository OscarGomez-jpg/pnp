/// Estrategia: Vecino Más Cercano (Nearest Neighbor - Greedy clásico)

use super::Strategy;
use crate::core::Node;

pub struct NearestNeighbor;

impl NearestNeighbor {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for NearestNeighbor {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        let unvisited: Vec<usize> = (0..nodes.len())
            .filter(|idx| !current_path.contains(idx))
            .collect();

        if unvisited.is_empty() {
            return true; // Finalizado
        }

        if current_path.is_empty() {
            current_path.push(0);
            return false;
        }

        let last_idx = current_path[current_path.len() - 1];
        let last_pos = nodes[last_idx].pos;

        if let Some(&closest) = unvisited.iter().min_by(|&&a, &&b| {
            last_pos
                .distance(nodes[a].pos)
                .partial_cmp(&last_pos.distance(nodes[b].pos))
                .unwrap()
        }) {
            current_path.push(closest);
        }

        false
    }

    fn name(&self) -> &str {
        "Vecino Más Cercano (Estándar)"
    }

    fn reset(&mut self) {
        // Esta estrategia no tiene estado que resetear
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_neighbor_starts_at_node_0() {
        let mut strategy = NearestNeighbor::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
        ];
        let mut path = vec![];
        strategy.execute_step(&mut path, &nodes);
        assert_eq!(path, vec![0]);
    }

    #[test]
    fn test_nearest_neighbor_selects_closest() {
        let mut strategy = NearestNeighbor::new();
        let nodes = vec![
            Node::new(0.0, 0.0),   // inicio
            Node::new(1.0, 0.0),   // cercano
            Node::new(10.0, 10.0), // lejano
        ];
        let mut path = vec![0];
        strategy.execute_step(&mut path, &nodes);
        assert_eq!(path[1], 1); // Debería seleccionar el nodo 1 (más cercano)
    }

    #[test]
    fn test_nearest_neighbor_completes() {
        let mut strategy = NearestNeighbor::new();
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(1.0, 0.0),
            Node::new(0.0, 1.0),
        ];
        let mut path = vec![];

        // Ejecutar hasta que termine
        let mut finished = false;
        for _ in 0..10 {
            finished = strategy.execute_step(&mut path, &nodes);
            if finished {
                break;
            }
        }

        assert!(finished);
        assert_eq!(path.len(), 3); // Todos los nodos visitados
    }
}
