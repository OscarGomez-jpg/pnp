#![allow(unused)]
/// Estrategia: Lin-Kernighan-Helsgaun (LKH) via ejecutable externo
///
/// Invoca el ejecutable LKH-3.0.14 compilado para resolver instancias TSP.
/// Genera archivos temporales en formato TSPLIB, ejecuta LKH, y lee la solución.
use super::Strategy;
use crate::core::{Node, path_distance};
use std::fs;
use std::io::Write;
use std::process::Command;

pub struct LinKernighan {
    solved: bool,
    solved_path: Vec<usize>,
    lkh_path: String,
    work_dir: String,
}

impl LinKernighan {
    pub fn new() -> Self {
        Self {
            solved: false,
            solved_path: Vec::new(),
            lkh_path: "./LKH-3.0.14/LKH".to_string(),
            work_dir: "/tmp/lkh_work".to_string(),
        }
    }

    fn ensure_work_dir(&self) {
        let _ = fs::create_dir_all(&self.work_dir);
    }

    fn write_problem_file(&self, nodes: &[Node]) -> String {
        let path = format!("{}/problem.tsp", self.work_dir);
        let mut file = fs::File::create(&path).expect("No se pudo crear problem.tsp");

        writeln!(file, "NAME : traveler_instance").unwrap();
        writeln!(file, "TYPE : TSP").unwrap();
        writeln!(file, "DIMENSION : {}", nodes.len()).unwrap();
        writeln!(file, "EDGE_WEIGHT_TYPE : EUC_2D").unwrap();
        writeln!(file, "NODE_COORD_SECTION").unwrap();

        for (i, node) in nodes.iter().enumerate() {
            writeln!(file, "{} {} {}", i + 1, node.pos.x, node.pos.y).unwrap();
        }

        writeln!(file, "EOF").unwrap();
        path
    }

    fn write_param_file(&self, problem_path: &str) -> String {
        let path = format!("{}/params.par", self.work_dir);
        let mut file = fs::File::create(&path).expect("No se pudo crear params.par");

        writeln!(file, "PROBLEM_FILE = {}", problem_path).unwrap();
        writeln!(file, "TOUR_FILE = {}/solution.tour", self.work_dir).unwrap();
        writeln!(file, "RUNS = 1").unwrap();
        writeln!(file, "MAX_TRIALS = 1").unwrap();
        writeln!(file, "SEED = 42").unwrap();
        writeln!(file, "OUTPUT_TOUR_FILE = {}/solution.tour", self.work_dir).unwrap();

        path
    }

    fn read_solution(&self, nodes: &[Node]) -> Vec<usize> {
        let tour_path = format!("{}/solution.tour", self.work_dir);
        let content = match fs::read_to_string(&tour_path) {
            Ok(c) => c,
            Err(_) => return (0..nodes.len()).collect(),
        };

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

        if indices.len() != nodes.len() {
            return (0..nodes.len()).collect();
        }

        indices
    }

    fn solve_with_lkh(&mut self, nodes: &[Node]) -> Vec<usize> {
        self.ensure_work_dir();

        let problem_path = self.write_problem_file(nodes);
        let _param_path = self.write_param_file(&problem_path);

        let result = Command::new(&self.lkh_path)
            .arg(format!("{}/params.par", self.work_dir))
            .output();

        match result {
            Ok(output) => {
                if !output.status.success() {
                    eprintln!(
                        "LKH error: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return (0..nodes.len()).collect();
                }
            }
            Err(e) => {
                eprintln!("No se pudo ejecutar LKH: {}", e);
                return (0..nodes.len()).collect();
            }
        }

        self.read_solution(nodes)
    }
}

impl Strategy for LinKernighan {
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        if self.solved {
            *current_path = self.solved_path.clone();
            return true;
        }

        if nodes.len() < 3 {
            current_path.extend(0..nodes.len());
            return true;
        }

        self.solved_path = self.solve_with_lkh(nodes);
        *current_path = self.solved_path.clone();
        self.solved = true;
        true
    }

    fn name(&self) -> &str {
        "LKH-3.0.14 (Ejecutable oficial)"
    }

    fn reset(&mut self) {
        self.solved = false;
        self.solved_path.clear();
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lkh_executable() {
        let nodes = vec![
            Node::new(0.0, 0.0),
            Node::new(10.0, 0.0),
            Node::new(10.0, 10.0),
            Node::new(0.0, 10.0),
        ];

        let mut lk = LinKernighan::new();
        let mut path = Vec::new();
        let finished = lk.execute_step(&mut path, &nodes);

        assert!(finished);
        assert_eq!(path.len(), 4);

        let dist = path_distance(&path, &nodes);
        assert!((dist - 40.0).abs() < 1.0, "Tour óptimo debe ser ~40");
    }
}
