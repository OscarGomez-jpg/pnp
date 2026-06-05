use std::time::Instant;
use traveler::core::{path_distance, Node};
use traveler::strategies::{create_registry, StrategyRegistry};
use traveler::tsplib::TspInstance;

fn main() {
    let registry = create_registry();
    let instances = vec![
        "assets/berlin52.tsp",
        "assets/kroA100.tsp",
        "assets/ch130.tsp",
    ];

    println!("{:<30} {:<25} {:<15} {:<15} {:<15}", "Instance", "Strategy", "Distance", "Time (ms)", "Error %");
    println!("{}", "-".repeat(110));

    // Run on TSPLIB files
    for instance_path in &instances {
        let instance = match TspInstance::from_file(instance_path) {
            Ok(inst) => inst,
            Err(e) => {
                eprintln!("Error loading {}: {}", instance_path, e);
                continue;
            }
        };

        let optimal = instance.optimal_distance;
        
        let strategy_ids = vec![
            "nearest_neighbor",
            "triangle_insertion",
            "triangle_insertion_v6",
            "triangle_insertion_v7",
            "christofides",
        ];

        for id in strategy_ids {
            let mut strategy = registry.get_strategy(id).expect("Strategy not found");
            let mut path = Vec::new();
            
            let start = Instant::now();
            let mut steps = 0;
            loop {
                let finished = strategy.execute_step(&mut path, &instance.nodes);
                steps += 1;
                if finished || steps > instance.nodes.len() + 200 {
                    break;
                }
            }
            let duration = start.elapsed();
            let distance = path_distance(&path, &instance.nodes);
            
            let error_str = if let Some(opt) = optimal {
                format!("{:.2}%", ((distance - opt) / opt) * 100.0)
            } else {
                "N/A".to_string()
            };

            println!(
                "{:<30} {:<25} {:<15.2} {:<15.2} {:<15}",
                instance.name,
                strategy.name(),
                distance,
                duration.as_secs_f64() * 1000.0,
                error_str
            );
        }
        println!("{}", "-".repeat(110));
    }

    // Add a synthetic large instance
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

    let strategy_ids = vec![
        "nearest_neighbor",
        "triangle_insertion",
        "triangle_insertion_v6",
        "triangle_insertion_v7",
        "christofides",
    ];

    for id in strategy_ids {
        if id == "triangle_insertion_v6" && synthetic_nodes.len() > 500 {
            println!("{:<30} {:<25} {:<15} {:<15} {:<15}", synthetic_name, "V6 (Skipped >500 nodes)", "---", "---", "N/A");
            continue;
        }
        let mut strategy = registry.get_strategy(id).expect("Strategy not found");
        let mut path = Vec::new();
        
        let start = Instant::now();
        let mut steps = 0;
        loop {
            let finished = strategy.execute_step(&mut path, &synthetic_nodes);
            steps += 1;
            if finished || steps > 1000 {
                break;
            }
        }
        let duration = start.elapsed();
        let distance = path_distance(&path, &synthetic_nodes);
        
        println!(
            "{:<30} {:<25} {:<15.2} {:<15.2} {:<15}",
            synthetic_name,
            strategy.name(),
            distance,
            duration.as_secs_f64() * 1000.0,
            "N/A"
        );
    }
    println!("{}", "-".repeat(110));
}
