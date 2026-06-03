/// Tests de integración para el visualizador TSP
///
/// Pruebas end-to-end que verifican la correcta operación de los algoritmos
/// en escenarios reales de uso.
use traveler::{
    core::{Node, path_distance},
    scenario::{TestScenario, generate_scenario},
    strategies::create_registry,
};

#[test]
fn test_triangle_insertion_solves_simple_problem() {
    let registry = create_registry();
    let mut strategy = registry.get_strategy("triangle_insertion").unwrap();

    let nodes = vec![
        Node::new(0.0, 0.0),
        Node::new(3.0, 0.0),
        Node::new(0.0, 4.0),
        Node::new(3.0, 4.0),
    ];

    let mut path = vec![];
    let mut finished = false;

    for _ in 0..10 {
        finished = strategy.execute_step(&mut path, &nodes);
        if finished {
            break;
        }
    }

    assert!(finished, "Algorithm should complete");
    assert_eq!(path.len(), 4, "All nodes should be visited");

    // Verificar que es un ciclo válido
    let mut visited = vec![false; nodes.len()];
    for &idx in &path {
        assert!(!visited[idx], "Each node should be visited exactly once");
        visited[idx] = true;
    }
}

#[test]
fn test_nearest_neighbor_solves_simple_problem() {
    let registry = create_registry();
    let mut strategy = registry.get_strategy("nearest_neighbor").unwrap();

    let nodes = vec![
        Node::new(0.0, 0.0),
        Node::new(1.0, 0.0),
        Node::new(0.0, 1.0),
        Node::new(1.0, 1.0),
    ];

    let mut path = vec![];
    let mut finished = false;

    for _ in 0..10 {
        finished = strategy.execute_step(&mut path, &nodes);
        if finished {
            break;
        }
    }

    assert!(finished, "Algorithm should complete");
    assert_eq!(path.len(), 4, "All nodes should be visited");
}

#[test]
fn test_both_strategies_complete_circle_scenario() {
    let nodes = generate_scenario(TestScenario::CirculoPerfecto, 400.0, 400.0);
    assert!(!nodes.is_empty(), "Circle scenario should generate nodes");

    let registry = create_registry();

    for strategy_id in registry.list_ids() {
        let mut strategy = registry.get_strategy(strategy_id).unwrap();
        let mut path = vec![];
        let mut finished = false;

        for _ in 0..100 {
            finished = strategy.execute_step(&mut path, &nodes);
            if finished {
                break;
            }
        }

        assert!(
            finished,
            "Strategy {} should complete circle scenario",
            strategy_id
        );
        assert_eq!(path.len(), nodes.len(), "All nodes should be visited");
    }
}

#[test]
fn test_both_strategies_complete_grid_scenario() {
    let nodes = generate_scenario(TestScenario::RejillaCuadrada, 400.0, 400.0);
    assert_eq!(
        nodes.len(),
        16,
        "Grid scenario should generate 4x4 = 16 nodes"
    );

    let registry = create_registry();

    for strategy_id in registry.list_ids() {
        let mut strategy = registry.get_strategy(strategy_id).unwrap();
        let mut path = vec![];
        let mut finished = false;

        for _ in 0..100 {
            finished = strategy.execute_step(&mut path, &nodes);
            if finished {
                break;
            }
        }

        assert!(
            finished,
            "Strategy {} should complete grid scenario",
            strategy_id
        );
        assert_eq!(path.len(), nodes.len(), "All nodes should be visited");
    }
}

#[test]
fn test_path_distance_is_consistent() {
    let nodes = vec![
        Node::new(0.0, 0.0),
        Node::new(3.0, 0.0),
        Node::new(0.0, 4.0),
    ];

    let path1 = vec![0, 1, 2];
    let path2 = vec![0, 2, 1];

    let dist1 = path_distance(&path1, &nodes);
    let dist2 = path_distance(&path2, &nodes);

    // Ambas rutas deberían tener la misma distancia (triángulo perfecto)
    assert!(
        (dist1 - 12.0).abs() < 0.1,
        "Path [0,1,2] should have distance ~12"
    );
    assert!(
        (dist2 - 12.0).abs() < 0.1,
        "Path [0,2,1] should have distance ~12"
    );
}

#[test]
fn test_strategy_reset_works() {
    let registry = create_registry();
    let mut strategy = registry.get_strategy("triangle_insertion").unwrap();

    let nodes = vec![
        Node::new(0.0, 0.0),
        Node::new(1.0, 0.0),
        Node::new(0.0, 1.0),
    ];

    // Ejecutar una vez
    let mut path1 = vec![];
    strategy.execute_step(&mut path1, &nodes);
    assert_eq!(path1, vec![0, 1, 2]);

    // Reset
    strategy.reset();

    // Ejecutar de nuevo
    let mut path2 = vec![];
    strategy.execute_step(&mut path2, &nodes);
    assert_eq!(path2, vec![0, 1, 2], "Path should be identical after reset");
}

#[test]
fn test_multiple_strategies_in_registry() {
    let registry = create_registry();
    let ids = registry.list_ids();

    assert!(
        ids.len() >= 2,
        "Should have at least 2 strategies registered"
    );
    assert!(
        ids.contains(&"triangle_insertion"),
        "Triangle insertion should be registered"
    );
    assert!(
        ids.contains(&"nearest_neighbor"),
        "Nearest neighbor should be registered"
    );
}

#[test]
fn test_strategy_names_are_descriptive() {
    let registry = create_registry();
    let names = registry.list_names();

    assert!(!names.is_empty(), "Should have strategy names");

    for name in names {
        assert!(!name.is_empty(), "Strategy names should not be empty");
        assert!(name.len() > 5, "Strategy names should be descriptive");
    }
}

#[test]
fn test_node_positioning_in_scenarios() {
    // Verificar que los nodos del círculo están correctamente posicionados
    let circle_nodes = generate_scenario(TestScenario::CirculoPerfecto, 400.0, 400.0);
    assert!(
        !circle_nodes.is_empty(),
        "Circle scenario should generate nodes"
    );

    for node in &circle_nodes {
        // Verificar que tienen posiciones numéricas válidas
        assert!(node.pos.x.is_finite(), "X coordinate should be finite");
        assert!(node.pos.y.is_finite(), "Y coordinate should be finite");
    }

    // Verificar que los nodos de la rejilla están correctamente posicionados
    let grid_nodes = generate_scenario(TestScenario::RejillaCuadrada, 400.0, 400.0);
    assert_eq!(grid_nodes.len(), 16, "Grid should have 4x4=16 nodes");

    for node in &grid_nodes {
        assert!(node.pos.x.is_finite(), "X coordinate should be finite");
        assert!(node.pos.y.is_finite(), "Y coordinate should be finite");
    }
}
