/// Tests de Benchmarks Visuales
///
/// Ejecuta comparativas de todas las estrategias contra soluciones óptimas conocidas
/// y muestra resultados en formato visual/tabular.
///
/// Ejecutar con: cargo test --test benchmarks -- --nocapture
use traveler::{
    benchmarks::{create_benchmark_suite, format_results_table, run_benchmark, summary_ranking},
    strategies::create_registry,
};

#[test]
fn benchmark_all_strategies_on_all_problems() {
    println!("\n");
    println!(
        "╔════════════════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                    🚀 BENCHMARK SUITE - COMPARATIVA DE ESTRATEGIAS 🚀                         ║"
    );
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════════════════════╝"
    );

    let registry = create_registry();
    let suite = create_benchmark_suite();
    let mut all_results = Vec::new();

    println!("\n📋 CONFIGURACIÓN:");
    println!("   Estrategias: {}", registry.list_ids().len());
    println!("   Problemas: {}", suite.len());

    println!("\n⏳ Ejecutando benchmarks...\n");

    for problem in suite {
        println!("   ▶ {:<30}", problem.name);

        for strategy_id in registry.list_ids() {
            let mut strategy = registry.get_strategy(strategy_id).unwrap();
            let result = run_benchmark(&mut *strategy, strategy_id, &problem);
            all_results.push(result);
        }
    }

    // Mostrar tabla de resultados
    let table = format_results_table(&all_results);
    println!("{}", table);

    // Mostrar ranking
    let ranking = summary_ranking(&all_results);
    println!("{}", ranking);

    // Estadísticas finales
    println!("\n📈 ESTADÍSTICAS GLOBALES:\n");

    let best_result = all_results
        .iter()
        .min_by(|a, b| {
            a.approximation_ratio
                .partial_cmp(&b.approximation_ratio)
                .unwrap()
        })
        .unwrap();
    let worst_result = all_results
        .iter()
        .max_by(|a, b| {
            a.approximation_ratio
                .partial_cmp(&b.approximation_ratio)
                .unwrap()
        })
        .unwrap();
    let avg_ratio = all_results
        .iter()
        .map(|r| r.approximation_ratio)
        .sum::<f32>()
        / all_results.len() as f32;

    println!(
        "   ✅ Mejor resultado: {} en {} (ratio: {:.4})",
        best_result.strategy_name, best_result.problem_name, best_result.approximation_ratio
    );
    println!(
        "   ❌ Peor resultado:  {} en {} (ratio: {:.4})",
        worst_result.strategy_name, worst_result.problem_name, worst_result.approximation_ratio
    );
    println!("   📊 Ratio promedio:  {:.4}", avg_ratio);

    let excellent_count = all_results
        .iter()
        .filter(|r| r.approximation_ratio <= 1.05)
        .count();
    let good_count = all_results
        .iter()
        .filter(|r| r.approximation_ratio <= 1.20)
        .count();

    println!(
        "   ⭐ Soluciones excelentes (≤5%):  {}/{}",
        excellent_count,
        all_results.len()
    );
    println!(
        "   👍 Soluciones buenas (≤20%):     {}/{}",
        good_count,
        all_results.len()
    );

    println!("\n");
}

#[test]
fn benchmark_nearest_neighbor_details() {
    println!("\n");
    println!(
        "╔════════════════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                      📊 BENCHMARK DETALLADO: NEAREST NEIGHBOR 📊                             ║"
    );
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════════════════════╝"
    );

    let registry = create_registry();
    let suite = create_benchmark_suite();
    let mut strategy = registry.get_strategy("nearest_neighbor").unwrap();

    println!("\n🔍 Analizando Nearest Neighbor (Greedy)...\n");

    for problem in suite {
        println!("   Problema: {}", problem.name);
        println!("   Nodos: {}", problem.nodes.len());

        let result = run_benchmark(&mut *strategy, "nearest_neighbor", &problem);

        println!("      Distancia encontrada:  {:.2}", result.distance);
        println!(
            "      Distancia óptima:      {:.2}",
            result.optimal_distance
        );
        println!(
            "      Ratio:                 {:.4}x el óptimo",
            result.approximation_ratio
        );
        println!(
            "      Overhead:              {:.2}%",
            result.overhead_percentage()
        );
        println!(
            "      Calidad:               {} {}\n",
            match result.quality() {
                q if q.contains("Excelente") => "⭐",
                q if q.contains("Muy bueno") => "✨",
                q if q.contains("Bueno") => "👍",
                q if q.contains("Aceptable") => "🤔",
                _ => "❌",
            },
            result.quality()
        );
    }
}

#[test]
fn benchmark_triangle_insertion_details() {
    println!("\n");
    println!(
        "╔════════════════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║               📊 BENCHMARK DETALLADO: TRIANGLE INSERTION 📊                                  ║"
    );
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════════════════════╝"
    );

    let registry = create_registry();
    let suite = create_benchmark_suite();
    let mut strategy = registry.get_strategy("triangle_insertion").unwrap();

    println!("\n🔍 Analizando Triangle Insertion (Smart Insertion)...\n");

    for problem in suite {
        println!("   Problema: {}", problem.name);
        println!("   Nodos: {}", problem.nodes.len());

        if problem.nodes.len() < 3 {
            println!("      ⚠️  Saltado (requiere al menos 3 nodos)\n");
            continue;
        }

        let result = run_benchmark(&mut *strategy, "triangle_insertion", &problem);

        println!("      Distancia encontrada:  {:.2}", result.distance);
        println!(
            "      Distancia óptima:      {:.2}",
            result.optimal_distance
        );
        println!(
            "      Ratio:                 {:.4}x el óptimo",
            result.approximation_ratio
        );
        println!(
            "      Overhead:              {:.2}%",
            result.overhead_percentage()
        );
        println!(
            "      Calidad:               {} {}",
            match result.quality() {
                q if q.contains("Excelente") => "⭐",
                q if q.contains("Muy bueno") => "✨",
                q if q.contains("Bueno") => "👍",
                q if q.contains("Aceptable") => "🤔",
                _ => "❌",
            },
            result.quality()
        );

        strategy.reset();
        println!();
    }
}

#[test]
fn benchmark_comparison_table() {
    println!("\n");
    println!(
        "╔════════════════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                    📋 TABLA COMPARATIVA: SIDE-BY-SIDE 📋                                     ║"
    );
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════════════════════╝"
    );

    let registry = create_registry();
    let suite = create_benchmark_suite();

    println!("\n{}", "═".repeat(100));
    println!("PROBLEMA | NN DIST  | NN RATIO | TRIANGLE DIST | TRIANGLE RATIO | MEJOR | MEJORA");
    println!("{}", "═".repeat(100));

    for problem in suite {
        if problem.nodes.len() < 3 {
            continue; // Triangle insertion requiere 3+ nodos
        }

        let mut nn_strategy = registry.get_strategy("nearest_neighbor").unwrap();
        let nn_result = run_benchmark(&mut *nn_strategy, "nearest_neighbor", &problem);

        let mut ti_strategy = registry.get_strategy("triangle_insertion").unwrap();
        let ti_result = run_benchmark(&mut *ti_strategy, "triangle_insertion", &problem);

        let mejor = if nn_result.distance <= ti_result.distance {
            "NN"
        } else {
            "TI"
        };

        let mejora_porcentaje = ((nn_result.distance - ti_result.distance).abs()
            / nn_result.distance.max(ti_result.distance))
            * 100.0;

        println!(
            "{:<8} | {:<8.2} | {:<8.4} | {:<13.2} | {:<14.4} | {:<6} | {:.2}%",
            problem.name,
            nn_result.distance,
            nn_result.approximation_ratio,
            ti_result.distance,
            ti_result.approximation_ratio,
            mejor,
            mejora_porcentaje
        );
    }

    println!("{}", "═".repeat(100));
}

#[test]
fn benchmark_scalability_test() {
    println!("\n");
    println!(
        "╔════════════════════════════════════════════════════════════════════════════════════════════════╗"
    );
    println!(
        "║                      📈 TEST DE ESCALABILIDAD 📈                                             ║"
    );
    println!(
        "╚════════════════════════════════════════════════════════════════════════════════════════════════╝"
    );

    let registry = create_registry();

    println!("\n🔬 Probando con diferentes números de nodos...\n");

    for num_nodes in [5, 10, 15, 20] {
        // Generar nodos en círculo (óptimo: seguir el perímetro)
        let radius = 50.0;
        let nodes: Vec<_> = (0..num_nodes)
            .map(|i| {
                let angle = (i as f32) * 2.0 * std::f32::consts::PI / num_nodes as f32;
                traveler::core::Node::new(
                    100.0 + angle.cos() * radius,
                    100.0 + angle.sin() * radius,
                )
            })
            .collect();

        let optimal_distance = 2.0 * std::f32::consts::PI * radius;

        let problem = traveler::benchmarks::BenchmarkProblem::new(
            &format!("Círculo ({} nodos)", num_nodes),
            nodes,
            optimal_distance,
            (0..num_nodes).collect(),
        );

        println!("   Problema: {} nodos", num_nodes);

        for strategy_id in registry.list_ids() {
            let mut strategy = registry.get_strategy(strategy_id).unwrap();
            let result = run_benchmark(&mut *strategy, strategy_id, &problem);

            let overhead = result.overhead_percentage();
            let status = if overhead <= 5.0 {
                "✅"
            } else if overhead <= 20.0 {
                "⚠️"
            } else {
                "❌"
            };

            println!(
                "      {} {:<30}: ratio={:.4} overhead={:.1}%",
                status, result.strategy_name, result.approximation_ratio, overhead
            );
        }
        println!();
    }
}
