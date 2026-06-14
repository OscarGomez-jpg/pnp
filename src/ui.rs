use crate::core::Node;
/// Módulo de interfaz de usuario y renderizado
use macroquad::prelude::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AppState {
    Edit,
    Running,
    Finished,
}

pub struct UIConfig {
    pub step_delay: f32, // Segundos entre pasos
    pub ui_height: f32,  // Altura del HUD
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            step_delay: 0.3,
            ui_height: 140.0,
        }
    }
}

/// Renderiza la interfaz de usuario (HUD)
pub fn render_hud(
    state: AppState,
    strategy_name: &str,
    scenario_name: &str,
    nodes_count: usize,
    total_distance: f32,
) {
    let status_color = match state {
        AppState::Edit => YELLOW,
        AppState::Running => ORANGE,
        AppState::Finished => GREEN,
    };

    draw_text(
        "=== CONTROLES DE EXPERIMENTACIÓN ===",
        20.0,
        25.0,
        18.0,
        WHITE,
    );
    draw_text(
        &format!(" [E] Estrategia actual: {}", strategy_name),
        20.0,
        50.0,
        18.0,
        LIGHTGRAY,
    );
    draw_text(
        &format!(" [T] Escenario de Test: {}", scenario_name),
        20.0,
        75.0,
        18.0,
        LIGHTGRAY,
    );

    let state_text = match state {
        AppState::Edit => "ESTADO: CONFIGURACIÓN | [ESPACIO] para ejecutar",
        AppState::Running => "ESTADO: EJECUTANDO PASO A PASO...",
        AppState::Finished => "ESTADO: FINALIZADO | [ESPACIO] para reiniciar/editar",
    };
    draw_text(state_text, 20.0, 105.0, 18.0, status_color);

    draw_text(
        &format!(
            "Ciudades: {} | Distancia: {:.2} px",
            nodes_count, total_distance
        ),
        20.0,
        130.0,
        18.0,
        WHITE,
    );
    draw_text(
        "[C] Resetear a modo Manual vacio",
        20.0,
        screen_height() - 40.0,
        16.0,
        DARKGRAY,
    );
    draw_text(
        "[X] Exportar solución a TXT",
        20.0,
        screen_height() - 20.0,
        16.0,
        DARKGRAY,
    );
}

/// Renderiza los nodos y el camino
pub fn render_graph(nodes: &[Node], path: &[usize], is_finished: bool, strategy_id: &str) {
    // Dibujar líneas del camino
    if path.len() >= 2 {
        for i in 0..path.len() {
            // En vecino más cercano, solo cerramos el ciclo al final
            if strategy_id == "nearest_neighbor" && !is_finished && i == path.len() - 1 {
                continue;
            }
            let n1 = nodes[path[i]];
            let n2 = nodes[path[(i + 1) % path.len()]];
            draw_line(n1.pos.x, n1.pos.y, n2.pos.x, n2.pos.y, 3.0, GREEN);
        }
    }

    // Dibujar nodos
    for (idx, node) in nodes.iter().enumerate() {
        let color = if path.contains(&idx) { RED } else { GRAY };
        draw_circle(node.pos.x, node.pos.y, 7.0, color);
        draw_circle_lines(node.pos.x, node.pos.y, 10.0, 1.5, WHITE);
    }
}

/// Detecta input del mouse en modo edición manual
pub fn handle_mouse_input(nodes: &mut Vec<Node>, ui_height: f32) {
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if my > ui_height {
            // Click fuera del HUD
            nodes.push(Node::new(mx, my));
        }
    }
}

/// Detecta cambio de estrategia con [E]
pub fn handle_strategy_switch() -> bool {
    is_key_pressed(KeyCode::E)
}

/// Detecta cambio de escenario con [T]
pub fn handle_scenario_switch() -> bool {
    is_key_pressed(KeyCode::T)
}

/// Detecta control de ejecución con [ESPACIO]
pub fn handle_execution_toggle() -> bool {
    is_key_pressed(KeyCode::Space)
}

/// Detecta reset con [C]
pub fn handle_reset() -> bool {
    is_key_pressed(KeyCode::C)
}

/// Detecta exportación con [X]
pub fn handle_export() -> bool {
    is_key_pressed(KeyCode::X)
}

/// Exporta las coordenadas de los nodos a un archivo TXT
pub fn export_nodes_to_txt(nodes: &[Node], path: &[usize], filename: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(filename)?;

    writeln!(file, "# TSP Solution Export")?;
    writeln!(file, "# Total nodes: {}", nodes.len())?;
    writeln!(file, "# Path length: {}", path.len())?;
    writeln!(file, "")?;

    writeln!(file, "# Node coordinates (index x y)")?;
    for (idx, node) in nodes.iter().enumerate() {
        writeln!(file, "{} {:.2} {:.2}", idx, node.pos.x, node.pos.y)?;
    }

    writeln!(file, "")?;
    writeln!(file, "# Solution path (node indices in order)")?;
    for &node_idx in path {
        writeln!(file, "{}", node_idx)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_config_default() {
        let config = UIConfig::default();
        assert_eq!(config.step_delay, 0.3);
        assert_eq!(config.ui_height, 140.0);
    }

    #[test]
    fn test_app_state_equality() {
        assert_eq!(AppState::Edit, AppState::Edit);
        assert_ne!(AppState::Edit, AppState::Running);
    }
}
