use macroquad::prelude::*;
use traveler::{
    core::{Node, path_distance},
    scenario::{TestScenario, generate_scenario},
    strategies::create_registry,
    ui::{self, AppState, UIConfig},
};

#[macroquad::main("TSP - Entorno de Pruebas Refactorizado")]
async fn main() {
    let mut nodes: Vec<Node> = Vec::new();
    let mut current_path: Vec<usize> = Vec::new();
    let mut state = AppState::Edit;

    // Crear registry de estrategias
    let registry = create_registry();
    let mut current_strategy_id = "triangle_insertion";
    let mut current_strategy = registry.get_strategy(current_strategy_id).unwrap();

    let mut current_scenario = TestScenario::Manual;
    let ui_config = UIConfig::default();

    let mut last_step_time = 0.0;

    // Iterador de estrategias
    let available_strategies: Vec<&str> = registry.list_ids();
    let mut strategy_idx = 0;

    loop {
        clear_background(BLACK);
        let time = get_time();

        // ==========================================
        // MANEJO DE ENTRADAS Y MENÚS
        // ==========================================
        if matches!(state, AppState::Edit) && current_scenario == TestScenario::Manual {
            ui::handle_mouse_input(&mut nodes, ui_config.ui_height);
        }

        // Cambiar de Estrategia con [E]
        if ui::handle_strategy_switch() && matches!(state, AppState::Edit) {
            strategy_idx = (strategy_idx + 1) % available_strategies.len();
            current_strategy_id = available_strategies[strategy_idx];
            current_strategy = registry.get_strategy(current_strategy_id).unwrap();
            current_strategy.reset();
        }

        // Cambiar de Escenario de Test con [T]
        if ui::handle_scenario_switch() && matches!(state, AppState::Edit) {
            current_scenario = match current_scenario {
                TestScenario::Manual => TestScenario::CirculoPerfecto,
                TestScenario::CirculoPerfecto => TestScenario::RejillaCuadrada,
                TestScenario::RejillaCuadrada => TestScenario::Manual,
            };
            nodes = generate_scenario(current_scenario, screen_width(), screen_height());
            current_path.clear();
        }

        // Control de ejecución [ESPACIO]
        if ui::handle_execution_toggle() {
            match state {
                AppState::Edit => {
                    if nodes.len() >= 3 {
                        state = AppState::Running;
                        current_path.clear();
                        last_step_time = time;
                    }
                }
                AppState::Running | AppState::Finished => {
                    state = AppState::Edit;
                    current_path.clear();
                    nodes = generate_scenario(current_scenario, screen_width(), screen_height());
                    current_strategy.reset();
                }
            }
        }

        if ui::handle_reset() {
            current_scenario = TestScenario::Manual;
            nodes.clear();
            current_path.clear();
            state = AppState::Edit;
            current_strategy.reset();
        }

        // ==========================================
        // EJECUCIÓN DEL ALGORITMO SELECCIONADO
        // ==========================================
        if matches!(state, AppState::Running) && time - last_step_time > ui_config.step_delay as f64
        {
            last_step_time = time;
            let finished = current_strategy.execute_step(&mut current_path, &nodes);
            if finished {
                state = AppState::Finished;
            }
        }

        // ==========================================
        // RENDERS
        // ==========================================
        ui::render_graph(
            &nodes,
            &current_path,
            matches!(state, AppState::Finished),
            current_strategy_id,
        );

        ui::render_hud(
            state,
            current_strategy.name(),
            current_scenario.name(),
            nodes.len(),
            path_distance(&current_path, &nodes),
        );

        next_frame().await
    }
}
