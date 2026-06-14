use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

fn main() {
    println!("Test de Super Bubble Removal en berlin52\n");

    let instance = TspInstance::from_file("assets/berlin52.tsp").unwrap();
    let params = V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    let mut strategy = TriangleInsertionV86::with_params(params);
    let mut path = Vec::new();
    let mut steps = 0;

    loop {
        let finished = strategy.execute_step(&mut path, &instance.nodes);
        steps += 1;
        if finished || steps > instance.nodes.len() + 500 {
            break;
        }
    }

    let distance = path_distance(&path, &instance.nodes);
    let optimal = 7542.0;
    let error = ((distance - optimal) / optimal) * 100.0;

    println!("Nodos: {}", instance.dimension);
    println!("Pasos: {}", steps);
    println!("Distancia: {:.2}", distance);
    println!("Óptimo: {:.2}", optimal);
    println!("Error: {:.2}%", error);
    println!("\nTour: {:?}", path);
}
