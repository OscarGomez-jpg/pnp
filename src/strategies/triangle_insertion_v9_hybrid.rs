/// Estrategia híbrida: selecciona entre V9 (Recursive Edge Insertion) y V8.9
/// (Pre-Seagull) según las características geométricas de la instancia.
///
/// La heurística de selección se basa en el análisis de clusters:
/// - Si la instancia tiene clusters bien separados (detectados por DBSCAN simple),
///   V9 tiende a funcionar mejor porque rellena clusters completos antes de saltar.
/// - En instancias más uniformes o con una sola componente conectada densa,
///   V8.9 suele ser más robusto.
use super::Strategy;
use crate::core::Node;
use crate::strategies::triangle_insertion_v8_9::TriangleInsertionV89;
use crate::strategies::triangle_insertion_v9::TriangleInsertionV9;
use std::any::Any;

/// Resultado del análisis geométrico de una instancia.
#[derive(Debug, Clone)]
pub struct InstanceGeometry {
    pub dbscan_clusters: usize,
    pub nearest_ratio: f32,
    pub dispersion: f32,
}

pub struct TriangleInsertionV9Hybrid {
    strategy: Box<dyn Strategy>,
    chosen_name: String,
}

impl TriangleInsertionV9Hybrid {
    pub fn new() -> Self {
        Self {
            strategy: Box::new(TriangleInsertionV9::new()),
            chosen_name: "V9Hybrid (pending analysis)".to_string(),
        }
    }

    /// Analiza la geometría de la instancia y elige la estrategia subyacente.
    fn select_strategy(nodes: &[Node]) -> (Box<dyn Strategy>, String) {
        let geom = Self::analyze_geometry(nodes);

        // Regla de selección:
        // - Clusters bien separados (>= 2) favorecen V9.
        // - También usamos V9 si nearest_ratio es muy bajo (clusters fuertes)
        //   y la dispersión no es excesiva (evita instancias con escalas muy mixtas).
        let use_v9 = geom.dbscan_clusters >= 2
            || (geom.nearest_ratio < 0.10 && geom.dispersion < 500.0);

        if use_v9 {
            (
                Box::new(TriangleInsertionV9::new()),
                "V9Hybrid → V9 (Recursive Edge Insertion)".to_string(),
            )
        } else {
            (
                Box::new(TriangleInsertionV89::new()),
                "V9Hybrid → V8.9 (Pre-Seagull)".to_string(),
            )
        }
    }

    fn analyze_geometry(nodes: &[Node]) -> InstanceGeometry {
        if nodes.len() < 3 {
            return InstanceGeometry {
                dbscan_clusters: 1,
                nearest_ratio: 1.0,
                dispersion: 1.0,
            };
        }

        // Calcular todas las distancias
        let mut all_dists: Vec<f32> = Vec::with_capacity(nodes.len() * nodes.len());
        let mut nearest_dists: Vec<f32> = Vec::with_capacity(nodes.len());

        for i in 0..nodes.len() {
            let mut min_dist = f32::MAX;
            for j in 0..nodes.len() {
                if i == j {
                    continue;
                }
                let d = nodes[i].distance_to(&nodes[j]);
                all_dists.push(d);
                if d < min_dist {
                    min_dist = d;
                }
            }
            nearest_dists.push(min_dist);
        }

        let mean_nearest = nearest_dists.iter().sum::<f32>() / nearest_dists.len() as f32;
        let mean_all = all_dists.iter().sum::<f32>() / all_dists.len() as f32;
        let nearest_ratio = if mean_all > 1e-6 {
            mean_nearest / mean_all
        } else {
            1.0
        };

        // Percentil 25 para DBSCAN
        let mut sorted = all_dists.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let p25_idx = ((0.25 * (sorted.len() - 1) as f32) as usize).min(sorted.len() - 1);
        let eps = sorted[p25_idx];

        // DBSCAN simple: contar componentes conectadas con distancia <= eps
        let mut visited = vec![false; nodes.len()];
        let mut cluster_count = 0;
        for i in 0..nodes.len() {
            if visited[i] {
                continue;
            }
            cluster_count += 1;
            let mut stack = vec![i];
            visited[i] = true;
            while let Some(current) = stack.pop() {
                for j in 0..nodes.len() {
                    if visited[j] {
                        continue;
                    }
                    if nodes[current].distance_to(&nodes[j]) <= eps {
                        visited[j] = true;
                        stack.push(j);
                    }
                }
            }
        }

        // Dispersión: distancia máxima / distancia mínima
        let min_dist = sorted[0];
        let max_dist = sorted[sorted.len() - 1];
        let dispersion = if min_dist > 1e-6 {
            max_dist / min_dist
        } else {
            1.0
        };

        InstanceGeometry {
            dbscan_clusters: cluster_count,
            nearest_ratio,
            dispersion,
        }
    }

    pub fn geometry(nodes: &[Node]) -> InstanceGeometry {
        Self::analyze_geometry(nodes)
    }
}

impl Strategy for TriangleInsertionV9Hybrid {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        // En el primer paso, analizamos la instancia y elegimos estrategia.
        if current_path.is_empty() && self.chosen_name.contains("pending") {
            let (strategy, name) = Self::select_strategy(nodes);
            self.strategy = strategy;
            self.chosen_name = name;
        }
        self.strategy.execute_step(current_path, nodes)
    }

    fn name(&self) -> &str {
        &self.chosen_name
    }

    fn reset(&mut self) {
        self.strategy.reset();
        self.chosen_name = "V9Hybrid (pending analysis)".to_string();
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;

    fn run_to_completion(strategy: &mut TriangleInsertionV9Hybrid, nodes: &[Node]) -> Vec<usize> {
        let mut path = vec![];
        for _ in 0..nodes.len() + 10 {
            if strategy.execute_step(&mut path, nodes) {
                break;
            }
        }
        path
    }

    #[test]
    fn test_hybrid_visits_all_nodes_square() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];
        let mut strategy = TriangleInsertionV9Hybrid::new();
        let path = run_to_completion(&mut strategy, &nodes);
        assert_eq!(path.len(), 4, "Debe visitar todos los nodos");
    }

    #[test]
    fn test_hybrid_selects_strategy() {
        // Dos clusters bien separados deberían favorecer V9
        let mut nodes = Vec::new();
        for i in 0..5 {
            nodes.push(Node::new(i as f32 * 2.0, 0.0));
            nodes.push(Node::new(100.0 + i as f32 * 2.0, 100.0));
        }

        let geom = TriangleInsertionV9Hybrid::geometry(&nodes);
        assert!(
            geom.dbscan_clusters >= 2,
            "Dos clusters separados deberían detectarse"
        );
    }
}
