pub mod christofides;
/// Sistema declarativo de estrategias para TSP
///
/// Permite registrar y ejecutar estrategias de forma consistente,
/// facilitando la comparación de eficiencia entre algoritmos.
pub mod lin_kernighan;
pub mod nearest_neighbor;
pub mod triangle_insertion;
pub mod triangle_insertion_v2;
pub mod triangle_insertion_v3;
pub mod triangle_insertion_v4;
pub mod triangle_insertion_v5;
pub mod triangle_insertion_v6;
pub mod triangle_insertion_v7;
pub mod triangle_insertion_v8;
pub mod triangle_insertion_v8_5;
pub mod triangle_insertion_v8_6;
pub mod triangle_insertion_v8_7;
pub mod triangle_insertion_v8_9;
pub mod triangle_insertion_v9;
pub mod triangle_insertion_v9_hybrid;
pub mod v9_ils;

use crate::core::Node;
use std::any::Any;
use std::collections::HashMap;

/// Trait que define una estrategia de solución para TSP
pub trait Strategy: Send {
    /// Ejecuta un paso del algoritmo
    /// Devuelve true si la ejecución terminó, false si hay más pasos
    fn execute_step(&mut self, current_path: &mut Vec<usize>, nodes: &[Node]) -> bool;

    /// Nombre descriptivo de la estrategia
    fn name(&self) -> &str;

    /// Reinicia el estado interno de la estrategia
    fn reset(&mut self);

    /// Permite downcast para acceso a métodos específicos de cada estrategia
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Descriptor declarativo de una estrategia
pub struct StrategyDescriptor {
    pub id: String,
    pub name: String,
    pub factory: fn() -> Box<dyn Strategy>,
}

/// Registro global de estrategias
pub struct StrategyRegistry {
    strategies: HashMap<String, StrategyDescriptor>,
}

impl StrategyRegistry {
    pub fn new() -> Self {
        Self {
            strategies: HashMap::new(),
        }
    }

    /// Registra una nueva estrategia
    pub fn register(&mut self, descriptor: StrategyDescriptor) {
        self.strategies.insert(descriptor.id.clone(), descriptor);
    }

    /// Obtiene una estrategia por ID
    pub fn get_strategy(&self, id: &str) -> Option<Box<dyn Strategy>> {
        self.strategies.get(id).map(|desc| (desc.factory)())
    }

    /// Lista todos los IDs de estrategias disponibles
    pub fn list_ids(&self) -> Vec<&str> {
        self.strategies.keys().map(|s| s.as_str()).collect()
    }

    /// Lista todos los nombres de estrategias
    pub fn list_names(&self) -> Vec<&str> {
        self.strategies
            .values()
            .map(|desc| desc.name.as_str())
            .collect()
    }
}

/// Factory para crear el registry con todas las estrategias registradas
pub fn create_registry() -> StrategyRegistry {
    let mut registry = StrategyRegistry::new();

    // Registrar Triangle + Inserción Inteligente (V1)
    registry.register(StrategyDescriptor {
        id: "triangle_insertion".to_string(),
        name: "Triángulo + Inserción Inteligente (Tu Query)".to_string(),
        factory: || Box::new(triangle_insertion::TriangleInsertion::new()),
    });

    // Registrar Vecino Más Cercano
    registry.register(StrategyDescriptor {
        id: "nearest_neighbor".to_string(),
        name: "Vecino Más Cercano (Estándar)".to_string(),
        factory: || Box::new(nearest_neighbor::NearestNeighbor::new()),
    });

    // Registrar Triangle Insertion V2 mejorado
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v2".to_string(),
        name: "Triangle Insertion V2 (Smart + 2-Opt)".to_string(),
        factory: || Box::new(triangle_insertion_v2::TriangleInsertionV2::new()),
    });

    // Registrar Triangle Insertion V3 — Cheapest Insertion + Convex Hull
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v3".to_string(),
        name: "Triangle Insertion V3 (Convex Hull + Cheapest)".to_string(),
        factory: || Box::new(triangle_insertion_v3::TriangleInsertionV3::new()),
    });

    // Registrar Triangle Insertion V4 — Triangle Rotation
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v4".to_string(),
        name: "Triangle Insertion V4 (Look-Ahead Rotation)".to_string(),
        factory: || Box::new(triangle_insertion_v4::TriangleInsertionV4::new()),
    });

    // Registrar Triangle Insertion V5 — Exact Branch & Bound
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v5".to_string(),
        name: "Triangle Insertion V5 (Root Backtracking B&B)".to_string(),
        factory: || Box::new(triangle_insertion_v5::TriangleInsertionV5::new()),
    });

    // Registrar Triangle Insertion V6 — Smoothest Angle
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v6".to_string(),
        name: "Triangle Insertion V6 (Smoothest Angle)".to_string(),
        factory: || Box::new(triangle_insertion_v6::TriangleInsertionV6::new()),
    });

    // Registrar Triangle Insertion V7 — Geo-Accel + Ejection Chains + Simulated Annealing
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v7".to_string(),
        name: "Triangle Insertion V7 (Geo-Accel + Ejection Chains + SA)".to_string(),
        factory: || Box::new(triangle_insertion_v7::TriangleInsertionV7::new()),
    });

    // Registrar Triangle Insertion V8 — Outside-In Angle Optimization
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v8".to_string(),
        name: "Triangle Insertion V8 (Outside-In Angle Optimization)".to_string(),
        factory: || Box::new(triangle_insertion_v8::TriangleInsertionV8::new()),
    });

    // Registrar Triangle Insertion V8.5 — Adaptive LKH-H Integration
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v8_5".to_string(),
        name: "Triangle Insertion V8.5 (Adaptive LKH-H)".to_string(),
        factory: || Box::new(triangle_insertion_v8_5::TriangleInsertionV85::new()),
    });

    // Registrar Triangle Insertion V8.6 — Calibrated Outside-In
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v8_6".to_string(),
        name: "Triangle Insertion V8.6 (Calibrated)".to_string(),
        factory: || Box::new(triangle_insertion_v8_6::TriangleInsertionV86::new()),
    });

    // Registrar Triangle Insertion V8.7 — Onion Peeling
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v8_7".to_string(),
        name: "Triangle Insertion V8.7 (Onion Peeling)".to_string(),
        factory: || Box::new(triangle_insertion_v8_7::TriangleInsertionV87::new()),
    });

    // Registrar Triangle Insertion V8.9 — Pre-Seagull (Original)
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v8_9".to_string(),
        name: "Triangle Insertion V8.9 (Pre-Seagull)".to_string(),
        factory: || Box::new(triangle_insertion_v8_9::TriangleInsertionV89::new()),
    });

    // Registrar Triangle Insertion V9 — Recursive Edge Insertion
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v9".to_string(),
        name: "Triangle Insertion V9 (Recursive Edge Insertion)".to_string(),
        factory: || Box::new(triangle_insertion_v9::TriangleInsertionV9::new()),
    });

    // Registrar Triangle Insertion V9 Hybrid — Selector V9/V8.9
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v9_hybrid".to_string(),
        name: "Triangle Insertion V9 Hybrid (V9/V8.9 Selector)".to_string(),
        factory: || Box::new(triangle_insertion_v9_hybrid::TriangleInsertionV9Hybrid::new()),
    });

    // Registrar Triangle Insertion V9 + ILS
    registry.register(StrategyDescriptor {
        id: "triangle_insertion_v9_ils".to_string(),
        name: "Triangle Insertion V9 + ILS".to_string(),
        factory: || Box::new(v9_ils::TriangleInsertionV9Ils::new()),
    });

    // Registrar Lin-Kernighan (LK Simplificado)
    registry.register(StrategyDescriptor {
        id: "lin_kernighan".to_string(),
        name: "Lin-Kernighan (LK Simplificado)".to_string(),
        factory: || Box::new(lin_kernighan::LinKernighan::new()),
    });

    registry.register(StrategyDescriptor {
        id: "christofides".to_string(),
        name: "christofides heuristic (O(N3))".to_string(),
        factory: || Box::new(christofides::ChristofidesStrategy::new()),
    });

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = create_registry();
        let ids = registry.list_ids();
        assert!(ids.contains(&"triangle_insertion"));
        assert!(ids.contains(&"nearest_neighbor"));
    }

    #[test]
    fn test_get_strategy() {
        let registry = create_registry();
        let strategy = registry.get_strategy("nearest_neighbor");
        assert!(strategy.is_some());
    }

    #[test]
    fn test_list_names() {
        let registry = create_registry();
        let names = registry.list_names();
        assert_eq!(names.len(), 18);
    }
}
