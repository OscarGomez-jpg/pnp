/// Triangle Insertion V9 + ILS (Iterated Local Search)
///
/// Versión modular y optimizada de V9+ILS. Separa construcción,
/// búsqueda local, perturbación y el loop ILS en componentes iterables.
pub mod construction;
pub mod local_search;
pub mod perturbation;
pub mod solver;

use super::Strategy;
use crate::core::Node;

pub use construction::V9ConstructionParams;
pub use solver::{V9IlsParams, V9IlsSolver};

/// Estrategia batch que ejecuta el solver ILS completo en un paso.
pub struct TriangleInsertionV9Ils {
    params: V9IlsParams,
    finished: bool,
}

impl TriangleInsertionV9Ils {
    pub fn new() -> Self {
        Self {
            params: V9IlsParams::default(),
            finished: false,
        }
    }

    pub fn with_params(params: V9IlsParams) -> Self {
        Self {
            params,
            finished: false,
        }
    }

    pub fn set_params(&mut self, params: V9IlsParams) {
        self.params = params;
    }
}

impl Strategy for TriangleInsertionV9Ils {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if self.finished {
            return true;
        }

        let mut solver = V9IlsSolver::new(nodes, self.params);
        *current_path = solver.solve();
        self.finished = true;
        true
    }

    fn name(&self) -> &str {
        "Triangle Insertion V9 + ILS"
    }

    fn reset(&mut self) {
        self.finished = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Node, path_distance};

    #[test]
    fn test_v9_ils_visits_all_nodes() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
            Node::new(5.0, 5.0),
        ];

        let mut strategy = TriangleInsertionV9Ils::new();
        let mut path = Vec::new();
        let finished = strategy.execute_step(&mut path, &nodes);

        assert!(finished);
        assert_eq!(path.len(), nodes.len());
    }

    #[test]
    fn test_v9_ils_solver_iterable() {
        let nodes: Vec<Node> = (0..20)
            .map(|i| Node::new((i as f32) * 3.0, (i as f32).sin() * 10.0))
            .collect();

        let solver = V9IlsSolver::new(&nodes, V9IlsParams::default());
        let initial = solver.build_initial();
        let improved = solver.local_search(&initial);

        assert_eq!(improved.len(), nodes.len());
        assert!(path_distance(&improved, &nodes) <= path_distance(&initial, &nodes) * 1.001);
    }
}
