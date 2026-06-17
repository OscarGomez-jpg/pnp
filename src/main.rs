use macroquad::prelude::*;
use traveler::{
    core::{Node, path_distance},
    scenario::{TestScenario, generate_scenario},
    strategies::create_registry,
    ui::{self, AppState, RandomGeneratorState, UIConfig},
};

#[macroquad::main("TSP - Entorno de Pruebas Refactorizado")]
async fn main() {
    let mut nodes: Vec<Node> = Vec::new();
    let mut current_path: Vec<usize> = Vec::new();
    let mut state = AppState::Edit;

    let registry = create_registry();
    let mut current_strategy_id = "triangle_insertion";
    let mut current_strategy = registry.get_strategy(current_strategy_id).unwrap();

    let mut current_scenario = TestScenario::Manual;
    let ui_config = UIConfig::default();

    let mut last_step_time = 0.0;

    let available_strategies: Vec<&str> = registry.list_ids();
    let mut strategy_idx = 0;

    let mut random_state = RandomGeneratorState::default();
    let mut random_mode = "puntos";

    loop {
        clear_background(BLACK);
        let time = get_time();

        random_state.handle_input();

        if random_state.is_active {
            if ui::handle_confirm() {
                if let Some(count) = random_state.get_count() {
                    if count >= 3 {
                        let w = screen_width();
                        let h = screen_height();
                        let y_offset = ui_config.ui_height;
                        nodes = if random_mode == "clusters" {
                            ui::generate_cluster_points(count, w, h, y_offset)
                        } else {
                            ui::generate_random_points(count, w, h, y_offset)
                        };
                        current_path.clear();
                        current_scenario = TestScenario::Manual;
                        state = AppState::Edit;
                        current_strategy.reset();
                    }
                }
                random_state.is_active = false;
                random_state.input_text.clear();
            }
        } else {
            if matches!(state, AppState::Edit) && current_scenario == TestScenario::Manual {
                ui::handle_mouse_input(&mut nodes, ui_config.ui_height);
            }

            if ui::handle_strategy_switch() && matches!(state, AppState::Edit) {
                strategy_idx = (strategy_idx + 1) % available_strategies.len();
                current_strategy_id = available_strategies[strategy_idx];
                current_strategy = registry.get_strategy(current_strategy_id).unwrap();
                current_strategy.reset();
            }

            if ui::handle_scenario_switch() && matches!(state, AppState::Edit) {
                current_scenario = match current_scenario {
                    TestScenario::Manual => TestScenario::CirculoPerfecto,
                    TestScenario::CirculoPerfecto => TestScenario::RejillaCuadrada,
                    TestScenario::RejillaCuadrada => TestScenario::PuntosAleatorios8,
                    TestScenario::PuntosAleatorios8 => TestScenario::Manual,
                };
                nodes = generate_scenario(current_scenario, screen_width(), screen_height());
                current_path.clear();
            }

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
                        current_strategy.reset();
                    }
                }
            }

            if ui::handle_export() {
                if !nodes.is_empty() {
                    let filename = format!("tsp_solution_{}.txt", nodes.len());
                    match ui::export_nodes_to_txt(&nodes, &current_path, &filename) {
                        Ok(_) => {
                            println!("✓ Solución exportada a: {}", filename);
                        }
                        Err(e) => {
                            eprintln!(" Error exportando: {}", e);
                        }
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

            if ui::handle_random_generate() && matches!(state, AppState::Edit) {
                random_state.is_active = true;
                random_state.input_text.clear();
                random_mode = "puntos";
            }

            if ui::handle_cluster_generate() && matches!(state, AppState::Edit) {
                random_state.is_active = true;
                random_state.input_text.clear();
                random_mode = "clusters";
            }
        }

        if matches!(state, AppState::Running) && time - last_step_time > ui_config.step_delay as f64
        {
            last_step_time = time;
            let finished = current_strategy.execute_step(&mut current_path, &nodes);
            if finished {
                state = AppState::Finished;
            }
        }

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

        ui::render_random_input_dialog(&random_state, random_mode);

        next_frame().await
    }
}
