use std::collections::HashMap;
use std::time::Instant;
use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
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
    println!("║     Comparación: V8.6 (Super Bubble) vs V8.6 (Original)       ║");
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

    let params = V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    println!("\n{:─<100}", "");
    println!("{:<12} {:>10} {:>12} {:>15} {:>15} {:>12}", "Instancia", "Nodos", "Óptimo", "V8.6 Original", "V8.6 Super", "Mejora %");
    println!("{:─<100}", "");

    let mut original_total_error = 0.0;
    let mut super_total_error = 0.0;
    let mut original_total_time = 0.0;
    let mut super_total_time = 0.0;

    for instance in &instances {
        let optimal = optima.get(&instance.name).copied().unwrap_or(0.0);

        // V8.6 Original (con 2-opt + or-opt + bubble)
        let start = Instant::now();
        let mut strategy_orig = TriangleInsertionV86::with_params(params);
        let mut path_orig = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy_orig.execute_step(&mut path_orig, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }
        let time_orig = start.elapsed().as_secs_f64();
        let dist_orig = path_distance(&path_orig, &instance.nodes);
        let error_orig = ((dist_orig - optimal as f32) / optimal as f32) * 100.0;

        // V8.6 Super Bubble (sin 2-opt separado)
        let start = Instant::now();
        let mut strategy_super = TriangleInsertionV86::with_params(params);
        let mut path_super = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy_super.execute_step(&mut path_super, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }
        let time_super = start.elapsed().as_secs_f64();
        let dist_super = path_distance(&path_super, &instance.nodes);
        let error_super = ((dist_super - optimal as f32) / optimal as f32) * 100.0;

        original_total_error += error_orig as f64;
        super_total_error += error_super as f64;
        original_total_time += time_orig;
        super_total_time += time_super;

        let improvement = ((error_orig - error_super) / error_orig) * 100.0;

        println!(
            "{:<12} {:>10} {:>12.0} {:>15.2} {:>15.2} {:>11.1}%",
            instance.name,
            instance.dimension,
            optimal,
            dist_orig,
            dist_super,
            improvement
        );
    }

    println!("{:─<100}", "");
    println!(
        "{:<12} {:>10} {:>12} {:>15.2} {:>15.2} {:>11.1}%",
        "PROMEDIO",
        "",
        "",
        original_total_error / instances.len() as f64,
        super_total_error / instances.len() as f64,
        ((original_total_error - super_total_error) / original_total_error) * 100.0
    );
    println!(
        "{:<12} {:>10} {:>12} {:>15.4}s {:>15.4}s {:>11.1}%",
        "TIEMPO",
        "",
        "",
        original_total_time / instances.len() as f64,
        super_total_time / instances.len() as f64,
        ((original_total_time - super_total_time) / original_total_time) * 100.0
    );

    let winner = if super_total_error < original_total_error {
        "V8.6 Super Bubble"
    } else if original_total_error < super_total_error {
        "V8.6 Original"
    } else {
        "Empate"
    };

    println!("\n════════════════════════════════════════════════════════════════");
    println!("Ganador: {} (Error: {:.2}% vs {:.2}%)", winner, original_total_error / instances.len() as f64, super_total_error / instances.len() as f64);
    println!("════════════════════════════════════════════════════════════════");
}
