use std::collections::HashMap;
use traveler::core::Node;
use traveler::tsplib::TspInstance;

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn stddev(values: &[f32], mean: f32) -> f32 {
    (values.iter().map(|&v| (v - mean).powi(2)).sum::<f32>() / values.len() as f32).sqrt()
}

fn percentile(values: &[f32], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p / 100.0) * (sorted.len() - 1) as f32) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn analyze(nodes: &[Node]) -> HashMap<String, f32> {
    let mut metrics = HashMap::new();

    let xs: Vec<f32> = nodes.iter().map(|n| n.pos.x).collect();
    let ys: Vec<f32> = nodes.iter().map(|n| n.pos.y).collect();

    let mean_x = mean(&xs);
    let mean_y = mean(&ys);
    let std_x = stddev(&xs, mean_x);
    let std_y = stddev(&ys, mean_y);

    // Varianza total (suma de varianzas x e y)
    metrics.insert("var_total".to_string(), std_x.powi(2) + std_y.powi(2));

    // CV combinado
    let cv_x = std_x / mean_x.abs().max(1e-6);
    let cv_y = std_y / mean_y.abs().max(1e-6);
    metrics.insert("cv_total".to_string(), cv_x + cv_y);

    // Distancias
    let mut all_dists = Vec::new();
    let mut nearest_dists = Vec::new();

    for i in 0..nodes.len() {
        let mut min_dist = f32::MAX;
        for j in 0..nodes.len() {
            if i == j {
                continue;
            }
            let d = nodes[i].distance_to(&nodes[j]);
            all_dists.push(d);
            if d < min_dist {
                min_dist = d;
            }
        }
        nearest_dists.push(min_dist);
    }

    let mean_nearest = mean(&nearest_dists);
    let mean_all = mean(&all_dists);
    metrics.insert("mean_nearest".to_string(), mean_nearest);
    metrics.insert("mean_all".to_string(), mean_all);
    metrics.insert("nearest_ratio".to_string(), mean_nearest / mean_all);

    // Percentiles de distancias
    let p10 = percentile(&all_dists, 10.0);
    let p25 = percentile(&all_dists, 25.0);
    let p75 = percentile(&all_dists, 75.0);
    let p90 = percentile(&all_dists, 90.0);
    metrics.insert("p10_dist".to_string(), p10);
    metrics.insert("p25_dist".to_string(), p25);
    metrics.insert("p75_dist".to_string(), p75);
    metrics.insert("p90_dist".to_string(), p90);
    metrics.insert("iqr_dist".to_string(), p75 - p25);

    // Cluster score simple: puntos con al menos un vecino dentro del percentil 25
    let mut cluster_count = 0;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if nodes[i].distance_to(&nodes[j]) <= p25 {
                cluster_count += 2;
                break;
            }
        }
    }
    metrics.insert("cluster_score".to_string(), cluster_count as f32 / nodes.len() as f32);

    // Densidad local vs global (Hopkins-like simplificado)
    // Contar puntos en celdas de una cuadrícula simple
    let min_x = xs.iter().cloned().fold(f32::MAX, f32::min);
    let max_x = xs.iter().cloned().fold(f32::MIN, f32::max);
    let min_y = ys.iter().cloned().fold(f32::MAX, f32::min);
    let max_y = ys.iter().cloned().fold(f32::MIN, f32::max);
    let width = max_x - min_x;
    let height = max_y - min_y;
    let area = width * height;
    metrics.insert("density".to_string(), nodes.len() as f32 / area.max(1.0));

    // Dispersión relativa: distancia máxima / distancia mínima entre puntos
    let min_dist = all_dists.iter().cloned().fold(f32::MAX, f32::min);
    let max_dist = all_dists.iter().cloned().fold(f32::MIN, f32::max);
    metrics.insert("dispersion".to_string(), max_dist / min_dist.max(1e-6));

    // Número de clusters con DBSCAN simple (eps = percentil 25 de distancias)
    let eps = p25;
    let mut visited = vec![false; nodes.len()];
    let mut cluster_count = 0;

    for i in 0..nodes.len() {
        if visited[i] {
            continue;
        }
        cluster_count += 1;
        let mut stack = vec![i];
        visited[i] = true;
        while let Some(current) = stack.pop() {
            for j in 0..nodes.len() {
                if visited[j] {
                    continue;
                }
                if nodes[current].distance_to(&nodes[j]) <= eps {
                    visited[j] = true;
                    stack.push(j);
                }
            }
        }
    }
    metrics.insert("dbscan_clusters".to_string(), cluster_count as f32);

    // Tamaño del cluster más grande
    let mut cluster_sizes = Vec::new();
    visited = vec![false; nodes.len()];
    for i in 0..nodes.len() {
        if visited[i] {
            continue;
        }
        let mut size = 0;
        let mut stack = vec![i];
        visited[i] = true;
        while let Some(current) = stack.pop() {
            size += 1;
            for j in 0..nodes.len() {
                if visited[j] {
                    continue;
                }
                if nodes[current].distance_to(&nodes[j]) <= eps {
                    visited[j] = true;
                    stack.push(j);
                }
            }
        }
        cluster_sizes.push(size);
    }
    let max_cluster_size = cluster_sizes.iter().cloned().max().unwrap_or(0);
    metrics.insert("max_cluster_ratio".to_string(), max_cluster_size as f32 / nodes.len() as f32);

    // Ratio entre distancia media inter-cluster (aproximada por percentiles) e intra-cluster
    metrics.insert("p75_p25_ratio".to_string(), p75 / p25.max(1e-6));

    // Proporción de puntos con al menos un vecino muy cercano (< p10)
    let mut very_close_count = 0;
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if nodes[i].distance_to(&nodes[j]) <= p10 {
                very_close_count += 2;
                break;
            }
        }
    }
    metrics.insert("very_close_ratio".to_string(), very_close_count as f32 / nodes.len() as f32);

    metrics
}

fn main() {
    let configs = [
        ("berlin52", "assets/berlin52.tsp"),
        ("eil51", "assets/eil51.tsp"),
        ("st70", "assets/st70.tsp"),
        ("kroA100", "assets/kroA100.tsp"),
        ("eil76", "assets/eil76.tsp"),
        ("ch130", "assets/ch130.tsp"),
        ("pr76", "assets/pr76.tsp"),
    ];

    println!("Instance     var_total      cv_total   mean_nearest  nearest_ratio   iqr_dist    cluster    density    dispersion   dbscan  max_clust   p75/p25  very_close");
    println!("{}", "-".repeat(160));

    for (name, path) in &configs {
        let inst = match TspInstance::from_file(path) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error {}: {}", name, e);
                continue;
            }
        };

        let metrics = analyze(&inst.nodes);
        println!(
            "{:<12} {:>12.2} {:>12.4} {:>12.2} {:>14.4} {:>12.2} {:>10.4} {:>10.6} {:>12.2} {:>8.0} {:>10.4} {:>10.2} {:>11.4}",
            name,
            metrics["var_total"],
            metrics["cv_total"],
            metrics["mean_nearest"],
            metrics["nearest_ratio"],
            metrics["iqr_dist"],
            metrics["cluster_score"],
            metrics["density"],
            metrics["dispersion"],
            metrics["dbscan_clusters"],
            metrics["max_cluster_ratio"],
            metrics["p75_p25_ratio"],
            metrics["very_close_ratio"]
        );
    }
}
