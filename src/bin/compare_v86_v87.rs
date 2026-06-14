use std::collections::HashMap;
use std::time::Instant;
use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::triangle_insertion_v8_7::{TriangleInsertionV87, V87Params};
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
    println!("║     Comparación Detallada: V8.6 vs V8.7                       ║");
    println!("════════════════════════════════════════════════════════════════╝\n");

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

    // Parámetros calibrados
    let v86_params = V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    let v87_params = V87Params {
        k_neighbors: 4,
        w_angle: 0.25,
        w_cost: 0.75,
    };

    println!("\n{:─<100}", "");
    println!("{:<12} {:>10} {:>12} {:>12} {:>12} {:>12} {:>12}", "Instancia", "Nodos", "Óptimo", "V8.6 Dist", "V8.7 Dist", "V8.6 Err", "V8.7 Err");
    println!("{:─<100}", "");

    let mut v86_total_error = 0.0;
    let mut v87_total_error = 0.0;
    let mut v86_total_time = 0.0;
    let mut v87_total_time = 0.0;

    for instance in &instances {
        let optimal = optima.get(&instance.name).copied().unwrap_or(0.0);

        // V8.6
        let start = Instant::now();
        let mut strategy_v86 = TriangleInsertionV86::with_params(v86_params);
        let mut path_v86 = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy_v86.execute_step(&mut path_v86, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }
        let time_v86 = start.elapsed().as_secs_f64();
        let dist_v86 = path_distance(&path_v86, &instance.nodes);
        let error_v86 = ((dist_v86 - optimal as f32) / optimal as f32) * 100.0;

        // V8.7
        let start = Instant::now();
        let mut strategy_v87 = TriangleInsertionV87::with_params(v87_params);
        let mut path_v87 = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy_v87.execute_step(&mut path_v87, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }
        let time_v87 = start.elapsed().as_secs_f64();
        let dist_v87 = path_distance(&path_v87, &instance.nodes);
        let error_v87 = ((dist_v87 - optimal as f32) / optimal as f32) * 100.0;

        v86_total_error += error_v86 as f64;
        v87_total_error += error_v87 as f64;
        v86_total_time += time_v86;
        v87_total_time += time_v87;

        println!(
            "{:<12} {:>10} {:>12.0} {:>12.2} {:>12.2} {:>11.2}% {:>11.2}%",
            instance.name,
            instance.dimension,
            optimal,
            dist_v86,
            dist_v87,
            error_v86,
            error_v87
        );
    }

    println!("{:─<100}", "");
    println!(
        "{:<12} {:>10} {:>12} {:>12} {:>12} {:>11.2}% {:>11.2}%",
        "PROMEDIO",
        "",
        "",
        "",
        "",
        v86_total_error / instances.len() as f64,
        v87_total_error / instances.len() as f64
    );
    println!(
        "{:<12} {:>10} {:>12} {:>12} {:>12} {:>11.4}s {:>11.4}s",
        "TIEMPO",
        "",
        "",
        "",
        "",
        v86_total_time / instances.len() as f64,
        v87_total_time / instances.len() as f64
    );

    let winner = if v86_total_error < v87_total_error {
        "V8.6"
    } else if v87_total_error < v86_total_error {
        "V8.7"
    } else {
        "Empate"
    };

    println!("\n════════════════════════════════════════════════════════════════");
    println!("Ganador: {} (Error: {:.2}% vs {:.2}%)", winner, v86_total_error / instances.len() as f64, v87_total_error / instances.len() as f64);
    println!("════════════════════════════════════════════════════════════════");
}
