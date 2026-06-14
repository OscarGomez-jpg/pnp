/// Binario para entrenamiento automático de parámetros V8.5
///
/// Ejecuta grid search sobre instancias TSPLIB para encontrar
/// los pesos óptimos α y β que minimicen el error promedio.
///
/// Uso: cargo run --bin train_params --release
use std::path::Path;
use traveler::calibration::ParameterCalibrator;
use traveler::tsplib::TspInstance;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     Entrenamiento Automático de Parámetros V8.5               ║");
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

    let mut instances = Vec::new();
    for path in &instances_paths {
        match TspInstance::from_file(path) {
            Ok(inst) => {
                println!("✓ Cargada: {} ({} nodos, óptimo: {:?})", 
                    inst.name, inst.dimension, inst.optimal_distance);
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

    println!("\n{} instancias cargadas exitosamente.\n", instances.len());

    let calibrator = ParameterCalibrator::new();
    let result = calibrator.calibrate(&instances);

    let output_path = "assets/v85_calibrated_params.txt";
    match result.save_to_file(output_path) {
        Ok(_) => {
            println!("\n✓ Parámetros guardados en: {}", output_path);
            println!("  V8.5 usará automáticamente estos parámetros calibrados.");
        }
        Err(e) => {
            eprintln!("\n✗ Error guardando parámetros: {}", e);
        }
    }
}
