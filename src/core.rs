/// Tipos fundamentales y utilidades del proyecto TSP
use macroquad::prelude::Vec2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub pos: Vec2,
}

impl Node {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
        }
    }

    pub fn distance_to(&self, other: &Node) -> f32 {
        self.pos.distance(other.pos)
    }
}

/// Calcula la distancia total de un camino cerrado
pub fn path_distance(path: &[usize], nodes: &[Node]) -> f32 {
    if path.len() < 2 {
        return 0.0;
    }
    let mut dist = 0.0;
    for i in 0..path.len() {
        dist += nodes[path[i]].distance_to(&nodes[path[(i + 1) % path.len()]]);
    }
    dist
}

/// Calcula el costo de insertar un nodo en una arista
pub fn insertion_cost(edge_start: usize, edge_end: usize, node_idx: usize, nodes: &[Node]) -> f32 {
    let p_start = &nodes[edge_start];
    let p_end = &nodes[edge_end];
    let p_new = &nodes[node_idx];

    let new_cost = p_start.distance_to(p_new) + p_new.distance_to(p_end);
    let old_cost = p_start.distance_to(p_end);
    new_cost - old_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(10.0, 20.0);
        assert_eq!(node.pos.x, 10.0);
        assert_eq!(node.pos.y, 20.0);
    }

    #[test]
    fn test_distance_to() {
        let n1 = Node::new(0.0, 0.0);
        let n2 = Node::new(3.0, 4.0);
        assert!((n1.distance_to(&n2) - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_path_distance_empty() {
        let nodes = vec![Node::new(0.0, 0.0)];
        let path: Vec<usize> = vec![];
        assert_eq!(path_distance(&path, &nodes), 0.0);
    }

    #[test]
    fn test_path_distance_triangle() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
        ];
        let path = vec![0, 1, 2];
        let expected = 3.0 + 5.0 + 4.0; // 3-4-5 triangle
        assert!((path_distance(&path, &nodes) - expected).abs() < 0.001);
    }

    #[test]
    fn test_insertion_cost() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
            Node::new(1.5, 0.0),
        ];
        let cost = insertion_cost(0, 1, 3, &nodes);
        // Insertar nodo 3 (1.5, 0) entre 0 y 1
        // Costo nuevo: (0->1.5) + (1.5->3) = 1.5 + 1.5 = 3.0
        // Costo viejo: (0->3) = 3.0
        // Diferencia: 0
        assert!(cost.abs() < 0.001);
    }
}
