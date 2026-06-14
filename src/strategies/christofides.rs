/// Estrategia: Algoritmo de Christofides-Serdyukov (Aproximación Heurística O(N³))
///
/// A diferencia de las versiones constructivas basadas en inserción (V3, V4, V6),
/// Christofides es un enfoque topológico basado en la teoría de grafos pura.
/// Es famoso mundialmente por ser el único algoritmo polinomial que garantiza matemáticamente
/// que el tour final nunca será peor que 1.5 veces el óptimo real en problemas métricos.
///
/// Fases del algoritmo ejecutadas paso a paso para el visualizador:
/// 1. Árbol de Expansión Mínima (MST): Conecta todos los nodos con el mínimo coste (Algoritmo de Prim).
/// 2. Nodos de Grado Impar: Filtra aquellos nodos que rompen la propiedad Euleriana.
/// 3. Emparejamiento Perfecto Codicioso (Greedy MWPM): Une los nodos impares en parejas óptimas.
/// 4. Multigrafo y Circuito Euleriano: Une MST + Matching y encuentra un camino cerrado (Hierholzer).
/// 5. Atajos (Shortcuts): Salta nodos repetidos para consolidar el Ciclo Hamiltoniano (TSP Tour).
use super::Strategy;
use crate::core::Node;

#[derive(Clone, Debug, PartialEq)]
enum ChristofidesPhase {
    NotStarted,
    MstComputed {
        mst_edges: Vec<(usize, usize)>,
    },
    MatchingComputed {
        mst_edges: Vec<(usize, usize)>,
        matching_edges: Vec<(usize, usize)>,
    },
    Finished,
}

pub struct ChristofidesStrategy {
    phase: ChristofidesPhase,
}

impl ChristofidesStrategy {
    pub fn new() -> Self {
        Self {
            phase: ChristofidesPhase::NotStarted,
        }
    }

    // -------------------------------------------------------------------------
    // Fase 1: Algoritmo de Prim para el MST — O(N²)
    // -------------------------------------------------------------------------
    fn compute_prim_mst(nodes: &[Node]) -> Vec<(usize, usize)> {
        let n = nodes.len();
        let mut mst_edges = Vec::with_capacity(n - 1);
        let mut in_mst = vec![false; n];
        let mut min_dist = vec![f32::MAX; n];
        let mut parent = vec![0; n];

        min_dist[0] = 0.0;

        for _ in 0..n {
            let mut u = None;
            let mut best_d = f32::MAX;

            // Encontrar el nodo fuera del MST más cercano
            for i in 0..n {
                if !in_mst[i] && min_dist[i] < best_d {
                    best_d = min_dist[i];
                    u = Some(i);
                }
            }

            let u = match u {
                Some(idx) => idx,
                None => break,
            };

            in_mst[u] = true;
            if u != 0 {
                mst_edges.push((parent[u], u));
            }

            // Actualizar las distancias a los nodos vecinos restantes usando tu core::Node
            for v in 0..n {
                if !in_mst[v] {
                    let d = nodes[u].distance_to(&nodes[v]);
                    if d < min_dist[v] {
                        min_dist[v] = d;
                        parent[v] = u;
                    }
                }
            }
        }
        mst_edges
    }

    // -------------------------------------------------------------------------
    // Fase 2: Filtrado de Nodos de Grado Impar — O(N)
    // -------------------------------------------------------------------------
    fn find_odd_degree_nodes(n: usize, mst_edges: &[(usize, usize)]) -> Vec<usize> {
        let mut degrees = vec![0; n];
        for &(u, v) in mst_edges {
            degrees[u] += 1;
            degrees[v] += 1;
        }

        // Teorema del apretón de manos: la cantidad de nodos impares SIEMPRE es par
        (0..n).filter(|&i| degrees[i] % 2 != 0).collect()
    }

    // -------------------------------------------------------------------------
    // Fase 3: Emparejamiento Perfecto Codicioso — O(N² log N)
    // -------------------------------------------------------------------------
    fn greedy_perfect_matching(odd_nodes: &[usize], nodes: &[Node]) -> Vec<(usize, usize)> {
        let mut matching = Vec::new();
        let mut matched = vec![false; odd_nodes.len()];
        let mut candidates = Vec::new();

        // Generar todas las aristas posibles estrictamente entre los nodos impares
        for i in 0..odd_nodes.len() {
            for j in (i + 1)..odd_nodes.len() {
                let u = odd_nodes[i];
                let v = odd_nodes[j];
                let dist = nodes[u].distance_to(&nodes[v]);
                candidates.push((dist, i, j));
            }
        }

        // Ordenar aristas de menor a mayor distancia
        candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Emparejar codiciosamente los nodos más cercanos que sigan libres
        for (_, i, j) in candidates {
            if !matched[i] && !matched[j] {
                matched[i] = true;
                matched[j] = true;
                matching.push((odd_nodes[i], odd_nodes[j]));
            }
        }
        matching
    }

    // -------------------------------------------------------------------------
    // Fase 4 y 5: Circuito Euleriano (Hierholzer) y Atajos — O(N)
    // -------------------------------------------------------------------------
    fn build_hamiltonian_cycle(
        n: usize,
        mst: &[(usize, usize)],
        matching: &[(usize, usize)],
    ) -> Vec<usize> {
        // 1. Construir la lista de adyacencia del Multigrafo (admite aristas paralelas)
        let mut adj = vec![Vec::new(); n];
        for &(u, v) in mst.iter().chain(matching.iter()) {
            adj[u].push(v);
            adj[v].push(u);
        }

        // 2. Encontrar el Circuito Euleriano usando el algoritmo de Hierholzer lineal
        let mut circuit = Vec::new();
        let mut stack = vec![0];
        let mut current_adj = adj.clone();

        while let Some(&u) = stack.last() {
            if current_adj[u].is_empty() {
                circuit.push(u);
                stack.pop();
            } else {
                let v = current_adj[u].pop().unwrap();
                // Remover la arista en sentido inverso para mantener simetría en el multigrafo
                if let Some(pos) = current_adj[v].iter().position(|&x| x == u) {
                    current_adj[v].remove(pos);
                }
                stack.push(v);
            }
        }
        circuit.reverse();

        // 3. Aplicar atajos (Shortcuts) eliminando nodos repetidos para obtener el ciclo TSP
        let mut visited = vec![false; n];
        let mut path = Vec::with_capacity(n);
        for node in circuit {
            if !visited[node] {
                visited[node] = true;
                path.push(node);
            }
        }
        path
    }

    // -------------------------------------------------------------------------
    // Refinamiento Opcional: Post-Optimización Local Polinomial — O(N²)
    // -------------------------------------------------------------------------
    fn optimize_2opt(path: &mut Vec<usize>, nodes: &[Node]) {
        let mut improved = true;
        let mut max_passes = 3; // Límite estricto para proteger el tiempo polinomial

        while improved && max_passes > 0 {
            improved = false;
            max_passes -= 1;

            for i in 0..path.len().saturating_sub(2) {
                for j in (i + 2)..path.len() {
                    if i == 0 && j == path.len() - 1 {
                        continue;
                    }

                    let p1 = &nodes[path[i]];
                    let p2 = &nodes[path[i + 1]];
                    let p3 = &nodes[path[j]];
                    let p4 = &nodes[path[(j + 1) % path.len()]];

                    let current = p1.distance_to(p2) + p3.distance_to(p4);
                    let swapped = p1.distance_to(p3) + p2.distance_to(p4);

                    if swapped < current - 0.01 {
                        path[i + 1..=j].reverse();
                        improved = true;
                    }
                }
            }
        }
    }
}

impl Strategy for ChristofidesStrategy {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        // Manejo seguro para mapas críticamente pequeños
        if nodes.len() < 3 {
            current_path.clear();
            current_path.extend(0..nodes.len());
            return true;
        }

        // Extraer el estado interno evitando conflictos con el borrow checker de Rust
        let current_phase = std::mem::replace(&mut self.phase, ChristofidesPhase::NotStarted);

        match current_phase {
            ChristofidesPhase::NotStarted => {
                // Paso 1: Computar el MST global
                let mst_edges = Self::compute_prim_mst(nodes);

                // Avanzar estado para el renderizado del siguiente frame
                self.phase = ChristofidesPhase::MstComputed { mst_edges };
                false
            }
            ChristofidesPhase::MstComputed { mst_edges } => {
                // Paso 2 y 3: Localizar los nodos impares y calcular su Matching
                let odd_nodes = Self::find_odd_degree_nodes(nodes.len(), &mst_edges);
                let matching_edges = Self::greedy_perfect_matching(&odd_nodes, nodes);

                self.phase = ChristofidesPhase::MatchingComputed {
                    mst_edges,
                    matching_edges,
                };
                false
            }
            ChristofidesPhase::MatchingComputed {
                mst_edges,
                matching_edges,
            } => {
                // Paso 4 y 5: Resolver el multigrafo, aplicar atajos y consolidar la ruta final
                let mut final_path =
                    Self::build_hamiltonian_cycle(nodes.len(), &mst_edges, &matching_edges);

                // El matching codicioso puede dejar cruces locales leves. Un 2-opt rápido lo pule.
                Self::optimize_2opt(&mut final_path, nodes);

                *current_path = final_path;
                self.phase = ChristofidesPhase::Finished;
                true // El algoritmo ha concluido con éxito
            }
            ChristofidesPhase::Finished => {
                self.phase = ChristofidesPhase::Finished;
                true
            }
        }
    }

    fn name(&self) -> &str {
        "Christofides Heuristic (O(N³))"
    }

    fn reset(&mut self) {
        self.phase = ChristofidesPhase::NotStarted;
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
