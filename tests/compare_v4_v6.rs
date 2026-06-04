use traveler::core::{Node, path_distance};
use traveler::strategies::create_registry;

struct SimpleRng {
    state: u64,
}
impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 32) as u32) as f32 / (u32::MAX as f32)
    }
}

fn run_strategy(
    registry: &traveler::strategies::StrategyRegistry,
    id: &str,
    nodes: &[Node],
    max_steps: usize,
) -> f32 {
    let mut strat = registry.get_strategy(id).unwrap();
    let mut path = vec![];
    for _ in 0..max_steps {
        if strat.execute_step(&mut path, nodes) {
            break;
        }
    }
    path_distance(&path, nodes)
}

fn gen_nodes(rng: &mut SimpleRng, n: usize, scale: f32) -> Vec<Node> {
    (0..n)
        .map(|_| Node::new(rng.next_f32() * scale, rng.next_f32() * scale))
        .collect()
}

#[test]
fn compare_v4_v6_large_scenarios() {
    let registry = create_registry();
    let mut rng = SimpleRng::new(12345);

    println!("\n{:=<72}", "");
    println!("  BENCHMARK V4 vs V6 — Escalado de Nodos");
    println!("{:=<72}", "");
    println!("{:<10} {:>8} {:>8} {:>10} {:>10} {:>12}", "Nodos", "V4 gana", "V6 gana", "V4 avg", "V6 avg", "Mejora V6");
    println!("{:-<72}", "");

    for (n_nodes, tests) in [(15, 100), (25, 100), (50, 60), (75, 40), (100, 20)] {
        let mut v4_wins = 0;
        let mut v6_wins = 0;
        let mut ties = 0;
        let mut v4_total = 0.0f64;
        let mut v6_total = 0.0f64;
        let scale = (n_nodes as f32) * 30.0;

        for _ in 0..tests {
            let nodes = gen_nodes(&mut rng, n_nodes, scale);
            let dist_v4 = run_strategy(&registry, "triangle_insertion_v4", &nodes, n_nodes + 20) as f64;
            let dist_v6 = run_strategy(&registry, "triangle_insertion_v6", &nodes, n_nodes + 20) as f64;

            v4_total += dist_v4;
            v6_total += dist_v6;

            if (dist_v4 - dist_v6).abs() < 0.1 {
                ties += 1;
            } else if dist_v4 < dist_v6 {
                v4_wins += 1;
            } else {
                v6_wins += 1;
            }
        }

        let n = tests as f64;
        let mejora = ((v4_total - v6_total) / v4_total) * 100.0;
        println!(
            "{:<10} {:>8} {:>8} {:>10.1} {:>10.1} {:>11.4}%",
            n_nodes, v4_wins, v6_wins, v4_total / n, v6_total / n, mejora
        );
    }

    println!("{:=<72}", "");
}
