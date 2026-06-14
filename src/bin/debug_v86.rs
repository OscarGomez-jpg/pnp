/// Herramienta de debugging paso a paso para V8.6
///
/// Muestra en detalle cada decisión del algoritmo:
/// - Qué puntos se evalúan
/// - Scores de cada candidato
/// - Por qué se elige un punto y posición específica
/// - Valores de angle_score, cost_penalty, total_score
use std::fs;
use std::io::Write;
use traveler::core::{Node, insertion_cost, path_distance};
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

struct DebugInfo {
    step: usize,
    path_before: Vec<usize>,
    candidate_evaluated: usize,
    position_evaluated: usize,
    angle_score: f32,
    cost_penalty: f32,
    total_score: f32,
    chosen: bool,
    path_after: Vec<usize>,
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Debug Step-by-Step V8.6                                   ║");
    println!("════════════════════════════════════════════════════════════════╝\n");

    // Cargar instancia de ejemplo (berlin52)
    let instance_path = "assets/berlin52.tsp";
    let instance = match TspInstance::from_file(instance_path) {
        Ok(inst) => inst,
        Err(e) => {
            eprintln!("Error cargando {}: {}", instance_path, e);
            return;
        }
    };

    println!("Instancia: {} ({} nodos)", instance.name, instance.dimension);
    println!("Óptimo conocido: 7542\n");

    // Parámetros de V8.6
    let params = V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    println!("Parámetros V8.6:");
    println!("  k_neighbors: {}", params.k_neighbors);
    println!("  w_angle: {:.2}", params.w_angle);
    println!("  w_cost: {:.2}\n", params.w_cost);

    // Ejecutar V8.6 con debugging
    let mut strategy = TriangleInsertionV86::with_params(params);
    let mut path = Vec::new();
    let mut debug_log = Vec::new();
    let mut step = 0;

    println!("Ejecutando V8.6 paso a paso...\n");

    loop {
        let path_before = path.clone();
        let finished = strategy.execute_step(&mut path, &instance.nodes);
        step += 1;

        if !path_before.is_empty() && path.len() > path_before.len() {
            // Se insertó un nuevo nodo
            let new_node = path.iter().find(|&&n| !path_before.contains(&n)).copied().unwrap_or(0);
            let position = path.iter().position(|&n| n == new_node).unwrap_or(0);

            debug_log.push(DebugInfo {
                step,
                path_before,
                candidate_evaluated: new_node,
                position_evaluated: position,
                angle_score: 0.0,
                cost_penalty: 0.0,
                total_score: 0.0,
                chosen: true,
                path_after: path.clone(),
            });
        }

        if finished || step > instance.nodes.len() + 10 {
            break;
        }
    }

    // Mostrar resumen
    println!("=== RESUMEN DE EJECUCIÓN ===\n");
    println!("Tour final: {} nodos", path.len());
    println!("Distancia: {:.2}", path_distance(&path, &instance.nodes));
    println!("Error vs óptimo: {:.2}%\n", ((path_distance(&path, &instance.nodes) - 7542.0) / 7542.0) * 100.0);

    // Guardar log detallado
    let log_path = "v86_debug_log.txt";
    let mut file = fs::File::create(log_path).unwrap();

    writeln!(file, "# Debug Log V8.6 - Berlin52").unwrap();
    writeln!(file, "# Parámetros: k={}, w_angle={:.2}, w_cost={:.2}\n", params.k_neighbors, params.w_angle, params.w_cost).unwrap();

    for info in &debug_log {
        writeln!(file, "Paso {}: Insertar nodo {} en posición {}", info.step, info.candidate_evaluated, info.position_evaluated).unwrap();
        writeln!(file, "  Path antes: {:?}", info.path_before).unwrap();
        writeln!(file, "  Path después: {:?}", info.path_after).unwrap();
        writeln!(file, "").unwrap();
    }

    println!("✓ Log detallado guardado en: {}", log_path);

    // Ahora ejecutar con debugging completo de scores
    println!("\n════════════════════════════════════════════════════════════════╗");
    println!("║     ANÁLISIS DETALLADO DE SCORES                              ║");
    println!("════════════════════════════════════════════════════════════════╝\n");

    analyze_scores(&instance, params);
}

fn analyze_scores(instance: &TspInstance, params: V86Params) {
    use macroquad::prelude::Vec2;
    use std::collections::BinaryHeap;
    use std::cmp::Ordering;

    // Reconstruir K-D Tree y lógica de V8.6 para mostrar scores
    println!("Analizando primeros 5 pasos en detalle...\n");

    // Simular los primeros pasos manualmente
    let nodes = &instance.nodes;
    let mut unvisited: Vec<usize> = (0..nodes.len()).collect();

    // Paso 1: Casco convexo
    println!("=== PASO 1: Inicialización con Casco Convexo ===\n");

    // Calcular casco convexo (simplificado)
    let mut indexed: Vec<usize> = (0..nodes.len()).collect();
    indexed.sort_by(|&a, &b| {
        let pa = nodes[a].pos;
        let pb = nodes[b].pos;
        pa.x.partial_cmp(&pb.x).unwrap().then(pa.y.partial_cmp(&pb.y).unwrap())
    });

    let cross = |o: usize, a: usize, b: usize| -> f32 {
        let po = nodes[o].pos;
        let pa = nodes[a].pos;
        let pb = nodes[b].pos;
        (pa.x - po.x) * (pb.y - po.y) - (pa.y - po.y) * (pb.x - po.x)
    };

    let mut lower: Vec<usize> = Vec::new();
    for &idx in &indexed {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], idx) <= 0.0 {
            lower.pop();
        }
        lower.push(idx);
    }

    let mut upper: Vec<usize> = Vec::new();
    for &idx in indexed.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], idx) <= 0.0 {
            upper.pop();
        }
        upper.push(idx);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);

    println!("Casco convexo: {} nodos", lower.len());
    println!("Nodos del casco: {:?}\n", lower);

    let mut path = lower.clone();
    for &idx in &lower {
        if let Some(pos) = unvisited.iter().position(|&x| x == idx) {
            unvisited.swap_remove(pos);
        }
    }

    println!("Unvisited restantes: {} nodos\n", unvisited.len());

    // Mostrar análisis de los primeros 3 unvisited
    println!("=== ANÁLISIS DE SCORES PARA PRIMEROS 3 NODOS NO VISITADOS ===\n");

    for (i, &candidate) in unvisited.iter().take(3).enumerate() {
        println!("--- Candidato #{}: Nodo {} ---", i + 1, candidate);
        println!("Coordenadas: ({:.2}, {:.2})\n", nodes[candidate].pos.x, nodes[candidate].pos.y);

        // Evaluar todas las posiciones de inserción
        let mut best_pos = 0;
        let mut best_score = f32::MIN;

        for pos in 0..path.len() {
            let next = (pos + 1) % path.len();

            // Calcular ángulo de inserción
            let p_i = nodes[path[pos]].pos;
            let p_j = nodes[path[next]].pos;
            let p_u = nodes[candidate].pos;

            let v1 = p_i - p_u;
            let v2 = p_j - p_u;
            let len1 = v1.length();
            let len2 = v2.length();

            let angle = if len1 > 1e-5 && len2 > 1e-5 {
                let cos_theta = (v1.dot(v2) / (len1 * len2)).clamp(-1.0, 1.0);
                cos_theta.acos()
            } else {
                0.0
            };

            let angle_score = angle / std::f32::consts::PI;

            // Calcular penalización por costo
            let cost = insertion_cost(path[pos], path[next], candidate, nodes);
            let edge_len = nodes[path[pos]].pos.distance(nodes[path[next]].pos);
            let cost_ratio = if edge_len > 1e-5 { cost / edge_len } else { 1.0 };
            let cost_penalty = 1.0 / (1.0 + cost_ratio);

            // Score total
            let total_score = angle_score * params.w_angle + cost_penalty * params.w_cost;

            println!("  Posición {} (entre nodos {} y {}):", pos, path[pos], path[next]);
            println!("    Ángulo: {:.2}° ({:.3} rad)", angle * 180.0 / std::f32::consts::PI, angle);
            println!("    Angle score: {:.4}", angle_score);
            println!("    Costo inserción: {:.2}", cost);
            println!("    Longitud arista: {:.2}", edge_len);
            println!("    Cost penalty: {:.4}", cost_penalty);
            println!("    TOTAL SCORE: {:.4}\n", total_score);

            if total_score > best_score {
                best_score = total_score;
                best_pos = pos;
            }
        }

        println!("  → MEJOR POSICIÓN: {} (score: {:.4})\n", best_pos, best_score);
        println!("─────────────────────────────────────────────────────────────\n");
    }

    println!("Este análisis muestra cómo V8.6 evalúa cada candidato.");
    println!("Para ver el algoritmo completo en acción, revisa el log detallado.");
}
