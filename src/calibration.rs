/// Módulo de calibración automática de parámetros para V8.5
///
/// Implementa grid search sobre instancias TSPLIB para encontrar
/// los pesos óptimos α y β que minimicen el error promedio.
use crate::core::path_distance;
use crate::strategies::triangle_insertion_v8_5::TriangleInsertionV85;
use crate::strategies::Strategy;
use crate::tsplib::TspInstance;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CalibratedParams {
    pub alpha: f32,
    pub beta: f32,
    pub avg_error: f32,
    pub instances_tested: usize,
}

impl CalibratedParams {
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = format!(
            "# Parámetros calibrados para V8.5\n\
             # Generado automáticamente por grid search\n\
             alpha: {}\n\
             beta: {}\n\
             avg_error: {}\n\
             instances_tested: {}\n",
            self.alpha, self.beta, self.avg_error, self.instances_tested
        );
        fs::write(path, content).map_err(|e| format!("Error escribiendo archivo: {}", e))
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Error leyendo archivo: {}", e))?;

        let mut alpha = 0.3f32;
        let mut beta = 0.2f32;
        let mut avg_error = 0.0f32;
        let mut instances_tested = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "alpha" => alpha = value.parse().unwrap_or(0.3),
                    "beta" => beta = value.parse().unwrap_or(0.2),
                    "avg_error" => avg_error = value.parse().unwrap_or(0.0),
                    "instances_tested" => instances_tested = value.parse().unwrap_or(0),
                    _ => {}
                }
            }
        }

        Ok(CalibratedParams {
            alpha,
            beta,
            avg_error,
            instances_tested,
        })
    }
}

pub struct ParameterCalibrator {
    alpha_range: Vec<f32>,
    beta_range: Vec<f32>,
    optima: HashMap<String, f64>,
}

impl ParameterCalibrator {
    pub fn new() -> Self {
        Self {
            alpha_range: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
            beta_range: vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6],
            optima: Self::load_optima(),
        }
    }

    fn load_optima() -> HashMap<String, f64> {
        let mut optima = HashMap::new();
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

    pub fn with_range(mut self, alpha_range: Vec<f32>, beta_range: Vec<f32>) -> Self {
        self.alpha_range = alpha_range;
        self.beta_range = beta_range;
        self
    }

    /// Ejecuta grid search sobre las instancias proporcionadas
    pub fn calibrate(&self, instances: &[TspInstance]) -> CalibratedParams {
        let mut best_params = CalibratedParams {
            alpha: 0.3,
            beta: 0.2,
            avg_error: f32::MAX,
            instances_tested: 0,
        };

        println!("Iniciando calibración con {} instancias...", instances.len());
        println!("Range α: {:?}, Range β: {:?}", self.alpha_range, self.beta_range);
        println!("Total combinaciones: {}\n", self.alpha_range.len() * self.beta_range.len());

        for &alpha in &self.alpha_range {
            for &beta in &self.beta_range {
                let avg_error = self.evaluate_params(instances, alpha, beta);

                if avg_error < best_params.avg_error {
                    best_params.alpha = alpha;
                    best_params.beta = beta;
                    best_params.avg_error = avg_error;
                    best_params.instances_tested = instances.len();

                    println!(
                        "Nuevo mejor: α={:.2}, β={:.2} → Error promedio: {:.2}%",
                        alpha, beta, avg_error
                    );
                }
            }
        }

        println!("\n=== Calibración completada ===");
        println!("Mejores parámetros: α={:.2}, β={:.2}", best_params.alpha, best_params.beta);
        println!("Error promedio: {:.2}%", best_params.avg_error);

        best_params
    }

    /// Evalúa un conjunto de parámetros sobre todas las instancias
    fn evaluate_params(&self, instances: &[TspInstance], alpha: f32, beta: f32) -> f32 {
        let mut total_error = 0.0f64;
        let mut valid_instances = 0;

        for instance in instances {
            if let Some(&optimal) = self.optima.get(&instance.name) {
                let distance = self.run_strategy(instance, alpha, beta);
                let error = ((distance - optimal as f32) / optimal as f32) * 100.0;
                total_error += error as f64;
                valid_instances += 1;
            }
        }

        if valid_instances > 0 {
            (total_error / valid_instances as f64) as f32
        } else {
            f32::MAX
        }
    }

    /// Ejecuta V8.5 con parámetros específicos sobre una instancia
    fn run_strategy(&self, instance: &TspInstance, alpha: f32, beta: f32) -> f32 {
        let mut strategy = TriangleInsertionV85::new();
        strategy.set_params(alpha, beta);

        let mut path = Vec::new();
        let mut steps = 0;
        loop {
            let finished = strategy.execute_step(&mut path, &instance.nodes);
            steps += 1;
            if finished || steps > instance.nodes.len() + 500 {
                break;
            }
        }

        path_distance(&path, &instance.nodes)
    }
}

impl Default for ParameterCalibrator {
    fn default() -> Self {
        Self::new()
    }
}
