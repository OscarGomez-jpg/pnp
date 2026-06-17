use std::collections::HashMap;
use std::time::Instant;
use traveler::core::{path_distance, Node};
use traveler::strategies::create_registry;
use traveler::tsplib::TspInstance;

fn get_state_of_the_art() -> HashMap<&'static str, (f64, f64)> {
    let mut refs = HashMap::new();
    refs.insert("berlin52", (7542.0, 7542.0));
    refs.insert("eil51", (426.0, 426.0));
    refs.insert("st70", (675.0, 675.0));
    refs.insert("kroA100", (21282.0, 21282.0));
    refs.insert("eil76", (538.0, 538.0));
    refs.insert("ch130", (6110.0, 6110.0));
    refs.insert("pr76", (108159.0, 108159.0));
    refs
}

fn load_optima() -> HashMap<String, f64> {
    let mut optima = HashMap::new();
    if let Ok(content) = std::fs::read_to_string("assets/optima.txt") {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(val) = parts[1].parse::<f64>() {
                    optima.insert(parts[0].to_string(), val);
                }
            }
        }
    }
    optima
}

fn main() {
    let registry = create_registry();
    let optima = load_optima();
    let state_of_art = get_state_of_the_art();

    let instances = vec![
        "assets/berlin52.tsp",
        "assets/eil51.tsp",
        "assets/st70.tsp",
        "assets/kroA100.tsp",
        "assets/eil76.tsp",
        "assets/ch130.tsp",
        "assets/pr76.tsp",
    ];

    println!("\n=== TSP BENCHMARK: COMPARACION VS ESTADO DEL ARTE ===");
    println!("Referencias:");
    println!("  - Optimo: Solucion optima conocida (TSPLIB/Concorde)");
    println!("  - LKH: Lin-Kernighan-Helsgaun (mejor heuristica, ~0-2% error)");
    println!("  - Concorde: Solver exacto (optimo garantizado)\n");

    println!(
        "{:<12} {:<6} {:<35} {:<12} {:<10} {:<10} {:<10}",
        "Instance", "N", "Strategy", "Distance", "Time(ms)", "%Error", "vs LKH"
    );
    println!("{}", "-".repeat(100));

    let strategy_ids = vec![
        "lin_kernighan",
        "triangle_insertion_v8_9",
        "triangle_insertion_v9",
        "triangle_insertion_v9_hybrid",
        "triangle_insertion_v9_ils",
    ];

    for instance_path in &instances {
        let instance = match TspInstance::from_file(instance_path) {
            Ok(inst) => inst,
            Err(e) => {
                eprintln!("Error loading {}: {}", instance_path, e);
                continue;
            }
        };

        let optimal = optima.get(&instance.name).copied();
        let (lkh_dist, _concorde_dist) = state_of_art
            .get(instance.name.as_str())
            .copied()
            .unwrap_or((0.0, 0.0));

        for id in &strategy_ids {
            let mut strategy = match registry.get_strategy(id) {
                Some(s) => s,
                None => continue,
            };
            
            if *id == "triangle_insertion_v8_5" {
                if let Some(mut s) = registry.get_strategy(id) {
                    if let Some(v85) = s.as_any_mut().downcast_mut::<traveler::strategies::triangle_insertion_v8_5::TriangleInsertionV85>() {
                        if v85.load_calibrated_params("assets/v85_calibrated_params.txt") {
                            strategy = s;
                        }
                    }
                }
            }
            
            let mut path = Vec::new();

            let start = Instant::now();
            let mut steps = 0;
            let max_steps = if *id == "lin_kernighan" {
                instance.nodes.len() * 20
            } else {
                instance.nodes.len() + 500
            };
            loop {
                let finished = strategy.execute_step(&mut path, &instance.nodes);
                steps += 1;
                if finished || steps > max_steps {
                    break;
                }
            }
            let duration = start.elapsed();
            let distance = path_distance(&path, &instance.nodes);

            let error_str = if let Some(opt) = optimal {
                format!("{:.2}%", ((distance as f64 - opt) / opt) * 100.0)
            } else {
                "N/A".to_string()
            };

            let vs_lkh = if lkh_dist > 0.0 {
                let diff = ((distance as f64 - lkh_dist) / lkh_dist) * 100.0;
                if diff > 0.0 {
                    format!("+{:.2}%", diff)
                } else {
                    format!("{:.2}%", diff)
                }
            } else {
                "N/A".to_string()
            };

            println!(
                "{:<12} {:<6} {:<35} {:<12.2} {:<10.2} {:<10} {:<10}",
                instance.name,
                instance.dimension,
                strategy.name(),
                distance,
                duration.as_secs_f64() * 1000.0,
                error_str,
                vs_lkh
            );
        }

        if let Some(opt) = optimal {
            println!(
                "{:<12} {:<6} {:<35} {:<12.2} {:<10} {:<10} {:<10}",
                instance.name,
                instance.dimension,
                "--- OPTIMO (Concorde) ---",
                opt,
                "-",
                "0.00%",
                "ref"
            );
        }
        if lkh_dist > 0.0 {
            let lkh_err = if let Some(opt) = optimal {
                ((lkh_dist - opt) / opt) * 100.0
            } else {
                0.0
            };
            println!(
                "{:<12} {:<6} {:<35} {:<12.2} {:<10} {:<10} {:<10}",
                instance.name,
                instance.dimension,
                "--- LKH (ref) ---",
                lkh_dist,
                "-",
                format!("{:.2}%", lkh_err),
                "ref"
            );
        }
        println!("{}", "-".repeat(100));
    }

    println!("\nGenerating Synthetic Instance (1000 nodes)...");
    let mut rng_seed = 42u32;
    let mut simple_rand = || {
        rng_seed = rng_seed.wrapping_mul(1103515245).wrapping_add(12345);
        (rng_seed % 10000) as f32 / 10000.0
    };

    let synthetic_nodes: Vec<Node> = (0..1000)
        .map(|_| Node::new(simple_rand() * 1000.0, simple_rand() * 1000.0))
        .collect();

    let synthetic_name = "Synthetic-1000";

    for id in &strategy_ids {
        if (*id == "lin_kernighan" || *id == "triangle_insertion_v9_ils") && synthetic_nodes.len() > 500 {
            println!(
                "{:<12} {:<6} {:<35} {:<12} {:<10} {:<10} {:<10}",
                synthetic_name,
                1000,
                if *id == "lin_kernighan" { "LK (Skipped >500 nodes)" } else { "V9+ILS (Skipped >500 nodes)" },
                "---",
                "---",
                "N/A",
                "N/A"
            );
            continue;
        }
        let mut strategy = registry.get_strategy(id).expect("Strategy not found");
        let mut path = Vec::new();

        let start = Instant::now();
        let mut steps = 0;
        loop {
            let finished = strategy.execute_step(&mut path, &synthetic_nodes);
            steps += 1;
            if finished || steps > 2000 {
                break;
            }
        }
        let duration = start.elapsed();
        let distance = path_distance(&path, &synthetic_nodes);

        println!(
            "{:<12} {:<6} {:<35} {:<12.2} {:<10.2} {:<10} {:<10}",
            synthetic_name,
            1000,
            strategy.name(),
            distance,
            duration.as_secs_f64() * 1000.0,
            "N/A",
            "N/A"
        );
    }
    println!("{}", "-".repeat(100));
}
