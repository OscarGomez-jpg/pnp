use std::fs;
use std::io::Write;
use std::process::Command;
use traveler::core::{path_distance, Node};
use traveler::strategies::triangle_insertion_v8_6::{TriangleInsertionV86, V86Params};
use traveler::strategies::triangle_insertion_v9::{TriangleInsertionV9, V9Params};
use traveler::strategies::Strategy;
use traveler::tsplib::TspInstance;

struct TourResult {
    label: String,
    path: Vec<usize>,
    distance: f32,
}

struct InstanceResult {
    name: String,
    nodes: Vec<Node>,
    tours: Vec<TourResult>,
}

fn run_v86(nodes: &[Node]) -> TourResult {
    let mut s = TriangleInsertionV86::with_params(V86Params {
        k_neighbors: 8,
        w_angle: 0.25,
        w_cost: 0.25,
    });
    let mut path = vec![];
    let mut steps = 0;
    loop {
        if s.execute_step(&mut path, nodes) || steps > nodes.len() + 500 {
            break;
        }
        steps += 1;
    }
    let dist = path_distance(&path, nodes);
    TourResult { label: "V8.6".into(), path, distance: dist }
}

fn run_v9(nodes: &[Node]) -> TourResult {
    let mut s = TriangleInsertionV9::with_params(V9Params {
        k_neighbors: 8,
        w_angle: 0.40,
        w_cost: 0.30,
        w_density: 0.20,
    });
    let mut path = vec![];
    let mut steps = 0;
    loop {
        if s.execute_step(&mut path, nodes) || steps > nodes.len() + 500 {
            break;
        }
        steps += 1;
    }
    let dist = path_distance(&path, nodes);
    TourResult { label: "V9".into(), path, distance: dist }
}

fn run_lkh(instance: &TspInstance) -> Option<TourResult> {
    let dir = "/tmp/lkh_compare";
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
    writeln!(pf, "TRACE_LEVEL = 0").ok()?;
    writeln!(pf, "RUNS = 1").ok()?;
    drop(pf);

    let out = Command::new("./LKH-3.0.14/LKH")
        .arg(&par)
        .output()
        .ok()?;
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
        Some(TourResult { label: "LKH".into(), path, distance: dist })
    } else {
        None
    }
}

fn write_html(results: &[InstanceResult], optima: &[f64], out: &str) {
    let mut h = String::new();
    h.push_str(
        r#"<!DOCTYPE html><html><head><meta charset="UTF-8"><title>V8.6 vs V8.8 vs LKH</title>
<style>*{margin:0;padding:0;box-sizing:border-box}
body{background:#0a0a0a;color:#ccc;font-family:monospace;padding:16px}
h1{text-align:center;font-size:16px;color:#fff;margin-bottom:4px}
.sub{text-align:center;font-size:11px;color:#888;margin-bottom:16px}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:16px;max-width:1400px;margin:auto}
.p{background:#111;border:1px solid #333;border-radius:6px;overflow:hidden}
.pt{background:#1a1a1a;padding:6px 10px;font-size:12px;border-bottom:1px solid #333;color:#fff}
canvas{display:block;width:100%;height:400px}
.info{padding:6px 10px;font-size:10px;display:flex;gap:16px;flex-wrap:wrap;border-top:1px solid #333}
.s{display:flex;gap:3px;align-items:center}
.s .lb{color:#777}.s .v{font-weight:bold}
.lkh{color:#FF9800}.v86{color:#4CAF50}.v9{color:#2196F3}
</style></head><body>
<h1>V8.6 vs V9 vs LKH — Recursive Edge Insertion</h1>
<div class="sub">berlin52 &middot; eil51 &middot; st70 &middot; kroA100</div>
<div class="grid">"#,
    );

    for (idx, r) in results.iter().enumerate() {
        let opt = optima[idx];
        h.push_str(&format!(
            "<div class=\"p\"><div class=\"pt\">{} <span style=\"color:#888\">({} nodos)</span></div><canvas id=\"c{}\"></canvas><div class=\"info\">",
            r.name,
            r.nodes.len(),
            idx
        ));
        for t in &r.tours {
            let err = (t.distance - opt as f32) / opt as f32 * 100.0;
            let cls = if t.label == "LKH" {
                "lkh"
            } else if t.label == "V8.6" {
                "v86"
            } else {
                "v9"
            };
            h.push_str(&format!(
                "<div class=\"s\"><span class=\"lb\">{}:</span><span class=\"v {}\">{:.2}</span><span>({:.2}%)</span></div>",
                t.label, cls, t.distance, err
            ));
        }
        h.push_str(&format!(
            "<div class=\"s\"><span class=\"lb\">Optimo:</span><span class=\"v\">{:.2}</span></div></div></div>",
            opt
        ));
    }

    h.push_str("</div><script>\n");

    for (idx, r) in results.iter().enumerate() {
        h.push_str(&format!("const N{}=", idx));
        h.push('[');
        for n in &r.nodes {
            h.push_str(&format!("{{x:{},y:{}}},", n.pos.x, n.pos.y));
        }
        h.push_str("];\n");

        for t in &r.tours {
            let var = format!("{}_{}", t.label.replace('.', ""), idx);
            h.push_str(&format!("const {}=", var));
            h.push('[');
            for &p in &t.path {
                h.push_str(&format!("{},", p));
            }
            h.push_str("];\n");
        }

        h.push_str(&format!(
            r#"(function(){{
const c=document.getElementById('c{}');
function d(){{
const r=c.getBoundingClientRect();
const dpr=window.devicePixelRatio||1;
c.width=r.width*dpr;c.height=r.height*dpr;
const ctx=c.getContext('2d');
ctx.scale(dpr,dpr);
const W=r.width,H=r.height,p=40;
let mnX=1e9,mxX=-1e9,mnY=1e9,mxY=-1e9;
for(const n of N{}){{if(n.x<mnX)mnX=n.x;if(n.x>mxX)mxX=n.x;if(n.y<mnY)mnY=n.y;if(n.y>mxY)mxY=n.y;}}
const sc=Math.min((W-2*p)/(mxX-mnX||1),(H-2*p)/(mxY-mnY||1));
const ox=(W-(mxX-mnX)*sc)/2-mnX*sc;
const oy=(H-(mxY-mnY)*sc)/2-mnY*sc;
const px=n=>N{}[n].x*sc+ox;
const py=n=>N{}[n].y*sc+oy;
ctx.fillStyle='#0a0a0a';ctx.fillRect(0,0,W,H);
"#,
            idx, idx, idx, idx,
        ));

        for t in &r.tours {
            let var = format!("{}_{}", t.label.replace('.', ""), idx);
            let (color, lw, dash) = match t.label.as_str() {
                "LKH" => ("rgba(255,152,0,0.5)", "1.5", "ctx.setLineDash([6,4]);"),
                "V8.6" => ("rgba(76,175,80,0.7)", "2.0", "ctx.setLineDash([]);"),
                _ => ("rgba(33,150,243,0.7)", "2.0", "ctx.setLineDash([]);"),
            };
            h.push_str(&format!(
                "ctx.strokeStyle='{color}';ctx.lineWidth={lw};{dash}ctx.beginPath();ctx.moveTo(px({var}[0]),py({var}[0]));for(let i=1;i<{var}.length;i++)ctx.lineTo(px({var}[i]),py({var}[i]));ctx.closePath();ctx.stroke();\n"
            ));
        }

        h.push_str(&format!(
            "ctx.setLineDash([]);for(let i=0;i<N{idx}.length;i++){{ctx.fillStyle='#555';ctx.beginPath();ctx.arc(px(i),py(i),2.5,0,Math.PI*2);ctx.fill();}}}}d();window.addEventListener('resize',d);}})();\n"
        ));
    }

    h.push_str("</script></body></html>");
    fs::write(out, h).expect("write failed");
}

fn main() {
    println!("Comparacion V8.6 vs V9 vs LKH\n");

    let configs = [
        ("berlin52", "assets/berlin52.tsp", 7542.0),
        ("eil51", "assets/eil51.tsp", 426.0),
        ("st70", "assets/st70.tsp", 675.0),
        ("kroA100", "assets/kroA100.tsp", 21282.0),
    ];

    let mut results: Vec<InstanceResult> = vec![];
    let mut optima: Vec<f64> = vec![];

    for (name, path, opt) in &configs {
        let inst = match TspInstance::from_file(path) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("Error {}: {}", name, e);
                continue;
            }
        };

        let v86 = run_v86(&inst.nodes);
        let v9 = run_v9(&inst.nodes);
        let lkh = run_lkh(&inst);

        let mut tours = vec![v9, v86];
        if let Some(l) = lkh {
            tours.insert(0, l);
        }

        println!("{} ({} nodos):", name, inst.dimension);
        for t in &tours {
            let err = (t.distance - *opt as f32) / *opt as f32 * 100.0;
            println!("  {}: {:.2} ({:.2}%)", t.label, t.distance, err);
        }

        results.push(InstanceResult {
            name: name.to_string(),
            nodes: inst.nodes,
            tours,
        });
        optima.push(*opt);
    }

    let out = "comparison_v86_v9.html";
    write_html(&results, &optima, out);
    println!("\n✓ Abrir: {}", out);
}
