/// Búsqueda local optimizada para V9+ILS.
///
/// Usa listas de candidatos (k vecinos más cercanos) para acotar el vecindario
/// de 2-opt, reinsertión y or-opt.
use crate::core::Node;
use crate::strategies::triangle_insertion_v9::KDTree;
use macroquad::prelude::Vec2;

#[derive(Clone, Copy)]
pub struct LocalSearchConfig {
    pub candidate_size: usize,
    pub max_2opt_rounds: usize,
    pub max_reinsert_rounds: usize,
    pub max_oropt_rounds: usize,
}

impl Default for LocalSearchConfig {
    fn default() -> Self {
        Self {
            candidate_size: 20,
            max_2opt_rounds: 5,
            max_reinsert_rounds: 2,
            max_oropt_rounds: 1,
        }
    }
}

pub struct LocalSearcher<'a> {
    nodes: &'a [Node],
    candidates: Vec<Vec<usize>>,
    config: LocalSearchConfig,
}

impl<'a> LocalSearcher<'a> {
    pub fn new(nodes: &'a [Node], config: LocalSearchConfig) -> Self {
        let candidates = build_candidate_lists(nodes, config.candidate_size);
        Self {
            nodes,
            candidates,
            config,
        }
    }

    pub fn optimize(&self, path: &mut Vec<usize>) {
        for _ in 0..self.config.max_2opt_rounds {
            if !self.two_opt(path) {
                break;
            }
        }

        for seg_len in [1usize, 2, 3] {
            for _ in 0..self.config.max_oropt_rounds {
                if !self.or_opt(path, seg_len) {
                    break;
                }
            }
        }

        for _ in 0..self.config.max_reinsert_rounds {
            if !self.node_reinsertion(path) {
                break;
            }
        }

        for _ in 0..2 {
            if !self.two_opt(path) {
                break;
            }
        }
    }

    fn dist(&self, a: usize, b: usize) -> f32 {
        self.nodes[a].pos.distance(self.nodes[b].pos)
    }

    fn pos_in_tour(path: &[usize]) -> Vec<usize> {
        let mut pos = vec![0; path.len()];
        for (p, &node) in path.iter().enumerate() {
            pos[node] = p;
        }
        pos
    }

    fn two_opt(&self, path: &mut Vec<usize>) -> bool {
        let n = path.len();
        if n < 4 {
            return false;
        }

        let mut improved = false;
        loop {
            let pos = Self::pos_in_tour(path);
            let mut local_improved = false;

            'outer: for i in 0..n {
                let a = path[i];
                let b = path[(i + 1) % n];

                let mut cand = std::collections::HashSet::new();
                for &nb in &self.candidates[a] {
                    cand.insert(nb);
                }
                for &nb in &self.candidates[b] {
                    cand.insert(nb);
                }

                for &c in &cand {
                    let j = pos[c];
                    if j == i || j == (i + 1) % n || i == (j + 1) % n {
                        continue;
                    }
                    let d = path[(j + 1) % n];

                    let cur = self.dist(a, b) + self.dist(c, d);
                    let swapped = self.dist(a, c) + self.dist(b, d);
                    if swapped < cur - 1e-4 {
                        Self::reverse_segment(path, (i + 1) % n, j);
                        local_improved = true;
                        break 'outer;
                    }
                }
            }

            if !local_improved {
                break;
            }
            improved = true;
        }
        improved
    }

    fn reverse_segment(path: &mut Vec<usize>, start: usize, end: usize) {
        let n = path.len();
        let mut i = start;
        let mut j = end;
        while i != j && (i + n - 1) % n != j {
            path.swap(i, j);
            i = (i + 1) % n;
            j = (j + n - 1) % n;
        }
    }

    fn node_reinsertion(&self, path: &mut Vec<usize>) -> bool {
        let n = path.len();
        if n < 4 {
            return false;
        }

        let mut ever_improved = false;
        loop {
            let pos = Self::pos_in_tour(path);
            let mut local_improved = false;

            'outer: for i in 0..n {
                let node = path[i];
                let prev = path[(i + n - 1) % n];
                let next = path[(i + 1) % n];
                let removal = self.dist(prev, node) + self.dist(node, next) - self.dist(prev, next);

                let mut best_j = i;
                let mut best_delta = 0.0f32;

                for &nb in &self.candidates[node] {
                    let p = pos[nb];
                    for j in [p, (p + n - 1) % n] {
                        if j == i || j == (i + n - 1) % n {
                            continue;
                        }
                        let r_prev = path[j];
                        let r_next = path[(j + 1) % n];
                        let ins = self.dist(r_prev, node) + self.dist(node, r_next) - self.dist(r_prev, r_next);
                        let delta = ins - removal;
                        if delta < best_delta - 1e-4 {
                            best_delta = delta;
                            best_j = j;
                        }
                    }
                }

                if best_j != i {
                    let node = path.remove(i);
                    let insert_at = if best_j > i { best_j } else { best_j };
                    path.insert(insert_at, node);
                    local_improved = true;
                    ever_improved = true;
                    break 'outer;
                }
            }

            if !local_improved {
                break;
            }
        }
        ever_improved
    }

    fn or_opt(&self, path: &mut Vec<usize>, seg_len: usize) -> bool {
        let n = path.len();
        if n < seg_len + 2 {
            return false;
        }

        let mut ever_improved = false;
        loop {
            let mut local_improved = false;

            'outer: for i in 0..n {
                let prev = path[(i + n - 1) % n];
                let first = path[i];
                let last = path[(i + seg_len - 1) % n];
                let next = path[(i + seg_len) % n];
                let removal = self.dist(prev, first) + self.dist(last, next) - self.dist(prev, next);

                let mut reduced = Vec::with_capacity(n - seg_len);
                for k in 0..n {
                    let mut in_seg = false;
                    for offset in 0..seg_len {
                        if k == (i + offset) % n {
                            in_seg = true;
                            break;
                        }
                    }
                    if !in_seg {
                        reduced.push(path[k]);
                    }
                }

                let m = reduced.len();
                let mut best_j = 0usize;
                let mut best_ins = f32::MAX;

                for &nb in &self.candidates[first] {
                    let p = reduced.iter().position(|&x| x == nb).unwrap_or(0);
                    for j in [p, (p + m - 1) % m] {
                        let r_prev = reduced[(j + m - 1) % m];
                        let r_next = reduced[j % m];
                        let ins = self.dist(r_prev, first) + self.dist(last, r_next) - self.dist(r_prev, r_next);
                        if ins < best_ins - 1e-4 {
                            best_ins = ins;
                            best_j = j;
                        }
                    }
                }

                if best_ins < removal - 1e-4 {
                    let seg: Vec<usize> = (0..seg_len).map(|k| path[(i + k) % n]).collect();
                    let mut new_path = Vec::with_capacity(n);
                    new_path.extend_from_slice(&reduced[..best_j]);
                    new_path.extend_from_slice(&seg);
                    new_path.extend_from_slice(&reduced[best_j..]);
                    *path = new_path;
                    local_improved = true;
                    ever_improved = true;
                    break 'outer;
                }
            }

            if !local_improved {
                break;
            }
        }
        ever_improved
    }
}

fn build_candidate_lists(nodes: &[Node], k: usize) -> Vec<Vec<usize>> {
    let points: Vec<(Vec2, usize)> = nodes.iter().enumerate().map(|(i, n)| (n.pos, i)).collect();
    let kdtree = KDTree::build(&points);

    (0..nodes.len())
        .map(|i| {
            let mut neighbors = kdtree.find_k_nearest(nodes[i].pos, k + 1);
            neighbors.retain(|&x| x != i);
            if neighbors.len() > k {
                neighbors.truncate(k);
            }
            neighbors
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Node;
    use crate::core::path_distance;

    #[test]
    fn test_two_opt_fixes_crossing() {
        let nodes = vec![
            Node::new(0.0, 10.0),
            Node::new(10.0, 10.0),
            Node::new(5.0, 5.0),
            Node::new(0.0, 0.0),
            Node::new(5.0, 0.0),
            Node::new(10.0, 0.0),
        ];

        let mut path = vec![0, 1, 2, 3, 4, 5];
        let before = path_distance(&path, &nodes);

        let ls = LocalSearcher::new(&nodes, LocalSearchConfig::default());
        ls.optimize(&mut path);

        let after = path_distance(&path, &nodes);
        assert!(after < before, "2-opt debería reducir distancia");
    }
}
