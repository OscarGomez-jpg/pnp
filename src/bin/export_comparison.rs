use std::fs;
use std::io::Write;
use std::process::Command;
use traveler::core::path_distance;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::triangle_insertion_v9::{TriangleInsertionV9, V9Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

fn run_strategy(nodes: &[traveler::core::Node], s: &mut dyn Strategy) -> (Vec<usize>, f32) {
    let mut path = vec![];
    let mut steps = 0;
    loop {
        if s.execute_step(&mut path, nodes) || steps > nodes.len() + 500 {
            break;
        }
        steps += 1;
    }
    let dist = path_distance(&path, nodes);
    (path, dist)
}

fn run_lkh(instance: &TspInstance) -> Option<(Vec<usize>, f32)> {
    let dir = "/tmp/lkh_json";
    let _ = fs::create_dir_all(dir);

    let tsp = format!("{}/p_{}.tsp", dir, instance.name);
    let mut f = fs::File::create(&tsp).ok()?;
    writeln!(f, "NAME : {}", instance.name).ok()?;
    writeln!(f, "TYPE : TSP").ok()?;
    writeln!(f, "DIMENSION : {}", instance.dimension).ok()?;
    writeln!(f, "EDGE_WEIGHT_TYPE : EUC_2D").ok()?;
    writeln!(f, "NODE_COORD_SECTION").ok()?;
    for (i, n) in instance.nodes.iter().enumerate() {
        writeln!(f, "{} {} {}", i + 1, n.pos.x, n.pos.y).ok()?;
    }
    writeln!(f, "EOF").ok()?;
    drop(f);

    let par = format!("{}/par_{}.par", dir, instance.name);
    let mut pf = fs::File::create(&par).ok()?;
    writeln!(pf, "PROBLEM_FILE = {}", tsp).ok()?;
    writeln!(pf, "TOUR_FILE = {}/{}.tour", dir, instance.name).ok()?;
    writeln!(pf, "RUNS = 1").ok()?;
    writeln!(pf, "MAX_TRIALS = 1").ok()?;
    writeln!(pf, "SEED = 42").ok()?;
    drop(pf);

    let out = Command::new("./LKH-3.0.14/LKH").arg(&par).output().ok()?;
    if !out.status.success() {
        return None;
    }

    let tour_p = format!("{}/{}.tour", dir, instance.name);
    let c = fs::read_to_string(&tour_p).ok()?;
    let mut path = vec![];
    let mut sec = false;
    for l in c.lines() {
        let l = l.trim();
        if l == "TOUR_SECTION" {
            sec = true;
            continue;
        }
        if l == "-1" || l == "EOF" {
            break;
        }
        if sec {
            if let Ok(i) = l.parse::<usize>() {
                if i >= 1 && i <= instance.dimension {
                    path.push(i - 1);
                }
            }
        }
    }
    if path.len() == instance.dimension {
        let dist = path_distance(&path, &instance.nodes);
        Some((path, dist))
    } else {
        None
    }
}

fn main() {
    let configs = [
        ("berlin52", "assets/berlin52.tsp", 7542.0),
        ("eil51", "assets/eil51.tsp", 426.0),
        ("st70", "assets/st70.tsp", 675.0),
        ("kroA100", "assets/kroA100.tsp", 21282.0),
    ];

    let mut json = String::from("[\n");

    for (name, path, opt) in &configs {
        let inst = match TspInstance::from_file(path) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error {}: {}", name, e);
                continue;
            }
        };

        let (v86_path, v86_dist) =
            run_strategy(&inst.nodes, &mut TriangleInsertionV86::with_params(V86Params {
                k_neighbors: 8,
                w_angle: 0.25,
                w_cost: 0.25,
            }));
        let (v9_path, v9_dist) =
            run_strategy(&inst.nodes, &mut TriangleInsertionV9::with_params(V9Params {
                k_neighbors: 8,
                w_angle: 0.40,
                w_cost: 0.30,
                w_density: 0.20,
            }));
        let lkh = run_lkh(&inst);

        json.push_str("  {\n");
        json.push_str(&format!("    \"name\": \"{}\",\n", name));
        json.push_str(&format!("    \"n\": {},\n", inst.dimension));
        json.push_str(&format!("    \"optimal\": {:.2},\n", opt));
        json.push_str("    \"nodes\": [");
        for (i, n) in inst.nodes.iter().enumerate() {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!("[{:.6},{:.6}]", n.pos.x, n.pos.y));
        }
        json.push_str("],\n");

        json.push_str(&format!(
            "    \"v86\": {{\"dist\": {:.2}, \"tour\": [{}]}},\n",
            v86_dist,
            v86_path.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
        ));
        json.push_str(&format!(
            "    \"v9\": {{\"dist\": {:.2}, \"tour\": [{}]}}",
            v9_dist,
            v9_path.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
        ));

        if let Some((lkh_path, lkh_dist)) = lkh {
            json.push_str(&format!(
                ",\n    \"lkh\": {{\"dist\": {:.2}, \"tour\": [{}]}}",
                lkh_dist,
                lkh_path.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")
            ));
        }

        json.push_str("\n  },\n");

        let v86_err = (v86_dist - *opt as f32) / *opt as f32 * 100.0;
        let v9_err = (v9_dist - *opt as f32) / *opt as f32 * 100.0;
        println!(
            "{}: V8.6={:.2} ({:.2}%) V9={:.2} ({:.2}%)",
            name, v86_dist, v86_err, v9_dist, v9_err
        );
    }

    if json.ends_with(",\n") {
        json.truncate(json.len() - 2);
        json.push('\n');
    }
    json.push_str("]\n");

    fs::write("assets/comparison_data.json", &json).unwrap();
    println!("\n✓ Datos guardados en assets/comparison_data.json");
}
