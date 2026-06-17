use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;
use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::triangle_insertion_v8_9::{TriangleInsertionV89, V89Params};
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

fn run_lkh(instance: &TspInstance) -> Option<(Vec<usize>, f64)> {
    let work_dir = "/tmp/lkh_compare";
    let _ = std::fs::create_dir_all(work_dir);

    let problem_path = format!("{}/problem.tsp", work_dir);
    let mut file = File::create(&problem_path).ok()?;

    writeln!(file, "NAME : {}", instance.name).ok()?;
    writeln!(file, "TYPE : TSP").ok()?;
    writeln!(file, "DIMENSION : {}", instance.dimension).ok()?;
    writeln!(file, "EDGE_WEIGHT_TYPE : EUC_2D").ok()?;
    writeln!(file, "NODE_COORD_SECTION").ok()?;
    for (i, node) in instance.nodes.iter().enumerate() {
        writeln!(file, "{} {} {}", i + 1, node.pos.x, node.pos.y).ok()?;
    }
    writeln!(file, "EOF").ok()?;

    let param_path = format!("{}/params.par", work_dir);
    let mut file = File::create(&param_path).ok()?;
    writeln!(file, "PROBLEM_FILE = {}", problem_path).ok()?;
    writeln!(file, "TOUR_FILE = {}/solution.tour", work_dir).ok()?;
    writeln!(file, "RUNS = 1").ok()?;
    writeln!(file, "MAX_TRIALS = 1").ok()?;
    writeln!(file, "SEED = 42").ok()?;

    let result = Command::new("./LKH-3.0.14/LKH")
        .arg(&param_path)
        .output()
        .ok()?;

    if !result.status.success() {
        return None;
    }

    let tour_path = format!("{}/solution.tour", work_dir);
    let content = std::fs::read_to_string(&tour_path).ok()?;

    let mut indices = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("NAME")
            || line.starts_with("COMMENT") || line.starts_with("TYPE")
            || line.starts_with("DIMENSION") || line.starts_with("TOUR_SECTION")
            || line.starts_with("EOF")
        {
            continue;
        }
        if let Ok(val) = line.parse::<i64>() {
            if val == -1 {
                break;
            }
            if val > 0 {
                indices.push((val - 1) as usize);
            }
        }
    }

    if indices.len() == instance.nodes.len() {
        let dist = path_distance(&indices, &instance.nodes) as f64;
        Some((indices, dist))
    } else {
        None
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Comparación Detallada: V8.6 vs V8.9 vs LKH               ║");
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

    let v86_params = V86Params {
        k_neighbors: 4,
        w_angle: 0.25,
        w_cost: 0.50,
    };

    let v89_params = V89Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    println!("\n{:─<120}", "");
    println!("{:<12} {:>8} {:>10} {:>12} {:>12} {:>12} {:>12} {:>12} {:>12}", 
             "Instancia", "Nodos", "Óptimo", "V8.6 Dist", "V8.9 Dist", "LKH Dist", 
             "V8.6 Err", "V8.9 Err", "LKH Err");
    println!("{:─<120}", "");

    let mut json_data = Vec::new();

    for instance in &instances {
        let optimal = optima.get(&instance.name).copied().unwrap_or(0.0);

        // V8.6 (Seagull)
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
        let time_v86 = start.elapsed().as_secs_f64() * 1000.0;
        let dist_v86 = path_distance(&path_v86, &instance.nodes);
        let error_v86 = ((dist_v86 - optimal as f32) / optimal as f32) * 100.0;

        // V8.9 (Pre-Seagull)
        let start = Instant::now();
        let mut strategy_v89 = TriangleInsertionV89::with_params(v89_params);
        let mut path_v89 = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy_v89.execute_step(&mut path_v89, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }
        let time_v89 = start.elapsed().as_secs_f64() * 1000.0;
        let dist_v89 = path_distance(&path_v89, &instance.nodes);
        let error_v89 = ((dist_v89 - optimal as f32) / optimal as f32) * 100.0;

        // LKH
        let (path_lkh, dist_lkh) = if let Some(result) = run_lkh(instance) {
            (result.0, result.1)
        } else {
            (vec![], 0.0)
        };
        let error_lkh = if dist_lkh > 0.0 {
            ((dist_lkh - optimal) / optimal) * 100.0
        } else {
            0.0
        };

        println!("{:<12} {:>8} {:>10.0} {:>12.2} {:>12.2} {:>12.2} {:>11.2}% {:>11.2}% {:>11.2}%",
                 instance.name, instance.dimension, optimal,
                 dist_v86, dist_v89, dist_lkh,
                 error_v86, error_v89, error_lkh);

        // Preparar datos JSON
        let nodes: Vec<Vec<f32>> = instance.nodes.iter()
            .map(|n| vec![n.pos.x, n.pos.y])
            .collect();

        let mut inst_data = serde_json::json!({
            "name": instance.name,
            "n": instance.dimension,
            "optimal": optimal,
            "nodes": nodes,
            "v86": {
                "tour": path_v86,
                "dist": dist_v86 as f64,
                "error": error_v86 as f64,
                "time_ms": time_v86
            },
            "v89": {
                "tour": path_v89,
                "dist": dist_v89 as f64,
                "error": error_v89 as f64,
                "time_ms": time_v89
            }
        });

        if !path_lkh.is_empty() {
            inst_data["lkh"] = serde_json::json!({
                "tour": path_lkh,
                "dist": dist_lkh,
                "error": error_lkh
            });
        }

        json_data.push(inst_data);
    }

    // Guardar JSON
    let json_file = File::create("assets/comparison_v86_v89.json").unwrap();
    serde_json::to_writer_pretty(json_file, &json_data).unwrap();
    println!("\n✓ Datos guardados en assets/comparison_v86_v89.json");

    // Resumen
    println!("\n{:─<60}", "");
    println!("RESUMEN:");
    let v86_avg_err: f64 = json_data.iter()
        .map(|d| d["v86"]["error"].as_f64().unwrap())
        .sum::<f64>() / json_data.len() as f64;
    let v89_avg_err: f64 = json_data.iter()
        .map(|d| d["v89"]["error"].as_f64().unwrap())
        .sum::<f64>() / json_data.len() as f64;
    let lkh_avg_err: f64 = json_data.iter()
        .filter(|d| d.get("lkh").is_some())
        .map(|d| d["lkh"]["error"].as_f64().unwrap())
        .sum::<f64>() / json_data.iter().filter(|d| d.get("lkh").is_some()).count() as f64;

    println!("  V8.6 (Seagull)     error promedio: {:.2}%", v86_avg_err);
    println!("  V8.9 (Pre-Seagull) error promedio: {:.2}%", v89_avg_err);
    println!("  LKH                error promedio: {:.2}%", lkh_avg_err);
}
