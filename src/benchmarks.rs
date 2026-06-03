/// Módulo de Benchmarks y Comparativas
///
/// Compara la eficiencia de diferentes estrategias contra soluciones óptimas conocidas,
/// mostrando resultados de forma visual para fácil análisis.
use crate::core::{Node, path_distance};
use crate::strategies::Strategy;

/// Información sobre un problema de prueba con su solución óptima conocida
#[derive(Debug, Clone)]
pub struct BenchmarkProblem {
    pub name: String,
    pub nodes: Vec<Node>,
    pub optimal_distance: f32,
    pub optimal_path: Vec<usize>,
}

impl BenchmarkProblem {
    pub fn new(
        name: &str,
        nodes: Vec<Node>,
        optimal_distance: f32,
        optimal_path: Vec<usize>,
    ) -> Self {
        Self {
            name: name.to_string(),
            nodes,
            optimal_distance,
            optimal_path,
        }
    }
}

/// Resultado de ejecutar una estrategia en un problema
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub strategy_name: String,
    pub strategy_id: String,
    pub problem_name: String,
    pub distance: f32,
    pub optimal_distance: f32,
    pub approximation_ratio: f32, // distance / optimal
    pub steps_taken: usize,
}

impl BenchmarkResult {
    pub fn new(
        strategy_name: &str,
        strategy_id: &str,
        problem_name: &str,
        distance: f32,
        optimal_distance: f32,
        steps_taken: usize,
    ) -> Self {
        let approximation_ratio = distance / optimal_distance;
        Self {
            strategy_name: strategy_name.to_string(),
            strategy_id: strategy_id.to_string(),
            problem_name: problem_name.to_string(),
            distance,
            optimal_distance,
            approximation_ratio,
            steps_taken,
        }
    }

    /// Calcula qué porcentaje peor es que el óptimo
    pub fn overhead_percentage(&self) -> f32 {
        (self.approximation_ratio - 1.0) * 100.0
    }

    /// Determina si el resultado es "bueno" (cerca del óptimo)
    pub fn quality(&self) -> &'static str {
        match self.approximation_ratio {
            r if r <= 1.05 => "Excelente (≤5% peor)",
            r if r <= 1.10 => "Muy bueno (≤10% peor)",
            r if r <= 1.20 => "Bueno (≤20% peor)",
            r if r <= 1.50 => "Aceptable (≤50% peor)",
            _ => "Pobre (>50% peor)",
        }
    }
}

/// Suite de problemas de benchmark
pub fn create_benchmark_suite() -> Vec<BenchmarkProblem> {
    let mut suite = Vec::new();

    // Problema 1: Triángulo Simple (3 nodos)
    // Óptimo: cualquier permutación = 12.0 (triángulo 3-4-5)
    suite.push(BenchmarkProblem::new(
        "Triángulo 3-4-5",
        vec![
            Node::new(0.0, 0.0),
            Node::new(3.0, 0.0),
            Node::new(0.0, 4.0),
        ],
        12.0,
        vec![0, 1, 2],
    ));

    // Problema 2: Cuadrado (4 nodos)
    // Óptimo: seguir el perímetro = 40.0
    suite.push(BenchmarkProblem::new(
        "Cuadrado 10x10",
        vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ],
        40.0,
        vec![0, 1, 2, 3],
    ));

    // Problema 3: Línea recta (5 nodos)
    // Óptimo: ir de un extremo al otro y volver = 160.0
    suite.push(BenchmarkProblem::new(
        "Línea Recta (5 puntos)",
        vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(20.0, 0.0),
            Node::new(30.0, 0.0),
            Node::new(40.0, 0.0),
        ],
        80.0,
        vec![0, 4, 3, 2, 1],
    ));

    // Problema 4: Círculo Pequeño (6 nodos)
    // Aproximadamente perímetro del círculo
    let r = 50.0;
    let circle_nodes: Vec<Node> = (0..6)
        .map(|i| {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / 6.0;
            Node::new(100.0 + angle.cos() * r, 100.0 + angle.sin() * r)
        })
        .collect();
    // Perímetro del círculo: 2πr ≈ 314.16
    suite.push(BenchmarkProblem::new(
        "Círculo (6 puntos)",
        circle_nodes,
        314.16,
        vec![0, 1, 2, 3, 4, 5],
    ));

    // Problema 5: Puntos Aleatorios pero Conocidos (pequeño)
    suite.push(BenchmarkProblem::new(
        "Puntos Aleatorios (8 puntos)",
        vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 5.0),
            Node::new(20.0, 0.0),
            Node::new(15.0, 15.0),
            Node::new(5.0, 15.0),
            Node::new(-5.0, 10.0),
            Node::new(-10.0, 0.0),
            Node::new(-5.0, -5.0),
        ],
        71.8, // Aproximado
        vec![0, 2, 3, 4, 1, 5, 6, 7],
    ));

    suite
}

/// Ejecuta un benchmark completo de una estrategia
pub fn run_benchmark(
    strategy: &mut dyn Strategy,
    strategy_id: &str,
    problem: &BenchmarkProblem,
) -> BenchmarkResult {
    let mut path = vec![];
    let mut steps = 0;

    // Ejecutar la estrategia hasta que termine
    for _ in 0..1000 {
        // Max 1000 pasos de seguridad
        let finished = strategy.execute_step(&mut path, &problem.nodes);
        steps += 1;
        if finished {
            break;
        }
    }

    let distance = path_distance(&path, &problem.nodes);

    BenchmarkResult::new(
        strategy.name(),
        strategy_id,
        &problem.name,
        distance,
        problem.optimal_distance,
        steps,
    )
}

/// Formatea tabla de comparativa para impresión
pub fn format_results_table(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();

    // Agrupar por problema
    let mut problems: std::collections::BTreeMap<String, Vec<&BenchmarkResult>> =
        std::collections::BTreeMap::new();

    for result in results {
        problems
            .entry(result.problem_name.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    // Imprimir tabla para cada problema
    for (problem_name, problem_results) in problems {
        output.push_str(&format!("\n{}\n", "=".repeat(100)));
        output.push_str(&format!("📊 PROBLEMA: {}\n", problem_name));
        output.push_str(&format!("{}\n", "=".repeat(100)));

        // Header
        output.push_str(&format!(
            "{:<25} {:<12} {:<12} {:<12} {:<15} {:<20}\n",
            "Estrategia", "Distancia", "Óptimo", "Ratio", "Overhead", "Calidad"
        ));
        output.push_str(&format!("{}\n", "-".repeat(100)));

        // Resultados ordenados por distancia (mejor primero)
        let mut sorted = problem_results.clone();
        sorted.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());

        for (idx, result) in sorted.iter().enumerate() {
            let mark = if idx == 0 { "🏆 " } else { "   " };
            let quality_emoji = match result.quality() {
                q if q.contains("Excelente") => "⭐",
                q if q.contains("Muy bueno") => "✨",
                q if q.contains("Bueno") => "👍",
                q if q.contains("Aceptable") => "🤔",
                _ => "❌",
            };

            output.push_str(&format!(
                "{}{:<22} {:<12.2} {:<12.2} {:<12.4} {:<15.2}% {:<20}\n",
                mark,
                result.strategy_name,
                result.distance,
                result.optimal_distance,
                result.approximation_ratio,
                result.overhead_percentage(),
                format!("{} {}", quality_emoji, result.quality())
            ));
        }

        // Estadísticas
        if !sorted.is_empty() {
            let best = sorted[0].distance;
            let worst = sorted[sorted.len() - 1].distance;
            let avg = sorted.iter().map(|r| r.distance).sum::<f32>() / sorted.len() as f32;

            output.push_str(&format!("{}\n", "-".repeat(100)));
            output.push_str(&format!(
                "Mejor: {:.2} | Peor: {:.2} | Promedio: {:.2} | Rango: {:.2}\n",
                best,
                worst,
                avg,
                worst - best
            ));
        }
    }

    output
}

/// Resumen de ranking global de estrategias
pub fn summary_ranking(results: &[BenchmarkResult]) -> String {
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "=".repeat(80)));
    output.push_str(&"🏅 RANKING GLOBAL\n");
    output.push_str(&format!("{}\n", "=".repeat(80)));

    // Agrupar por estrategia
    let mut strategies: std::collections::BTreeMap<String, Vec<&BenchmarkResult>> =
        std::collections::BTreeMap::new();

    for result in results {
        strategies
            .entry(result.strategy_name.clone())
            .or_insert_with(Vec::new)
            .push(result);
    }

    // Calcular puntuación de cada estrategia
    let mut rankings: Vec<(String, f32, f32)> = strategies
        .into_iter()
        .map(|(name, results)| {
            let avg_ratio =
                results.iter().map(|r| r.approximation_ratio).sum::<f32>() / results.len() as f32;
            let num_wins = results
                .iter()
                .filter(|r| r.approximation_ratio <= 1.05)
                .count() as f32;
            (name, avg_ratio, num_wins)
        })
        .collect();

    rankings.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    output.push_str(&format!(
        "{:<30} {:<15} {:<15}\n",
        "Estrategia", "Ratio Promedio", "Soluciones ≤5%"
    ));
    output.push_str(&format!("{}\n", "-".repeat(80)));

    for (idx, (name, avg_ratio, wins)) in rankings.iter().enumerate() {
        let medal = match idx {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };

        output.push_str(&format!(
            "{} {:<28} {:<15.4} {:<15.0}\n",
            medal, name, avg_ratio, wins
        ));
    }

    output.push_str(&format!("{}\n", "=".repeat(80)));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_problem_creation() {
        let suite = create_benchmark_suite();
        assert!(!suite.is_empty(), "Suite should have problems");
        assert!(suite.len() >= 3, "Suite should have at least 3 problems");
    }

    #[test]
    fn test_benchmark_result_quality() {
        let result_excellent = BenchmarkResult::new("Test", "test", "Problem", 105.0, 100.0, 10);
        assert!(result_excellent.quality().contains("Excelente"));

        let result_poor = BenchmarkResult::new("Test", "test", "Problem", 200.0, 100.0, 10);
        assert!(result_poor.quality().contains("Pobre"));
    }

    #[test]
    fn test_overhead_percentage() {
        let result = BenchmarkResult::new("Test", "test", "Problem", 120.0, 100.0, 10);
        assert!((result.overhead_percentage() - 20.0).abs() < 0.1);
    }
}
