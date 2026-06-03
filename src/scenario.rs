/// Generador de escenarios de prueba para TSP
use crate::core::Node;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TestScenario {
    Manual,
    CirculoPerfecto,
    RejillaCuadrada,
    PuntosAleatorios8,
}

impl TestScenario {
    pub fn name(&self) -> &str {
        match self {
            TestScenario::Manual => "Manual (Click en pantalla)",
            TestScenario::CirculoPerfecto => "Test: Círculo Perfecto (Óptimo obvio)",
            TestScenario::RejillaCuadrada => "Test: Rejilla 4x4 (Óptimo simétrico)",
            TestScenario::PuntosAleatorios8 => "Test: Aleatorios 8 pts (Trampa Local)",
        }
    }
}

pub fn generate_scenario(
    scenario: TestScenario,
    screen_width: f32,
    screen_height: f32,
) -> Vec<Node> {
    let mut nodes = Vec::new();
    let cx = screen_width / 2.0;
    let cy = screen_height / 2.0;

    match scenario {
        TestScenario::Manual => {}
        TestScenario::CirculoPerfecto => {
            nodes = generate_circle(cx, cy, 12, 180.0);
        }
        TestScenario::RejillaCuadrada => {
            nodes = generate_grid(cx, cy, 4, 4, 100.0);
        }
        TestScenario::PuntosAleatorios8 => {
            // Escalar y centrar los puntos del benchmark
            let s = 15.0; // scale
            nodes = vec![
                Node::new(cx + 0.0 * s, cy + 0.0 * s),
                Node::new(cx + 10.0 * s, cy + 5.0 * s),
                Node::new(cx + 20.0 * s, cy + 0.0 * s),
                Node::new(cx + 15.0 * s, cy + 15.0 * s),
                Node::new(cx + 5.0 * s, cy + 15.0 * s),
                Node::new(cx + -5.0 * s, cy + 10.0 * s),
                Node::new(cx + -10.0 * s, cy + 0.0 * s),
                Node::new(cx + -5.0 * s, cy + -5.0 * s),
            ];
        }
    }
    nodes
}

/// Genera puntos en un círculo (desordenados intencionalmente)
fn generate_circle(cx: f32, cy: f32, num_points: usize, radius: f32) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut indices: Vec<usize> = (0..num_points).collect();

    // Desordenar para estresar al algoritmo
    indices.swap(1, 5);
    indices.swap(3, 9);
    indices.swap(2, 7);

    for i in indices {
        let angle = (i as f32) * (2.0 * std::f32::consts::PI / num_points as f32);
        nodes.push(Node::new(
            cx + angle.cos() * radius,
            cy + angle.sin() * radius,
        ));
    }
    nodes
}

/// Genera una rejilla cuadrada de puntos
fn generate_grid(cx: f32, cy: f32, cols: usize, rows: usize, spacing: f32) -> Vec<Node> {
    let mut nodes = Vec::new();
    let start_x = cx - (cols as f32 * spacing) / 2.0 + spacing / 2.0;
    let start_y = cy - (rows as f32 * spacing) / 2.0 + spacing / 2.0;

    for r in 0..rows {
        for c in 0..cols {
            nodes.push(Node::new(
                start_x + c as f32 * spacing,
                start_y + r as f32 * spacing,
            ));
        }
    }
    nodes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_names() {
        assert_eq!(TestScenario::Manual.name(), "Manual (Click en pantalla)");
        assert_eq!(
            TestScenario::CirculoPerfecto.name(),
            "Test: Círculo Perfecto (Óptimo obvio)"
        );
        assert_eq!(
            TestScenario::RejillaCuadrada.name(),
            "Test: Rejilla 4x4 (Óptimo simétrico)"
        );
        assert_eq!(
            TestScenario::PuntosAleatorios8.name(),
            "Test: Aleatorios 8 pts (Trampa Local)"
        );
    }

    #[test]
    fn test_generate_circle() {
        let nodes = generate_circle(100.0, 100.0, 12, 50.0);
        assert_eq!(nodes.len(), 12);
        for node in nodes {
            let dist = ((node.pos.x - 100.0).powi(2) + (node.pos.y - 100.0).powi(2)).sqrt();
            assert!((dist - 50.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_generate_grid() {
        let nodes = generate_grid(100.0, 100.0, 3, 3, 50.0);
        assert_eq!(nodes.len(), 9);

        // Verificar que están espaciados uniformemente
        let mut x_coords: Vec<_> = nodes.iter().map(|n| n.pos.x as i32).collect();
        x_coords.sort();
        x_coords.dedup();
        assert_eq!(x_coords.len(), 3); // 3 columnas diferentes
    }
}
