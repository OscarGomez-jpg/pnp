/// Generador de visualización HTML mejorada para debugging de V8.6
use std::fs;
use std::io::Write;
use std::process::Command;
use traveler::core::insertion_cost;
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

struct InsertionStep {
    step: usize,
    candidate: usize,
    position: usize,
    angle: f32,
    angle_score: f32,
    cost: f32,
    cost_penalty: f32,
    total_score: f32,
    path_after: Vec<usize>,
    current_distance: f32,
}

struct OptimizationStep {
    name: String,
    path_before: Vec<usize>,
    path_after: Vec<usize>,
    distance_before: f32,
    distance_after: f32,
    improved: bool,
}

fn run_lkh(instance: &TspInstance) -> Option<Vec<usize>> {
    let work_dir = "/tmp/lkh_viz";
    let _ = fs::create_dir_all(work_dir);

    let problem_path = format!("{}/problem.tsp", work_dir);
    let mut file = fs::File::create(&problem_path).ok()?;

    writeln!(file, "NAME : {}", instance.name).ok()?;
    writeln!(file, "TYPE : TSP").ok()?;
    writeln!(file, "DIMENSION : {}", instance.dimension).ok()?;
    writeln!(file, "EDGE_WEIGHT_TYPE : EUC_2D").ok()?;
    writeln!(file, "NODE_COORD_SECTION").ok()?;
    for (i, node) in instance.nodes.iter().enumerate() {
        writeln!(file, "{} {} {}", i + 1, node.pos.x, node.pos.y).ok()?;
    }
    writeln!(file, "EOF").ok()?;

    let param_path = format!("{}/params.par", work_dir);
    let mut file = fs::File::create(&param_path).ok()?;
    writeln!(file, "PROBLEM_FILE = {}", problem_path).ok()?;
    writeln!(file, "TOUR_FILE = {}/solution.tour", work_dir).ok()?;
    writeln!(file, "RUNS = 1").ok()?;
    writeln!(file, "MAX_TRIALS = 1").ok()?;
    writeln!(file, "SEED = 42").ok()?;

    let result = Command::new("./LKH-3.0.14/LKH")
        .arg(&param_path)
        .output()
        .ok()?;

    if !result.status.success() {
        return None;
    }

    let tour_path = format!("{}/solution.tour", work_dir);
    let content = fs::read_to_string(&tour_path).ok()?;

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

    if indices.len() == instance.nodes.len() {
        Some(indices)
    } else {
        None
    }
}

fn main() {
    println!("════════════════════════════════════════════════════════════════╗");
    println!("║     Generador de Visualización V8.6 (Mejorada)               ║");
    println!("════════════════════════════════════════════════════════════════╝\n");

    let instance_path = "assets/berlin52.tsp";
    let instance = match TspInstance::from_file(instance_path) {
        Ok(inst) => inst,
        Err(e) => {
            eprintln!("Error cargando {}: {}", instance_path, e);
            return;
        }
    };

    println!("Instancia: {} ({} nodos)", instance.name, instance.dimension);

    let params = V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    };

    // Ejecutar LKH para obtener tour de referencia
    print!("Ejecutando LKH para tour de referencia... ");
    let lkh_tour = run_lkh(&instance);
    let lkh_distance = lkh_tour.as_ref().map(|tour| {
        traveler::core::path_distance(tour, &instance.nodes)
    });
    if let Some(dist) = lkh_distance {
        println!("✓ (distancia: {:.2})", dist);
    } else {
        println!("✗ (no disponible)");
    }

    let mut strategy = TriangleInsertionV86::with_params(params);
    let mut path = Vec::new();
    let mut steps = Vec::new();
    let mut opt_steps = Vec::new();
    let mut step_num = 0;

    println!("Ejecutando V8.6 y registrando pasos...\n");

    loop {
        let finished = strategy.execute_step(&mut path, &instance.nodes);
        step_num += 1;

        if !path.is_empty() {
            let new_node = if step_num == 1 {
                path[0]
            } else {
                path.last().copied().unwrap_or(0)
            };

            let position = path.iter().position(|&n| n == new_node).unwrap_or(0);
            let pos_in_path = position;
            let prev_node = if pos_in_path > 0 { path[pos_in_path - 1] } else { path[path.len() - 1] };
            let next_node = path[(pos_in_path + 1) % path.len()];

            let p_i = instance.nodes[prev_node].pos;
            let p_j = instance.nodes[next_node].pos;
            let p_u = instance.nodes[new_node].pos;

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
            let cost = insertion_cost(prev_node, next_node, new_node, &instance.nodes);
            let edge_len = p_i.distance(p_j);
            let cost_ratio = if edge_len > 1e-5 { cost / edge_len } else { 1.0 };
            let cost_penalty = 1.0 / (1.0 + cost_ratio);
            let total_score = angle_score * params.w_angle + cost_penalty * params.w_cost;
            let current_distance = traveler::core::path_distance(&path, &instance.nodes);

            steps.push(InsertionStep {
                step: step_num,
                candidate: new_node,
                position: pos_in_path,
                angle,
                angle_score,
                cost,
                cost_penalty,
                total_score,
                path_after: path.clone(),
                current_distance,
            });
        }

        if finished || step_num > instance.nodes.len() + 10 {
            break;
        }
    }

    println!("Capturados {} pasos de inserción", steps.len());

    let final_distance = traveler::core::path_distance(&path, &instance.nodes);
    let optimal = 7542.0;
    let error = ((final_distance - optimal) / optimal) * 100.0;

    println!("Distancia final: {:.2}", final_distance);
    println!("Óptimo: {:.2}", optimal);
    println!("Error: {:.2}%\n", error);

    let html_path = "v86_visualization.html";
    generate_html(&instance, &steps, &opt_steps, params, html_path, final_distance, optimal, lkh_tour.as_deref(), lkh_distance);

    println!("Visualización generada: {}", html_path);
    println!("Abre el archivo en tu navegador.");
}

fn generate_html(
    instance: &TspInstance,
    steps: &[InsertionStep],
    opt_steps: &[OptimizationStep],
    params: V86Params,
    output_path: &str,
    final_distance: f32,
    optimal: f32,
    lkh_tour: Option<&[usize]>,
    lkh_distance: Option<f32>,
) {
    let mut file = fs::File::create(output_path).unwrap();

    writeln!(file, "<!DOCTYPE html>").unwrap();
    writeln!(file, "<html lang='es'>").unwrap();
    writeln!(file, "<head>").unwrap();
    writeln!(file, "    <meta charset='UTF-8'>").unwrap();
    writeln!(file, "    <meta name='viewport' content='width=device-width, initial-scale=1.0'>").unwrap();
    writeln!(file, "    <title>V8.6 Debug - {}</title>", instance.name).unwrap();
    writeln!(file, "    <style>").unwrap();
    writeln!(file, "        * {{ margin: 0; padding: 0; box-sizing: border-box; }}").unwrap();
    writeln!(file, "        body {{ font-family: 'Segoe UI', Arial, sans-serif; background: #0a0a0a; color: #e0e0e0; height: 100vh; overflow: hidden; }}").unwrap();
    writeln!(file, "        .header {{ background: #1a1a1a; padding: 10px 20px; border-bottom: 2px solid #4CAF50; display: flex; justify-content: space-between; align-items: center; }}").unwrap();
    writeln!(file, "        .header h1 {{ color: #4CAF50; font-size: 1.2em; }}").unwrap();
    writeln!(file, "        .stats {{ display: flex; gap: 20px; font-size: 0.9em; }}").unwrap();
    writeln!(file, "        .stat {{ text-align: center; }}").unwrap();
    writeln!(file, "        .stat-label {{ color: #888; font-size: 0.8em; }}").unwrap();
    writeln!(file, "        .stat-value {{ color: #4CAF50; font-weight: bold; font-size: 1.1em; }}").unwrap();
    writeln!(file, "        .stat-value.error {{ color: #FF5722; }}").unwrap();
    writeln!(file, "        .stat-value.good {{ color: #4CAF50; }}").unwrap();
    writeln!(file, "        .main {{ display: flex; height: calc(100vh - 60px); }}").unwrap();
    writeln!(file, "        .canvas-container {{ flex: 1; position: relative; background: #000; }}").unwrap();
    writeln!(file, "        canvas {{ width: 100%; height: 100%; display: block; }}").unwrap();
    writeln!(file, "        .sidebar {{ width: 320px; background: #1a1a1a; padding: 15px; overflow-y: auto; border-left: 1px solid #333; }}").unwrap();
    writeln!(file, "        .controls {{ display: flex; gap: 5px; margin-bottom: 15px; flex-wrap: wrap; }}").unwrap();
    writeln!(file, "        button {{ background: #333; color: #fff; border: 1px solid #555; padding: 8px 12px; cursor: pointer; border-radius: 4px; font-size: 0.85em; flex: 1; min-width: 60px; }}").unwrap();
    writeln!(file, "        button:hover {{ background: #4CAF50; border-color: #4CAF50; }}").unwrap();
    writeln!(file, "        button.active {{ background: #4CAF50; }}").unwrap();
    writeln!(file, "        .slider-container {{ margin: 10px 0; }}").unwrap();
    writeln!(file, "        input[type='range'] {{ width: 100%; accent-color: #4CAF50; }}").unwrap();
    writeln!(file, "        .step-info {{ background: #222; padding: 12px; border-radius: 6px; margin-bottom: 10px; }}").unwrap();
    writeln!(file, "        .step-title {{ color: #4CAF50; font-weight: bold; margin-bottom: 8px; font-size: 1.1em; }}").unwrap();
    writeln!(file, "        .metric {{ display: flex; justify-content: space-between; padding: 4px 0; border-bottom: 1px solid #333; }}").unwrap();
    writeln!(file, "        .metric:last-child {{ border-bottom: none; }}").unwrap();
    writeln!(file, "        .metric-label {{ color: #aaa; font-size: 0.9em; }}").unwrap();
    writeln!(file, "        .metric-value {{ color: #fff; font-weight: bold; font-size: 0.9em; }}").unwrap();
    writeln!(file, "        .metric-value.highlight {{ color: #4CAF50; }}").unwrap();
    writeln!(file, "        .comparison {{ background: #222; padding: 12px; border-radius: 6px; margin-top: 10px; }}").unwrap();
    writeln!(file, "        .comparison-title {{ color: #FF9800; font-weight: bold; margin-bottom: 8px; }}").unwrap();
    writeln!(file, "        .legend {{ margin-top: 15px; font-size: 0.85em; }}").unwrap();
    writeln!(file, "        .legend-item {{ display: flex; align-items: center; gap: 8px; margin: 4px 0; }}").unwrap();
    writeln!(file, "        .legend-color {{ width: 12px; height: 12px; border-radius: 50%; }}").unwrap();
    writeln!(file, "    </style>").unwrap();
    writeln!(file, "</head>").unwrap();
    writeln!(file, "<body>").unwrap();

    writeln!(file, "    <div class='header'>").unwrap();
    writeln!(file, "        <h1>V8.6 Debug - {}</h1>", instance.name).unwrap();
    writeln!(file, "        <div class='stats'>").unwrap();
    writeln!(file, "            <div class='stat'>").unwrap();
    writeln!(file, "                <div class='stat-label'>Nodos</div>").unwrap();
    writeln!(file, "                <div class='stat-value'>{}</div>", instance.dimension).unwrap();
    writeln!(file, "            </div>").unwrap();
    writeln!(file, "            <div class='stat'>").unwrap();
    writeln!(file, "                <div class='stat-label'>V8.6 Distance</div>").unwrap();
    writeln!(file, "                <div class='stat-value'>{:.0}</div>", final_distance).unwrap();
    writeln!(file, "            </div>").unwrap();
    writeln!(file, "            <div class='stat'>").unwrap();
    writeln!(file, "                <div class='stat-label'>Optimal</div>").unwrap();
    writeln!(file, "                <div class='stat-value'>{:.0}</div>", optimal).unwrap();
    writeln!(file, "            </div>").unwrap();
    writeln!(file, "            <div class='stat'>").unwrap();
    writeln!(file, "                <div class='stat-label'>Error</div>").unwrap();
    let error_class = if final_distance <= optimal * 1.05 { "good" } else { "error" };
    writeln!(file, "                <div class='stat-value {}'>{:.2}%</div>", error_class, ((final_distance - optimal) / optimal) * 100.0).unwrap();
    writeln!(file, "            </div>").unwrap();
    writeln!(file, "        </div>").unwrap();
    writeln!(file, "    </div>").unwrap();

    writeln!(file, "    <div class='main'>").unwrap();
    writeln!(file, "        <div class='canvas-container'>").unwrap();
    writeln!(file, "            <canvas id='tourCanvas'></canvas>").unwrap();
    writeln!(file, "        </div>").unwrap();
    writeln!(file, "        <div class='sidebar'>").unwrap();

    writeln!(file, "            <div class='controls'>").unwrap();
    writeln!(file, "                <button onclick='goToStep(0)'>Inicio</button>").unwrap();
    writeln!(file, "                <button onclick='prevStep()'>Anterior</button>").unwrap();
    writeln!(file, "                <button onclick='nextStep()'>Siguiente</button>").unwrap();
    writeln!(file, "                <button onclick='goToStep(stepsData.length - 1)'>Final</button>").unwrap();
    writeln!(file, "            </div>").unwrap();
    writeln!(file, "            <button onclick='toggleAnimation()' id='animBtn' style='width:100%; margin-bottom:10px;'>▶ Animar</button>").unwrap();
    if lkh_tour.is_some() {
        writeln!(file, "            <button onclick='toggleLkhTour()' id='lkhBtn' style='width:100%; margin-bottom:10px; background:#FF9800;'> Mostrar/Ocultar LKH</button>").unwrap();
    }
    writeln!(file, "            <button onclick='toggleBubbleView()' id='bubbleBtn' style='width:100%; margin-bottom:10px; background:#9C27B0;'> Ver Bubble Removal</button>").unwrap();

    writeln!(file, "            <div class='slider-container'>").unwrap();
    writeln!(file, "                <input type='range' id='stepSlider' min='0' max='{}' value='0' oninput='goToStep(parseInt(this.value))'>", steps.len() - 1).unwrap();
    writeln!(file, "            </div>").unwrap();

    writeln!(file, "            <div class='step-info' id='stepInfo'>").unwrap();
    writeln!(file, "                <div class='step-title'>Paso <span id='stepNum'>0</span> / {}</div>", steps.len() - 1).unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Nodo insertado:</span><span class='metric-value highlight' id='candidate'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Posición:</span><span class='metric-value' id='position'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Ángulo:</span><span class='metric-value' id='angle'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Angle Score:</span><span class='metric-value' id='angleScore'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Costo:</span><span class='metric-value' id='cost'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Cost Penalty:</span><span class='metric-value' id='costPenalty'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Total Score:</span><span class='metric-value highlight' id='totalScore'>-</span></div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Distancia actual:</span><span class='metric-value' id='currentDist'>-</span></div>").unwrap();
    writeln!(file, "            </div>").unwrap();

    writeln!(file, "            <div class='comparison'>").unwrap();
    writeln!(file, "                <div class='comparison-title'>Comparación de Tours</div>").unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>V8.6:</span><span class='metric-value'>{:.0}</span></div>", final_distance).unwrap();
    if let Some(lkh_dist) = lkh_distance {
        writeln!(file, "                <div class='metric'><span class='metric-label'>LKH:</span><span class='metric-value'>{:.0}</span></div>", lkh_dist).unwrap();
    }
    writeln!(file, "                <div class='metric'><span class='metric-label'>Óptimo:</span><span class='metric-value'>{:.0}</span></div>", optimal).unwrap();
    writeln!(file, "                <div class='metric'><span class='metric-label'>Error V8.6:</span><span class='metric-value error'>+{:.0} ({:.1}%)</span></div>", final_distance - optimal, ((final_distance - optimal) / optimal) * 100.0).unwrap();
    if let Some(lkh_dist) = lkh_distance {
        writeln!(file, "                <div class='metric'><span class='metric-label'>Error LKH:</span><span class='metric-value'>+{:.0} ({:.2}%)</span></div>", lkh_dist - optimal, ((lkh_dist - optimal) / optimal) * 100.0).unwrap();
    }
    writeln!(file, "            </div>").unwrap();

    writeln!(file, "            <div class='legend'>").unwrap();
    writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#4CAF50'></div>Tour V8.6</div>").unwrap();
    if lkh_tour.is_some() {
        writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#FF9800'></div>Tour LKH</div>").unwrap();
    }
    writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#9C27B0'></div>Bubble Removal (antes)</div>").unwrap();
    writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#2196F3'></div>Nodos visitados</div>").unwrap();
    writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#FF5722'></div>Nodo actual</div>").unwrap();
    writeln!(file, "                <div class='legend-item'><div class='legend-color' style='background:#666'></div>Nodos pendientes</div>").unwrap();
    writeln!(file, "            </div>").unwrap();

    writeln!(file, "        </div>").unwrap();
    writeln!(file, "    </div>").unwrap();

    writeln!(file, "    <script>").unwrap();

    // Datos de nodos
    writeln!(file, "        const nodes = [").unwrap();
    for (i, node) in instance.nodes.iter().enumerate() {
        writeln!(file, "            {{ x: {:.2}, y: {:.2} }},", node.pos.x, node.pos.y).unwrap();
    }
    writeln!(file, "        ];").unwrap();

    // Datos de pasos
    writeln!(file, "        const stepsData = [").unwrap();
    for (i, step) in steps.iter().enumerate() {
        writeln!(file, "            {{").unwrap();
        writeln!(file, "                step: {},", step.step).unwrap();
        writeln!(file, "                candidate: {},", step.candidate).unwrap();
        writeln!(file, "                position: {},", step.position).unwrap();
        writeln!(file, "                angle: {:.2},", step.angle).unwrap();
        writeln!(file, "                angleScore: {:.4},", step.angle_score).unwrap();
        writeln!(file, "                cost: {:.2},", step.cost).unwrap();
        writeln!(file, "                costPenalty: {:.4},", step.cost_penalty).unwrap();
        writeln!(file, "                totalScore: {:.4},", step.total_score).unwrap();
        writeln!(file, "                currentDist: {:.2},", step.current_distance).unwrap();
        write!(file, "                path: [{}]", step.path_after.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(file).unwrap();
        writeln!(file, "            }}{}", if i < steps.len() - 1 { "," } else { "" }).unwrap();
    }
    writeln!(file, "        ];").unwrap();

    // Tour de LKH
    if let Some(lkh) = lkh_tour {
        writeln!(file, "        const lkhTour = [{}];", lkh.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",")).unwrap();
    } else {
        writeln!(file, "        const lkhTour = null;").unwrap();
    }
    if let Some(lkh_dist) = lkh_distance {
        writeln!(file, "        const lkhDistance = {:.2};", lkh_dist).unwrap();
    } else {
        writeln!(file, "        const lkhDistance = null;").unwrap();
    }

    // Datos de optimización (bubble removal)
    writeln!(file, "        const optStepsData = [").unwrap();
    for (i, step) in opt_steps.iter().enumerate() {
        writeln!(file, "            {{").unwrap();
        writeln!(file, "                name: '{}',", step.name).unwrap();
        writeln!(file, "                distanceBefore: {:.2},", step.distance_before).unwrap();
        writeln!(file, "                distanceAfter: {:.2},", step.distance_after).unwrap();
        writeln!(file, "                improved: {},", step.improved).unwrap();
        write!(file, "                pathBefore: [{}]", step.path_before.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(file).unwrap();
        write!(file, "                pathAfter: [{}]", step.path_after.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(",")).unwrap();
        writeln!(file).unwrap();
        writeln!(file, "            }}{}", if i < opt_steps.len() - 1 { "," } else { "" }).unwrap();
    }
    writeln!(file, "        ];").unwrap();

    writeln!(file, "        const optimalDistance = {:.2};", optimal).unwrap();
    writeln!(file, "        const v86Distance = {:.2};", final_distance).unwrap();

    writeln!(file, "        let currentStep = 0;").unwrap();
    writeln!(file, "        let animInterval = null;").unwrap();
    writeln!(file, "        let showLkhTour = true;").unwrap();
    writeln!(file, "        let showBubbleView = false;").unwrap();
    writeln!(file, "        let currentOptStep = 0;").unwrap();
    writeln!(file, "        const canvas = document.getElementById('tourCanvas');").unwrap();
    writeln!(file, "        const ctx = canvas.getContext('2d');").unwrap();

    writeln!(file, "        function resizeCanvas() {{").unwrap();
    writeln!(file, "            const container = canvas.parentElement;").unwrap();
    writeln!(file, "            canvas.width = container.clientWidth;").unwrap();
    writeln!(file, "            canvas.height = container.clientHeight;").unwrap();
    writeln!(file, "            drawCurrentStep();").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function getScale() {{").unwrap();
    writeln!(file, "            let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;").unwrap();
    writeln!(file, "            for (const n of nodes) {{").unwrap();
    writeln!(file, "                minX = Math.min(minX, n.x); maxX = Math.max(maxX, n.x);").unwrap();
    writeln!(file, "                minY = Math.min(minY, n.y); maxY = Math.max(maxY, n.y);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            const padding = 40;").unwrap();
    writeln!(file, "            const scaleX = (canvas.width - padding * 2) / (maxX - minX);").unwrap();
    writeln!(file, "            const scaleY = (canvas.height - padding * 2) / (maxY - minY);").unwrap();
    writeln!(file, "            return {{ scale: Math.min(scaleX, scaleY), offsetX: padding - minX * Math.min(scaleX, scaleY), offsetY: padding - minY * Math.min(scaleX, scaleY) }};").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function drawCurrentStep() {{").unwrap();
    writeln!(file, "            ctx.clearRect(0, 0, canvas.width, canvas.height);").unwrap();
    writeln!(file, "            const {{ scale, offsetX, offsetY }} = getScale();").unwrap();
    writeln!(file, "            const step = stepsData[currentStep];").unwrap();
    writeln!(file, "            const path = step.path;").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar tour LKH (si está disponible y activado)").unwrap();
    writeln!(file, "            if (showLkhTour && lkhTour) {{").unwrap();
    writeln!(file, "                ctx.strokeStyle = 'rgba(255, 152, 0, 0.4)';").unwrap();
    writeln!(file, "                ctx.lineWidth = 3;").unwrap();
    writeln!(file, "                ctx.setLineDash([5, 5]);").unwrap();
    writeln!(file, "                ctx.beginPath();").unwrap();
    writeln!(file, "                for (let i = 0; i < lkhTour.length; i++) {{").unwrap();
    writeln!(file, "                    const n1 = nodes[lkhTour[i]];").unwrap();
    writeln!(file, "                    const n2 = nodes[lkhTour[(i + 1) % lkhTour.length]];").unwrap();
    writeln!(file, "                    ctx.moveTo(n1.x * scale + offsetX, n1.y * scale + offsetY);").unwrap();
    writeln!(file, "                    ctx.lineTo(n2.x * scale + offsetX, n2.y * scale + offsetY);").unwrap();
    writeln!(file, "                }}").unwrap();
    writeln!(file, "                ctx.stroke();").unwrap();
    writeln!(file, "                ctx.setLineDash([]);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar tour V8.6").unwrap();
    writeln!(file, "            if (path.length > 1) {{").unwrap();
    writeln!(file, "                ctx.strokeStyle = '#4CAF50';").unwrap();
    writeln!(file, "                ctx.lineWidth = 2;").unwrap();
    writeln!(file, "                ctx.beginPath();").unwrap();
    writeln!(file, "                for (let i = 0; i < path.length; i++) {{").unwrap();
    writeln!(file, "                    const n1 = nodes[path[i]];").unwrap();
    writeln!(file, "                    const n2 = nodes[path[(i + 1) % path.length]];").unwrap();
    writeln!(file, "                    ctx.moveTo(n1.x * scale + offsetX, n1.y * scale + offsetY);").unwrap();
    writeln!(file, "                    ctx.lineTo(n2.x * scale + offsetX, n2.y * scale + offsetY);").unwrap();
    writeln!(file, "                }}").unwrap();
    writeln!(file, "                ctx.stroke();").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar nodos").unwrap();
    writeln!(file, "            for (let i = 0; i < nodes.length; i++) {{").unwrap();
    writeln!(file, "                const node = nodes[i];").unwrap();
    writeln!(file, "                const x = node.x * scale + offsetX;").unwrap();
    writeln!(file, "                const y = node.y * scale + offsetY;").unwrap();
    writeln!(file, "                ").unwrap();
    writeln!(file, "                if (i === step.candidate && currentStep > 0) {{").unwrap();
    writeln!(file, "                    ctx.fillStyle = '#FF5722';").unwrap();
    writeln!(file, "                    ctx.beginPath();").unwrap();
    writeln!(file, "                    ctx.arc(x, y, 8, 0, 2 * Math.PI);").unwrap();
    writeln!(file, "                    ctx.fill();").unwrap();
    writeln!(file, "                }} else if (path.includes(i)) {{").unwrap();
    writeln!(file, "                    ctx.fillStyle = '#2196F3';").unwrap();
    writeln!(file, "                    ctx.beginPath();").unwrap();
    writeln!(file, "                    ctx.arc(x, y, 5, 0, 2 * Math.PI);").unwrap();
    writeln!(file, "                    ctx.fill();").unwrap();
    writeln!(file, "                }} else {{").unwrap();
    writeln!(file, "                    ctx.fillStyle = '#555';").unwrap();
    writeln!(file, "                    ctx.beginPath();").unwrap();
    writeln!(file, "                    ctx.arc(x, y, 3, 0, 2 * Math.PI);").unwrap();
    writeln!(file, "                    ctx.fill();").unwrap();
    writeln!(file, "                }}").unwrap();
    writeln!(file, "                ").unwrap();
    writeln!(file, "                ctx.fillStyle = '#ccc';").unwrap();
    writeln!(file, "                ctx.font = '10px Arial';").unwrap();
    writeln!(file, "                ctx.fillText(i.toString(), x + 6, y - 6);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Info en canvas").unwrap();
    writeln!(file, "            ctx.fillStyle = '#4CAF50';").unwrap();
    writeln!(file, "            ctx.font = 'bold 14px Arial';").unwrap();
    writeln!(file, "            ctx.fillText(`Paso: ${{currentStep + 1}} / ${{stepsData.length}}`, 10, 20);").unwrap();
    writeln!(file, "            ctx.fillText(`Nodos: ${{path.length}} / ${{nodes.length}}`, 10, 40);").unwrap();
    writeln!(file, "            ctx.fillText(`Distancia: ${{step.currentDist.toFixed(0)}}`, 10, 60);").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function updateInfo() {{").unwrap();
    writeln!(file, "            const step = stepsData[currentStep];").unwrap();
    writeln!(file, "            document.getElementById('stepNum').textContent = currentStep;").unwrap();
    writeln!(file, "            document.getElementById('candidate').textContent = step.candidate;").unwrap();
    writeln!(file, "            document.getElementById('position').textContent = step.position;").unwrap();
    writeln!(file, "            document.getElementById('angle').textContent = (step.angle * 180 / Math.PI).toFixed(2) + '°';").unwrap();
    writeln!(file, "            document.getElementById('angleScore').textContent = step.angleScore.toFixed(4);").unwrap();
    writeln!(file, "            document.getElementById('cost').textContent = step.cost.toFixed(2);").unwrap();
    writeln!(file, "            document.getElementById('costPenalty').textContent = step.costPenalty.toFixed(4);").unwrap();
    writeln!(file, "            document.getElementById('totalScore').textContent = step.totalScore.toFixed(4);").unwrap();
    writeln!(file, "            document.getElementById('currentDist').textContent = step.currentDist.toFixed(0);").unwrap();
    writeln!(file, "            document.getElementById('stepSlider').value = currentStep;").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function goToStep(idx) {{").unwrap();
    writeln!(file, "            if (idx < 0 || idx >= stepsData.length) return;").unwrap();
    writeln!(file, "            currentStep = idx;").unwrap();
    writeln!(file, "            drawCurrentStep();").unwrap();
    writeln!(file, "            updateInfo();").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function nextStep() {{ goToStep(currentStep + 1); }}").unwrap();
    writeln!(file, "        function prevStep() {{ goToStep(currentStep - 1); }}").unwrap();

    writeln!(file, "        function toggleAnimation() {{").unwrap();
    writeln!(file, "            const btn = document.getElementById('animBtn');").unwrap();
    writeln!(file, "            if (animInterval) {{").unwrap();
    writeln!(file, "                clearInterval(animInterval);").unwrap();
    writeln!(file, "                animInterval = null;").unwrap();
    writeln!(file, "                btn.textContent = '▶ Animar';").unwrap();
    writeln!(file, "            }} else {{").unwrap();
    writeln!(file, "                btn.textContent = '⏸ Pausar';").unwrap();
    writeln!(file, "                animInterval = setInterval(() => {{").unwrap();
    writeln!(file, "                    if (currentStep >= stepsData.length - 1) {{").unwrap();
    writeln!(file, "                        clearInterval(animInterval);").unwrap();
    writeln!(file, "                        animInterval = null;").unwrap();
    writeln!(file, "                        btn.textContent = '▶ Animar';").unwrap();
    writeln!(file, "                    }} else {{").unwrap();
    writeln!(file, "                        nextStep();").unwrap();
    writeln!(file, "                    }}").unwrap();
    writeln!(file, "                }}, 300);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function toggleLkhTour() {{").unwrap();
    writeln!(file, "            showLkhTour = !showLkhTour;").unwrap();
    writeln!(file, "            const btn = document.getElementById('lkhBtn');").unwrap();
    writeln!(file, "            btn.textContent = showLkhTour ? ' Ocultar LKH' : ' Mostrar LKH';").unwrap();
    writeln!(file, "            drawCurrentStep();").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function toggleBubbleView() {{").unwrap();
    writeln!(file, "            showBubbleView = !showBubbleView;").unwrap();
    writeln!(file, "            const btn = document.getElementById('bubbleBtn');").unwrap();
    writeln!(file, "            btn.textContent = showBubbleView ? ' Ocultar Bubble Removal' : ' Ver Bubble Removal';").unwrap();
    writeln!(file, "            if (showBubbleView && optStepsData.length > 0) {{").unwrap();
    writeln!(file, "                drawBubbleStep();").unwrap();
    writeln!(file, "            }} else {{").unwrap();
    writeln!(file, "                drawCurrentStep();").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        function drawBubbleStep() {{").unwrap();
    writeln!(file, "            if (optStepsData.length === 0) {{").unwrap();
    writeln!(file, "                drawCurrentStep();").unwrap();
    writeln!(file, "                return;").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ctx.clearRect(0, 0, canvas.width, canvas.height);").unwrap();
    writeln!(file, "            const {{ scale, offsetX, offsetY }} = getScale();").unwrap();
    writeln!(file, "            const step = optStepsData[currentOptStep];").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar tour antes de bubble removal (morado punteado)").unwrap();
    writeln!(file, "            if (step.pathBefore.length > 1) {{").unwrap();
    writeln!(file, "                ctx.strokeStyle = 'rgba(156, 39, 176, 0.5)';").unwrap();
    writeln!(file, "                ctx.lineWidth = 3;").unwrap();
    writeln!(file, "                ctx.setLineDash([8, 4]);").unwrap();
    writeln!(file, "                ctx.beginPath();").unwrap();
    writeln!(file, "                for (let i = 0; i < step.pathBefore.length; i++) {{").unwrap();
    writeln!(file, "                    const n1 = nodes[step.pathBefore[i]];").unwrap();
    writeln!(file, "                    const n2 = nodes[step.pathBefore[(i + 1) % step.pathBefore.length]];").unwrap();
    writeln!(file, "                    ctx.moveTo(n1.x * scale + offsetX, n1.y * scale + offsetY);").unwrap();
    writeln!(file, "                    ctx.lineTo(n2.x * scale + offsetX, n2.y * scale + offsetY);").unwrap();
    writeln!(file, "                }}").unwrap();
    writeln!(file, "                ctx.stroke();").unwrap();
    writeln!(file, "                ctx.setLineDash([]);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar tour después de bubble removal (verde)").unwrap();
    writeln!(file, "            if (step.pathAfter.length > 1) {{").unwrap();
    writeln!(file, "                ctx.strokeStyle = '#4CAF50';").unwrap();
    writeln!(file, "                ctx.lineWidth = 2;").unwrap();
    writeln!(file, "                ctx.beginPath();").unwrap();
    writeln!(file, "                for (let i = 0; i < step.pathAfter.length; i++) {{").unwrap();
    writeln!(file, "                    const n1 = nodes[step.pathAfter[i]];").unwrap();
    writeln!(file, "                    const n2 = nodes[step.pathAfter[(i + 1) % step.pathAfter.length]];").unwrap();
    writeln!(file, "                    ctx.moveTo(n1.x * scale + offsetX, n1.y * scale + offsetY);").unwrap();
    writeln!(file, "                    ctx.lineTo(n2.x * scale + offsetX, n2.y * scale + offsetY);").unwrap();
    writeln!(file, "                }}").unwrap();
    writeln!(file, "                ctx.stroke();").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Dibujar nodos").unwrap();
    writeln!(file, "            for (let i = 0; i < nodes.length; i++) {{").unwrap();
    writeln!(file, "                const node = nodes[i];").unwrap();
    writeln!(file, "                const x = node.x * scale + offsetX;").unwrap();
    writeln!(file, "                const y = node.y * scale + offsetY;").unwrap();
    writeln!(file, "                ctx.fillStyle = '#2196F3';").unwrap();
    writeln!(file, "                ctx.beginPath();").unwrap();
    writeln!(file, "                ctx.arc(x, y, 5, 0, 2 * Math.PI);").unwrap();
    writeln!(file, "                ctx.fill();").unwrap();
    writeln!(file, "                ctx.fillStyle = '#ccc';").unwrap();
    writeln!(file, "                ctx.font = '10px Arial';").unwrap();
    writeln!(file, "                ctx.fillText(i.toString(), x + 6, y - 6);").unwrap();
    writeln!(file, "            }}").unwrap();
    writeln!(file, "            ").unwrap();
    writeln!(file, "            // Info en canvas").unwrap();
    writeln!(file, "            ctx.fillStyle = '#9C27B0';").unwrap();
    writeln!(file, "            ctx.font = 'bold 14px Arial';").unwrap();
    writeln!(file, "            ctx.fillText(`Bubble Removal: ${{step.name}}`, 10, 20);").unwrap();
    writeln!(file, "            ctx.fillText(`Antes: ${{step.distanceBefore.toFixed(0)}} → Después: ${{step.distanceAfter.toFixed(0)}}`, 10, 40);").unwrap();
    writeln!(file, "            const improvement = ((step.distanceBefore - step.distanceAfter) / step.distanceBefore * 100);").unwrap();
    writeln!(file, "            ctx.fillText(`Mejora: ${{improvement.toFixed(2)}}%`, 10, 60);").unwrap();
    writeln!(file, "        }}").unwrap();

    writeln!(file, "        window.addEventListener('resize', resizeCanvas);").unwrap();
    writeln!(file, "        resizeCanvas();").unwrap();
    writeln!(file, "        updateInfo();").unwrap();
    writeln!(file, "    </script>").unwrap();
    writeln!(file, "</body>").unwrap();
    writeln!(file, "</html>").unwrap();
}
