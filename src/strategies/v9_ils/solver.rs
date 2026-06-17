/// Solver ILS para V9.
///
/// Componentes iterables:
/// - `build_initial()`: construye un tour con V9 REI.
/// - `local_search(path)`: optimiza el tour con búsqueda local guiada.
/// - `perturb(path)`: aplica double-bridge.
/// - `solve()`: ejecuta el loop ILS completo.
use crate::core::path_distance;
use crate::strategies::triangle_insertion_v9::{TriangleInsertionV9, V9Params};

use super::construction::{V9ConstructionParams, V9Constructor};
use super::local_search::{LocalSearchConfig, LocalSearcher};
use super::perturbation::double_bridge;

#[derive(Clone, Copy)]
pub struct V9IlsParams {
    pub v9: V9Params,
    pub max_iters: usize,
    pub ls_config: LocalSearchConfig,
}

impl Default for V9IlsParams {
    fn default() -> Self {
        Self {
            v9: V9Params::default(),
            max_iters: 10,
            ls_config: LocalSearchConfig::default(),
        }
    }
}

pub struct V9IlsSolver<'a> {
    nodes: &'a [crate::core::Node],
    params: V9IlsParams,
    local_searcher: LocalSearcher<'a>,
}

impl<'a> V9IlsSolver<'a> {
    pub fn new(nodes: &'a [crate::core::Node], params: V9IlsParams) -> Self {
        let local_searcher = LocalSearcher::new(nodes, params.ls_config);
        Self {
            nodes,
            params,
            local_searcher,
        }
    }

    pub fn build_initial(&self) -> Vec<usize> {
        V9Constructor::new(V9ConstructionParams::from(self.params.v9)).build(self.nodes)
    }

    pub fn local_search(&self, path: &[usize]) -> Vec<usize> {
        let mut improved = path.to_vec();
        self.local_searcher.optimize(&mut improved);
        improved
    }

    pub fn perturb(&self, path: &[usize]) -> Vec<usize> {
        double_bridge(path)
    }

    pub fn solve(&mut self) -> Vec<usize> {
        let mut best = self.build_initial();
        TriangleInsertionV9::optimize_full(&mut best, self.nodes);
        let mut best_dist = path_distance(&best, self.nodes);
        let mut current = best.clone();

        for _ in 0..self.params.max_iters {
            let mut candidate = self.perturb(&current);
            candidate = self.local_search(&candidate);

            let candidate_dist = path_distance(&candidate, self.nodes);
            if candidate_dist < best_dist {
                best_dist = candidate_dist;
                best = candidate.clone();
                current = candidate;
            }
        }

        TriangleInsertionV9::optimize_full(&mut best, self.nodes);
        best
    }
}
