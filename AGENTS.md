# AGENTS.md

## Commands

```bash
cargo run                          # Interactive macroquad visualizer
cargo run --release                # Optimized build
cargo build                        # Compile all binaries
cargo test                         # Run all tests
cargo test --lib                   # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo run --bin benchmark          # TSPLIB benchmark (all strategies)
cargo run --bin visualize_v86      # HTML visualization generator
cargo run --bin train_v86          # V8.6 parameter grid search
cargo run --bin train_v9           # V9 parameter grid search
cargo run --bin validate_v86       # Overfitting validation
```

## Architecture

- **Entry point**: `src/main.rs` → macroquad interactive visualizer
- **Library**: `src/lib.rs` exposes all modules
- **Strategies**: `src/strategies/` contains 16 TSP algorithm implementations (V1-V8.9, V9, V9Hybrid)
- **Core types**: `src/core.rs` defines `Node` (uses `macroquad::prelude::Vec2`), `path_distance`, `insertion_cost`
- **TSPLIB**: `src/tsplib.rs` parses `.tsp` files from `assets/`
- **UI**: `src/ui.rs` handles macroquad rendering, HUD is 140px tall
- **Binaries**: `src/bin/*.rs` contains benchmark, training, validation, visualization tools

## Strategy Pattern

All algorithms implement `Strategy` trait:
```rust
pub trait Strategy: Send {
    fn execute_step(&mut self, path: &mut Vec<usize>, nodes: &[Node]) -> bool;
    fn name(&self) -> &str;
    fn reset(&mut self);
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
```

New strategies must be registered in `create_registry()` at `src/strategies/mod.rs:84`.

## Current Focus (V9)

`src/strategies/triangle_insertion_v9.rs` is the active development target:
- Convex hull initialization
- K-D tree for neighbor search per edge midpoint
- **Recursive Edge Insertion (REI)**: evaluates every tour edge and picks the globally best insertion
- Outside-in insertion with angle + cost scoring
- Optional local-density term (`w_density`) to favor subdividing edges in dense unvisited regions
- Post-optimization: 2-opt → or-opt → node reinsertion → bubble removal (max 10 iterations)
- Parameters: `V9Params { k_neighbors, w_angle, w_cost, w_density }`

### Recursive Edge Insertion

Instead of selecting a candidate point and then finding its best position, V9 treats each tour edge as a potential subdivision. For every edge `(i,j)` it searches unvisited nodes near the edge midpoint, computes an insertion score, and inserts the best candidate globally. This makes long or poorly-shaped edges more likely to be refined first.

Default calibrated parameters:
- `k_neighbors = 8`
- `w_angle = 0.40`
- `w_cost = 0.30`
- `w_density = 0.00`

## V9 Hybrid

`src/strategies/triangle_insertion_v9_hybrid.rs` selects between V9 and V8.9 based on instance geometry:

- Detects separated clusters using a simple DBSCAN-like component analysis (eps = 25th percentile of pairwise distances).
- Also uses `nearest_ratio` (mean nearest-neighbor distance / mean pairwise distance) and `dispersion` (max/min distance ratio).
- Chooses V9 when clusters are well separated, or when nearest_ratio is very low and dispersion is moderate.
- Falls back to V8.9 for uniform / single-scale instances.

This hybrid currently achieves the best average error on the TSPLIB benchmark (~0.70%).

## Previous Focus (V8.6)

`src/strategies/triangle_insertion_v8_6.rs`:
- Convex hull initialization
- K-D tree for neighbor search (k=4 calibrated)
- **"Seagull" strategy**: Only considers uncontested candidates (nodes that are the closest to some tour node without competition)
- Outside-in insertion with angle + cost scoring
- Post-optimization: 2-opt → or-opt → node reinsertion → bubble removal (max 10 iterations)
- Parameters: `V86Params { k_neighbors, w_angle, w_cost }`

### Seagull Strategy (Estrategia de Gaviotas)

The algorithm mimics seagull foraging behavior: instead of evaluating all unvisited nodes, it only considers nodes that are "uncontested" - i.e. the closest unvisited node to some tour node. This ensures local coherence in decision-making and prevents the algorithm from making globally optimal but locally inconsistent choices.

Implementation:
1. Build K-D tree with ALL nodes (visited and unvisited)
2. For each tour node, find the closest unvisited node
3. Only evaluate these "uncontested" candidates for insertion
4. Fallback: if fewer than 3 candidates found, add candidates near path center

## External Dependencies

- **LKH-3.0.14**: Compiled binary at `./LKH-3.0.14/LKH` for reference tours
- **TSPLIB instances**: `assets/berlin52.tsp`, `assets/eil51.tsp`, etc.
- **Optima**: `assets/optima.txt` contains known optimal distances

## Conventions

- `rand` crate: use `::rand::rng()` with `RngExt` trait (not `thread_rng()`)
- Random point generation: pass `y_offset` parameter to respect HUD (140px)
- Node positions: `macroquad::prelude::Vec2`, not `glam::Vec2`
- Strategy IDs: snake_case (e.g., `triangle_insertion_v8_6`)
- Test naming: `test_<strategy>_<behavior>`

## Interactive Controls

| Key | Action |
|-----|--------|
| `E` | Cycle strategies |
| `T` | Cycle test scenarios |
| `Space` | Run/pause/reset |
| `R` | Generate N random points |
| `G` | Generate N cluster points |
| `C` | Reset to manual mode |
| `X` | Export solution to TXT |
