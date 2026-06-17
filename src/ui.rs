use crate::core::Node;
/// Módulo de interfaz de usuario y renderizado
use macroquad::prelude::*;
use ::rand::RngExt;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AppState {
    Edit,
    Running,
    Finished,
}

pub struct UIConfig {
    pub step_delay: f32,
    pub ui_height: f32,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            step_delay: 0.3,
            ui_height: 140.0,
        }
    }
}

pub struct RandomGeneratorState {
    pub input_text: String,
    pub is_active: bool,
}

impl Default for RandomGeneratorState {
    fn default() -> Self {
        Self {
            input_text: String::new(),
            is_active: false,
        }
    }
}

impl RandomGeneratorState {
    pub fn handle_input(&mut self) {
        if !self.is_active {
            return;
        }

        if let Some(key) = get_last_key_pressed() {
            match key {
                KeyCode::Key0 => self.input_text.push('0'),
                KeyCode::Key1 => self.input_text.push('1'),
                KeyCode::Key2 => self.input_text.push('2'),
                KeyCode::Key3 => self.input_text.push('3'),
                KeyCode::Key4 => self.input_text.push('4'),
                KeyCode::Key5 => self.input_text.push('5'),
                KeyCode::Key6 => self.input_text.push('6'),
                KeyCode::Key7 => self.input_text.push('7'),
                KeyCode::Key8 => self.input_text.push('8'),
                KeyCode::Key9 => self.input_text.push('9'),
                KeyCode::Backspace => {
                    self.input_text.pop();
                }
                KeyCode::Escape => {
                    self.is_active = false;
                    self.input_text.clear();
                }
                _ => {}
            }
        }
    }

    pub fn get_count(&self) -> Option<usize> {
        self.input_text.parse().ok()
    }
}

fn get_last_key_pressed() -> Option<KeyCode> {
    let keys = [
        KeyCode::Key0,
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
        KeyCode::Backspace,
        KeyCode::Escape,
    ];
    for key in keys {
        if is_key_pressed(key) {
            return Some(key);
        }
    }
    None
}

pub fn generate_random_points(count: usize, width: f32, height: f32, y_offset: f32) -> Vec<Node> {
    let mut rng = ::rand::rng();
    let mut nodes = Vec::with_capacity(count);
    for _ in 0..count {
        let x = rng.random_range(0.0..width);
        let y = rng.random_range(y_offset..height);
        nodes.push(Node::new(x, y));
    }
    nodes
}

pub fn generate_cluster_points(count: usize, width: f32, height: f32, y_offset: f32) -> Vec<Node> {
    let mut rng = ::rand::rng();
    let num_clusters = (count as f32).sqrt().ceil() as usize;
    let mut nodes = Vec::with_capacity(count);

    let area_height = height - y_offset;

    let mut cluster_centers: Vec<Vec2> = Vec::new();
    for _ in 0..num_clusters {
        cluster_centers.push(Vec2::new(
            rng.random_range(0.0..width),
            rng.random_range(y_offset..height),
        ));
    }

    let spread = (width.max(area_height) / num_clusters as f32) * 0.5;

    for i in 0..count {
        let center = cluster_centers[i % num_clusters];
        let x = center.x + rng.random_range(-spread..spread);
        let y = center.y + rng.random_range(-spread..spread);
        nodes.push(Node::new(x.clamp(0.0, width), y.clamp(y_offset, height)));
    }
    nodes
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
        "[R] Generar N puntos aleatorios  |  [G] Generar clusters",
        20.0,
        screen_height() - 60.0,
        16.0,
        DARKGRAY,
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

pub fn render_random_input_dialog(state: &RandomGeneratorState, mode: &str) {
    if !state.is_active {
        return;
    }

    let screen_w = screen_width();
    let screen_h = screen_height();
    let dialog_w = 400.0;
    let dialog_h = 150.0;
    let x = (screen_w - dialog_w) / 2.0;
    let y = (screen_h - dialog_h) / 2.0;

    draw_rectangle(x, y, dialog_w, dialog_h, Color::new(0.15, 0.15, 0.15, 0.95));
    draw_rectangle_lines(x, y, dialog_w, dialog_h, 2.0, WHITE);

    draw_text(
        &format!("Generar {} - Ingresa cantidad:", mode),
        x + 20.0,
        y + 35.0,
        18.0,
        WHITE,
    );

    let input_x = x + 20.0;
    let input_y = y + 55.0;
    let input_w = dialog_w - 40.0;
    let input_h = 35.0;

    draw_rectangle(input_x, input_y, input_w, input_h, Color::new(0.1, 0.1, 0.1, 1.0));
    draw_rectangle_lines(input_x, input_y, input_w, input_h, 2.0, YELLOW);

    let display_text = if state.input_text.is_empty() {
        "0".to_string()
    } else {
        state.input_text.clone()
    };
    draw_text(&display_text, input_x + 10.0, input_y + 25.0, 24.0, WHITE);

    draw_text(
        "[ENTER] Generar  |  [ESC] Cancelar",
        x + 20.0,
        y + 120.0,
        14.0,
        LIGHTGRAY,
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

/// Detecta generación de puntos aleatorios con [R]
pub fn handle_random_generate() -> bool {
    is_key_pressed(KeyCode::R)
}

/// Detecta generación de clusters con [G]
pub fn handle_cluster_generate() -> bool {
    is_key_pressed(KeyCode::G)
}

/// Detecta confirmación con [ENTER]
pub fn handle_confirm() -> bool {
    is_key_pressed(KeyCode::Enter)
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
