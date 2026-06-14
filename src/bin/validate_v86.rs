/// Validación de V8.6 con instancias no vistas durante el entrenamiento
///
/// Descarga instancias TSPLIB, ejecuta V8.6, V8 y LKH, y genera
/// un reporte estadístico para detectar overfitting.
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::process::Command;
use std::time::Instant;
use traveler::core::path_distance;
use traveler::stats;
use traveler::strategies::triangle_insertion_v8::TriangleInsertionV8;
use traveler::strategies::triangle_insertion_v8_6::TriangleInsertionV86;
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

/// Lista de instancias de validación (no usadas en entrenamiento)
const VALIDATION_INSTANCES: &[(&str, &str)] = &[
    // Instancias TSPLIB que ya tenemos en assets/
    ("berlin52", "assets/berlin52.tsp"),
    ("eil51", "assets/eil51.tsp"),
    ("st70", "assets/st70.tsp"),
    ("kroA100", "assets/kroA100.tsp"),
    ("eil76", "assets/eil76.tsp"),
    ("ch130", "assets/ch130.tsp"),
    ("pr76", "assets/pr76.tsp"),
];

/// Óptimos conocidos de las instancias de validación
fn get_validation_optima() -> HashMap<String, f64> {
    let mut optima = HashMap::new();
    // Cargar desde assets/optima.txt si existe
    if let Ok(content) = fs::read_to_string("assets/optima.txt") {
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

struct ValidationResult {
    name: String,
    n: usize,
    v8_error: f64,
    v86_error: f64,
    lkh_error: f64,
    v8_time_ms: f64,
    v86_time_ms: f64,
    lkh_time_ms: f64,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Validación de V8.6 - Detección de Overfitting             ║");
    println!("════════════════════════════════════════════════════════════════╝\n");

    let work_dir = "/tmp/v86_validation";
    let _ = fs::create_dir_all(work_dir);

    let optima = get_validation_optima();
    let mut results = Vec::new();
    let mut failed_downloads = Vec::new();

    println!("Paso 1: Cargando instancias de validación...\n");

    for &(name, path) in VALIDATION_INSTANCES {
        print!("  Cargando {}... ", name);

        let instance = match TspInstance::from_file(path) {
            Ok(inst) => {
                println!("✓ ({} nodos)", inst.dimension);
                inst
            }
            Err(e) => {
                println!("✗ ({})", e);
                failed_downloads.push(name);
                continue;
            }
        };

        let optimal = optima.get(&name.to_string()).copied();

        println!("  Ejecutando estrategias en {} ({} nodos)...", name, instance.dimension);

        // V8
        let (v8_dist, v8_time) = run_v8(&instance);
        let v8_error = optimal.map(|o| ((v8_dist as f64 - o) / o) * 100.0).unwrap_or(f64::MAX);

        // V8.6
        let (v86_dist, v86_time) = run_v86(&instance);
        let v86_error = optimal.map(|o| ((v86_dist as f64 - o) / o) * 100.0).unwrap_or(f64::MAX);

        // LKH
        let (lkh_dist, lkh_time) = run_lkh(&instance, work_dir);
        let lkh_error = optimal.map(|o| ((lkh_dist as f64 - o) / o) * 100.0).unwrap_or(f64::MAX);

        results.push(ValidationResult {
            name: name.to_string(),
            n: instance.dimension,
            v8_error,
            v86_error,
            lkh_error,
            v8_time_ms: v8_time,
            v86_time_ms: v86_time,
            lkh_time_ms: lkh_time,
        });

        println!("    V8:  {:.2}% ({:.1}ms)", v8_error, v8_time);
        println!("    V8.6: {:.2}% ({:.1}ms)", v86_error, v86_time);
        println!("    LKH: {:.2}% ({:.1}ms)", lkh_error, lkh_time);
        println!();
    }

    if !failed_downloads.is_empty() {
        println!("\n⚠ {} instancias no pudieron cargarse: {:?}", failed_downloads.len(), failed_downloads);
    }

    if results.is_empty() {
        println!("\n✗ No se pudieron obtener resultados.");
        return;
    }

    // Generar reporte
    println!("\n════════════════════════════════════════════════════════════════╗");
    println!("║     REPORTE ESTADÍSTICO                                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let v8_errors: Vec<f64> = results.iter().map(|r| r.v8_error).filter(|&e| e < f64::MAX).collect();
    let v86_errors: Vec<f64> = results.iter().map(|r| r.v86_error).filter(|&e| e < f64::MAX).collect();
    let lkh_errors: Vec<f64> = results.iter().map(|r| r.lkh_error).filter(|&e| e < f64::MAX).collect();
    let n_values: Vec<f64> = results.iter().map(|r| r.n as f64).collect();

    if !v8_errors.is_empty() {
        println!("Métricas de Error (% vs Óptimo):");
        println!("  V8:   media={:.2}%, mediana={:.2}%, std={:.2}%",
            stats::mean(&v8_errors), stats::median(&v8_errors), stats::std_dev(&v8_errors));
        println!("  V8.6: media={:.2}%, mediana={:.2}%, std={:.2}%",
            stats::mean(&v86_errors), stats::median(&v86_errors), stats::std_dev(&v86_errors));
        println!("  LKH:  media={:.2}%, mediana={:.2}%, std={:.2}%",
            stats::mean(&lkh_errors), stats::median(&lkh_errors), stats::std_dev(&lkh_errors));
    } else {
        println!("Métricas de Error: No disponibles (sin óptimos conocidos)");
    }

    println!("\nMétricas de Tiempo (ms):");
    let v8_times: Vec<f64> = results.iter().map(|r| r.v8_time_ms).collect();
    let v86_times: Vec<f64> = results.iter().map(|r| r.v86_time_ms).collect();
    println!("  V8:   media={:.1}ms, mediana={:.1}ms", stats::mean(&v8_times), stats::median(&v8_times));
    println!("  V8.6: media={:.1}ms, mediana={:.1}ms", stats::mean(&v86_times), stats::median(&v86_times));

    // Test de Wilcoxon
    let mut p_value = 1.0;
    if v8_errors.len() >= 10 && v8_errors.len() == v86_errors.len() {
        println!("\nTest de Wilcoxon Signed-Rank (V8 vs V8.6):");
        let (w_plus, w_minus, p_val) = stats::wilcoxon_signed_rank(&v8_errors, &v86_errors);
        p_value = p_val;
        println!("  W+ = {:.1}, W- = {:.1}", w_plus, w_minus);
        println!("  p-value = {:.4}", p_value);
        if p_value < 0.05 {
            println!("  → Diferencia estadísticamente significativa (p < 0.05)");
            if stats::mean(&v86_errors) < stats::mean(&v8_errors) {
                println!("  → V8.6 es significativamente MEJOR que V8");
            } else {
                println!("  → V8.6 es significativamente PEOR que V8");
            }
        } else {
            println!("  → No hay diferencia estadísticamente significativa");
        }
    }

    // Correlación tamaño-error
    let mut corr_v8 = 0.0;
    let mut corr_v86 = 0.0;
    if v8_errors.len() >= 10 {
        println!("\nCorrelación Tamaño vs Error:");
        corr_v8 = stats::pearson_correlation(&n_values[..v8_errors.len()], &v8_errors);
        corr_v86 = stats::pearson_correlation(&n_values[..v86_errors.len()], &v86_errors);
        println!("  V8:   r = {:.3}", corr_v8);
        println!("  V8.6: r = {:.3}", corr_v86);
    }

    // Detección de overfitting
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║     ANÁLISIS DE OVERFITTING                                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Error en entrenamiento (de resultados previos)
    let train_error_v8 = 3.68; // Promedio de las 7 instancias de entrenamiento
    let train_error_v86 = 1.97;

    let test_error_v8 = stats::mean(&v8_errors);
    let test_error_v86 = stats::mean(&v86_errors);

    println!("Error promedio en ENTRENAMIENTO (7 instancias TSPLIB):");
    println!("  V8:   {:.2}%", train_error_v8);
    println!("  V8.6: {:.2}%", train_error_v86);

    println!("\nError promedio en TEST ({} instancias no vistas):", results.len());
    println!("  V8:   {:.2}%", test_error_v8);
    println!("  V8.6: {:.2}%", test_error_v86);

    let degradation_v8 = test_error_v8 - train_error_v8;
    let degradation_v86 = test_error_v86 - train_error_v86;

    println!("\nDegradación (Test - Train):");
    println!("  V8:   {:+.2}%", degradation_v8);
    println!("  V8.6: {:+.2}%", degradation_v86);

    // Criterios de overfitting
    let std_v86_train = 1.0; // Aproximado de los resultados de entrenamiento
    let overfitting_threshold = train_error_v86 + 2.0 * std_v86_train;

    println!("\nCriterios de Overfitting:");
    println!("  1. Error test > Error train + 2*std:");
    println!("     Umbral: {:.2}%, Actual: {:.2}%", overfitting_threshold, test_error_v86);
    if test_error_v86 > overfitting_threshold {
        println!("     → POSIBLE OVERFITTING");
    } else {
        println!("     → Sin overfitting detectado");
    }

    println!("\n  2. Ratio V8.6/V8 > 1.5 en >30% de instancias:");
    let mut high_ratio_count = 0;
    for r in &results {
        if r.v8_error > 0.1 && r.v86_error / r.v8_error > 1.5 {
            high_ratio_count += 1;
        }
    }
    let ratio = high_ratio_count as f64 / results.len() as f64;
    println!("     Ratio: {:.1}% ({}/{})", ratio * 100.0, high_ratio_count, results.len());
    if ratio > 0.3 {
        println!("     → POSIBLE OVERFITTING");
    } else {
        println!("     → Sin overfitting detectado");
    }

    println!("\n  3. Correlación degradación vs diferencia de geometría:");
    println!("     (Requiere clasificación manual de geometrías)");

    // Conclusión
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║     CONCLUSIÓN                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    if test_error_v86 < train_error_v86 * 1.5 && ratio < 0.3 {
        println!("✓ V8.6 NO muestra signos de overfitting significativo.");
        println!("  La calibración generaliza bien a instancias no vistas.");
    } else if test_error_v86 < train_error_v86 * 2.0 {
        println!("⚠ V8.6 muestra leve degradación pero dentro de límites aceptables.");
        println!("  Se recomienda ampliar el conjunto de entrenamiento.");
    } else {
        println!("✗ V8.6 muestra signos de overfitting.");
        println!("  Se recomienda reentrenar con más instancias diversas.");
    }

    // Guardar reporte
    let report_path = format!("{}/validation_report.txt", work_dir);
    let mut report = String::new();
    report.push_str("# Reporte de Validación V8.6\n\n");
    report.push_str(&format!("## Instancias evaluadas: {}\n\n", results.len()));
    report.push_str("## Resultados detallados:\n\n");
    report.push_str("Instancia | N | V8 Error% | V8.6 Error% | LKH Error% | V8 ms | V8.6 ms | LKH ms\n");
    report.push_str("---|---|---|---|---|---|---|---\n");

    for r in &results {
        report.push_str(&format!(
            "{} | {} | {:.2} | {:.2} | {:.2} | {:.1} | {:.1} | {:.1}\n",
            r.name, r.n, r.v8_error, r.v86_error, r.lkh_error,
            r.v8_time_ms, r.v86_time_ms, r.lkh_time_ms
        ));
    }

    report.push_str(&format!("\n## Estadísticas:\n\n"));
    report.push_str(&format!("- V8 mean error: {:.2}%\n", stats::mean(&v8_errors)));
    report.push_str(&format!("- V8.6 mean error: {:.2}%\n", stats::mean(&v86_errors)));
    report.push_str(&format!("- LKH mean error: {:.2}%\n", stats::mean(&lkh_errors)));
    report.push_str(&format!("- Wilcoxon p-value: {:.4}\n", p_value));
    report.push_str(&format!("- Correlación V8 (N vs error): {:.3}\n", corr_v8));
    report.push_str(&format!("- Correlación V8.6 (N vs error): {:.3}\n", corr_v86));

    fs::write(&report_path, &report).unwrap();
    println!("\n✓ Reporte guardado en: {}", report_path);
}

fn run_v8(instance: &TspInstance) -> (f32, f64) {
    let mut strategy = TriangleInsertionV8::new();
    let mut path = Vec::new();
    let mut steps = 0;
    let start = Instant::now();

    loop {
        let finished = strategy.execute_step(&mut path, &instance.nodes);
        steps += 1;
        if finished || steps > instance.nodes.len() + 500 {
            break;
        }
    }

    let time = start.elapsed().as_secs_f64() * 1000.0;
    let dist = path_distance(&path, &instance.nodes);
    (dist, time)
}

fn run_v86(instance: &TspInstance) -> (f32, f64) {
    let mut strategy = TriangleInsertionV86::new();
    // Cargar parámetros calibrados
    strategy.load_calibrated_params("assets/v86_training_results.txt");

    let mut path = Vec::new();
    let mut steps = 0;
    let start = Instant::now();

    loop {
        let finished = strategy.execute_step(&mut path, &instance.nodes);
        steps += 1;
        if finished || steps > instance.nodes.len() + 500 {
            break;
        }
    }

    let time = start.elapsed().as_secs_f64() * 1000.0;
    let dist = path_distance(&path, &instance.nodes);
    (dist, time)
}

fn run_lkh(instance: &TspInstance, work_dir: &str) -> (f32, f64) {
    let problem_path = format!("{}/{}.tsp", work_dir, instance.name);

    // Escribir archivo de problema
    let mut file = fs::File::create(&problem_path).unwrap();
    use std::io::Write;
    writeln!(file, "NAME : {}", instance.name).unwrap();
    writeln!(file, "TYPE : TSP").unwrap();
    writeln!(file, "DIMENSION : {}", instance.dimension).unwrap();
    writeln!(file, "EDGE_WEIGHT_TYPE : EUC_2D").unwrap();
    writeln!(file, "NODE_COORD_SECTION").unwrap();
    for (i, node) in instance.nodes.iter().enumerate() {
        writeln!(file, "{} {} {}", i + 1, node.pos.x, node.pos.y).unwrap();
    }
    writeln!(file, "EOF").unwrap();

    // Escribir parámetros
    let param_path = format!("{}/{}_params.par", work_dir, instance.name);
    let mut file = fs::File::create(&param_path).unwrap();
    writeln!(file, "PROBLEM_FILE = {}", problem_path).unwrap();
    writeln!(file, "TOUR_FILE = {}/{}_solution.tour", work_dir, instance.name).unwrap();
    writeln!(file, "RUNS = 1").unwrap();
    writeln!(file, "MAX_TRIALS = 1").unwrap();
    writeln!(file, "SEED = 42").unwrap();

    let tour_path = format!("{}/{}_solution.tour", work_dir, instance.name);

    let start = Instant::now();
    let result = Command::new("./LKH-3.0.14/LKH")
        .arg(&param_path)
        .output();

    let time = start.elapsed().as_secs_f64() * 1000.0;

    match result {
        Ok(output) if output.status.success() => {
            // Leer solución
            if let Ok(content) = fs::read_to_string(&tour_path) {
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
                    let dist = path_distance(&indices, &instance.nodes);
                    return (dist, time);
                }
            }
        }
        _ => {}
    }

    // Fallback: retornar distancia máxima
    (f32::MAX, time)
}

fn download_and_decompress(url: &str, output_path: &str) -> Result<(), String> {
    // Usar curl para descargar
    let gz_path = format!("{}.gz", output_path);

    let status = Command::new("curl")
        .args(["-sL", "--connect-timeout", "10", "--max-time", "30", url, "-o", &gz_path])
        .status()
        .map_err(|e| format!("Error ejecutando curl: {}", e))?;

    if !status.success() {
        let _ = fs::remove_file(&gz_path);
        return Err("curl falló".to_string());
    }

    // Verificar que el archivo no esté vacío
    if let Ok(metadata) = fs::metadata(&gz_path) {
        if metadata.len() < 100 {
            let _ = fs::remove_file(&gz_path);
            return Err("Archivo demasiado pequeño".to_string());
        }
    }

    // Descomprimir con gunzip
    let status = Command::new("gunzip")
        .args(["-f", &gz_path])
        .status()
        .map_err(|e| format!("Error ejecutando gunzip: {}", e))?;

    if !status.success() {
        return Err("gunzip falló".to_string());
    }

    Ok(())
}

/// Genera instancias sintéticas para validación cuando no hay conexión
fn generate_synthetic_instances(work_dir: &str) -> Vec<(String, TspInstance)> {
    use traveler::core::Node;
    use macroquad::prelude::Vec2;

    let mut instances = Vec::new();

    // Instancia 1: 50 nodos aleatorios
    let nodes: Vec<Node> = (0..50)
        .map(|i| {
            let x = (i as f32 * 37.7) % 1000.0;
            let y = (i as f32 * 73.1) % 1000.0;
            Node::new(x, y)
        })
        .collect();
    instances.push(("synthetic_50".to_string(), TspInstance {
        name: "synthetic_50".to_string(),
        dimension: 50,
        nodes,
        optimal_distance: None,
    }));

    // Instancia 2: 100 nodos en grid
    let nodes: Vec<Node> = (0..100)
        .map(|i| {
            let x = (i % 10) as f32 * 100.0;
            let y = (i / 10) as f32 * 100.0;
            Node::new(x, y)
        })
        .collect();
    instances.push(("synthetic_100_grid".to_string(), TspInstance {
        name: "synthetic_100_grid".to_string(),
        dimension: 100,
        nodes,
        optimal_distance: None,
    }));

    // Instancia 3: 150 nodos en círculo
    let nodes: Vec<Node> = (0..150)
        .map(|i| {
            let angle = i as f32 * 2.0 * std::f32::consts::PI / 150.0;
            let x = 500.0 + 400.0 * angle.cos();
            let y = 500.0 + 400.0 * angle.sin();
            Node::new(x, y)
        })
        .collect();
    instances.push(("synthetic_150_circle".to_string(), TspInstance {
        name: "synthetic_150_circle".to_string(),
        dimension: 150,
        nodes,
        optimal_distance: None,
    }));

    // Instancia 4: 200 nodos con clusters
    let mut nodes = Vec::new();
    for cluster in 0..5 {
        let cx = (cluster % 3) as f32 * 300.0 + 100.0;
        let cy = (cluster / 3) as f32 * 300.0 + 100.0;
        for i in 0..40 {
            let x = cx + (i as f32 * 17.3) % 100.0 - 50.0;
            let y = cy + (i as f32 * 23.7) % 100.0 - 50.0;
            nodes.push(Node::new(x, y));
        }
    }
    instances.push(("synthetic_200_clusters".to_string(), TspInstance {
        name: "synthetic_200_clusters".to_string(),
        dimension: 200,
        nodes,
        optimal_distance: None,
    }));

    instances
}
