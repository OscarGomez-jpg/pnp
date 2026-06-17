/// Perturbaciones guiadas para ILS.
use ::rand::seq::SliceRandom;

/// Double-bridge move: corta el tour en 4 puntos y reconecta cruzado.
/// Mantiene la mayoría de la estructura del tour mientras escapa de óptimos locales.
pub fn double_bridge(path: &[usize]) -> Vec<usize> {
    let n = path.len();
    if n < 8 {
        return path.to_vec();
    }

    let mut rng = ::rand::rng();
    let mut cuts: Vec<usize> = (1..n - 1).collect();
    cuts.shuffle(&mut rng);
    cuts.truncate(4);
    cuts.sort();

    let [a, b, c, d] = [cuts[0], cuts[1], cuts[2], cuts[3]];

    // A + D + C + B + E
    let mut new_path = Vec::with_capacity(n);
    new_path.extend_from_slice(&path[..a]);
    new_path.extend_from_slice(&path[c..d]);
    new_path.extend_from_slice(&path[b..c]);
    new_path.extend_from_slice(&path[a..b]);
    new_path.extend_from_slice(&path[d..]);
    new_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_bridge_preserves_nodes() {
        let path: Vec<usize> = (0..12).collect();
        let perturbed = double_bridge(&path);
        let mut sorted = perturbed.clone();
        sorted.sort();
        assert_eq!(sorted, path);
    }
}
