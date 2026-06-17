use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v9::{TriangleInsertionV9, V9Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

fn load_optima() -> HashMap<String, f64> {
    let mut optima = HashMap::new();
    optima.insert("berlin52".to_string(), 7542.0);
    optima.insert("eil51".to_string(), 426.0);
    optima.insert("st70".to_string(), 675.0);
    optima.insert("kroA100".to_string(), 21282.0);
    optima.insert("eil76".to_string(), 538.0);
    optima.insert("ch130".to_string(), 6110.0);
    optima.insert("pr76".to_string(), 108159.0);
    optima
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Entrenamiento Automático de Parámetros V9                 ║");
    println!("║     Recursive Edge Insertion                                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let instances_paths = vec![
        "assets/berlin52.tsp",
        "assets/eil51.tsp",
        "assets/st70.tsp",
        "assets/kroA100.tsp",
        "assets/eil76.tsp",
        "assets/ch130.tsp",
        "assets/pr76.tsp",
    ];

    let optima = load_optima();

    let mut instances = Vec::new();
    for path in &instances_paths {
        match TspInstance::from_file(path) {
            Ok(inst) => {
                println!("✓ Cargada: {} ({} nodos)", inst.name, inst.dimension);
                instances.push(inst);
            }
            Err(e) => {
                println!("✗ Error cargando {}: {}", path, e);
            }
        }
    }

    if instances.is_empty() {
        eprintln!("No se pudieron cargar instancias. Abortando.");
        return;
    }

    // Grid de búsqueda reducido para V9
    let k_values = vec![4, 6, 8];
    let angle_values = vec![0.20, 0.30, 0.40];
    let cost_values = vec![0.30, 0.40, 0.50, 0.60];
    let density_values = vec![0.00, 0.10, 0.20, 0.30];

    let total_combinations = k_values.len() * angle_values.len() * cost_values.len() * density_values.len();
    println!("\nGrid de búsqueda:");
    println!("  k_neighbors: {:?}", k_values);
    println!("  w_angle: {:?}", angle_values);
    println!("  w_cost: {:?}", cost_values);
    println!("  w_density: {:?}", density_values);
    println!("  Total combinaciones: {}\n", total_combinations);

    let start_total = Instant::now();
    let mut best_params = V9Params::default();
    let mut best_avg_error = f64::MAX;
    let mut results = Vec::new();
    let mut counter = 0;

    for &k in &k_values {
        for &a in &angle_values {
            for &c in &cost_values {
                for &d in &density_values {
                    counter += 1;
                    let params = V9Params {
                        k_neighbors: k,
                        w_angle: a,
                        w_cost: c,
                        w_density: d,
                    };

                    let mut total_error = 0.0;
                    let mut valid_instances = 0;

                    for instance in &instances {
                        let mut strategy = TriangleInsertionV9::with_params(params);
                        let mut path = Vec::new();
                        let mut steps = 0;

                        loop {
                            let finished = strategy.execute_step(&mut path, &instance.nodes);
                            steps += 1;
                            if finished || steps > instance.nodes.len() + 500 {
                                break;
                            }
                        }

                        let distance = path_distance(&path, &instance.nodes);

                        if let Some(&optimal) = optima.get(&instance.name) {
                            let error = ((distance - optimal as f32) / optimal as f32) * 100.0;
                            total_error += error as f64;
                            valid_instances += 1;
                        }
                    }

                    let avg_error = if valid_instances > 0 {
                        total_error / valid_instances as f64
                    } else {
                        f64::MAX
                    };

                    results.push((params, avg_error));

                    if avg_error < best_avg_error {
                        best_params = params;
                        best_avg_error = avg_error;
                        println!(
                            "[{}/{}] Nuevo mejor: k={} a={:.2} c={:.2} d={:.2} → Error: {:.2}%",
                            counter, total_combinations, k, a, c, d, avg_error
                        );
                    } else {
                        println!(
                            "[{}/{}] k={} a={:.2} c={:.2} d={:.2} → Error: {:.2}%",
                            counter, total_combinations, k, a, c, d, avg_error
                        );
                    }
                }
            }
        }
    }

    let total_time = start_total.elapsed();
    println!(
        "\n════════════════════════════════════════════════════════════════"
    );
    println!(
        "Entrenamiento completado en {:.2}s",
        total_time.as_secs_f64()
    );
    println!(
        "════════════════════════════════════════════════════════════════\n"
    );

    println!("Mejores parámetros encontrados:");
    println!("  k_neighbors: {}", best_params.k_neighbors);
    println!("  w_angle: {:.2}", best_params.w_angle);
    println!("  w_cost: {:.2}", best_params.w_cost);
    println!("  w_density: {:.2}", best_params.w_density);
    println!("  Error promedio: {:.2}%", best_avg_error);

    // Guardar resultados
    let mut file = File::create("assets/v9_training_results.txt").unwrap();
    writeln!(file, "# Resultados de entrenamiento V9").unwrap();
    writeln!(
        file,
        "# Tiempo total: {:.2}s",
        total_time.as_secs_f64()
    )
    .unwrap();
    writeln!(
        file,
        "# Total combinaciones evaluadas: {}",
        total_combinations
    )
    .unwrap();
    writeln!(file, "#\n# Mejores parámetros:").unwrap();
    writeln!(file, "k_neighbors: {}", best_params.k_neighbors).unwrap();
    writeln!(file, "w_angle: {:.2}", best_params.w_angle).unwrap();
    writeln!(file, "w_cost: {:.2}", best_params.w_cost).unwrap();
    writeln!(file, "w_density: {:.2}", best_params.w_density).unwrap();
    writeln!(file, "avg_error: {:.2}", best_avg_error).unwrap();
    writeln!(file, "\n# Todos los resultados:").unwrap();

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    for (i, (params, error)) in results.iter().enumerate() {
        writeln!(
            file,
            "#{}. k={} a={:.2} c={:.2} d={:.2} → Error: {:.2}%",
            i + 1,
            params.k_neighbors,
            params.w_angle,
            params.w_cost,
            params.w_density,
            error
        )
        .unwrap();
    }

    println!("\n✓ Resultados guardados en: assets/v9_training_results.txt");
}
