use std::time::Instant;
// Traemos las funciones y estructuras de tu propia librería 'traveler'
// NOTA: Ajusta estos nombres según cómo expongas tu V7, NN y cálculo de distancias en tu src/lib.rs
// use traveler::{solve_tsp_nn, solve_tsp_khl, KDTreeV7, solve_tsp_v7, calculate_tour_length};

// Estructura básica de un punto en 2D (Ajusta los nombres según uses Point, Node, etc.)
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

struct BenchResult {
    algo_name: String,
    time_ms: u128,
    tour_length: f32,
}

// =========================================================================
// PUNTO DE ENTRADA DEL TEST DE ESTRÉS
// =========================================================================

#[test]
#[ignore] // Esto evita que corra en un 'cargo test' simple sin flags.
fn test_estres_tsp_v7() {
    println!("\n==================================================");
    println!("      INICIANDO COMPETENCIA DE ESTRÉS TSP         ");
    println!("==================================================");

    // Evaluamos con un tamaño donde el O(N^2) de NN empieza a sufrir
    // y tu V7 O(N log N) debería empezar a sacar ventaja.
    let tamaño_estres = 2000;

    run_stress_test(tamaño_estres, "uniform");
    run_stress_test(tamaño_estres, "clustered");
    run_stress_test(tamaño_estres, "diagonal");
}

// =========================================================================
// ORQUESTADOR DE LA PRUEBA
// =========================================================================

fn run_stress_test(n_nodes: usize, map_type: &str) {
    println!(
        "\n>>> CONFIGURACIÓN: N = {} | Distribución: [{}] <<<",
        n_nodes,
        map_type.to_uppercase()
    );

    let points = match map_type {
        "clustered" => generate_clustered_points(n_nodes),
        "diagonal" => generate_diagonal_points(n_nodes),
        _ => generate_uniform_points(n_nodes),
    };

    let mut resultados = Vec::new();

    // ----------------------------------------------------
    // 1. NEAREST NEIGHBOR (NN)
    // ----------------------------------------------------
    let start = Instant::now();
    // let tour_nn = solve_tsp_nn(&points);
    let duration_nn = start.elapsed().as_millis();
    resultados.push(BenchResult {
        algo_name: "Nearest Neighbor (NN)".to_string(),
        time_ms: duration_nn,
        tour_length: 0.0, // calculate_tour_length(&tour_nn, &points),
    });

    // ----------------------------------------------------
    // 2. TU SOLUCIÓN V7 (KD-Tree + Ángulos)
    // ----------------------------------------------------
    let start = Instant::now();
    // let tree_v7 = KDTreeV7::build(&points);
    // let tour_v7 = solve_tsp_v7(&tree_v7, &points, 10); // k = 10 vecinos
    let duration_v7 = start.elapsed().as_millis();
    resultados.push(BenchResult {
        algo_name: "V7 (KD-Tree + Angles)".to_string(),
        time_ms: duration_v7,
        tour_length: 0.0, // calculate_tour_length(&tour_v7, &points),
    });

    // ----------------------------------------------------
    // 3. KHL (Lin-Kernighan Heuristic)
    // ----------------------------------------------------
    let start = Instant::now();
    // let tour_khl = solve_tsp_khl(&points);
    let duration_khl = start.elapsed().as_millis();
    resultados.push(BenchResult {
        algo_name: "KHL Heuristic".to_string(),
        time_ms: duration_khl,
        tour_length: 0.0, // calculate_tour_length(&tour_khl, &points),
    });

    // Imprimir los resultados de este escenario de manera limpia
    for res in resultados {
        println!(
            " |- {:<25} -> Tiempo: {:>4} ms | Distancia Total: {:.2}",
            res.algo_name, res.time_ms, res.tour_length
        );
    }
}

// =========================================================================
// GENERADORES SINTÉTICOS DE PUNTOS
// =========================================================================

fn simple_rand(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    (*seed % 10000) as f32 / 10000.0
}

pub fn generate_uniform_points(n: usize) -> Vec<Point> {
    let mut seed = 42;
    let mut points = Vec::with_capacity(n);
    for _ in 0..n {
        points.push(Point {
            x: simple_rand(&mut seed) * 1000.0,
            y: simple_rand(&mut seed) * 1000.0,
        });
    }
    points
}

pub fn generate_clustered_points(n: usize) -> Vec<Point> {
    let mut seed = 1234;
    let mut points = Vec::with_capacity(n);

    let num_clusters = 5;
    let mut centers = Vec::new();
    for _ in 0..num_clusters {
        centers.push(Point {
            x: simple_rand(&mut seed) * 1000.0,
            y: simple_rand(&mut seed) * 1000.0,
        });
    }

    for i in 0..n {
        let center = centers[i % num_clusters];
        let offset_x = (simple_rand(&mut seed) - 0.5) * 150.0;
        let offset_y = (simple_rand(&mut seed) - 0.5) * 150.0;

        points.push(Point {
            x: center.x + offset_x,
            y: center.y + offset_y,
        });
    }
    points
}

pub fn generate_diagonal_points(n: usize) -> Vec<Point> {
    let mut seed = 999;
    let mut points = Vec::with_capacity(n);
    for i in 0..n {
        let progress = (i as f32 / n as f32) * 1000.0;
        let noise = (simple_rand(&mut seed) - 0.5) * 2.0;

        points.push(Point {
            x: progress + noise,
            y: progress + noise,
        });
    }
    points
}
