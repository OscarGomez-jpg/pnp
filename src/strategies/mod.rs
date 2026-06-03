/// Sistema declarativo de estrategias para TSP
///
/// Permite registrar y ejecutar estrategias de forma consistente,
/// facilitando la comparación de eficiencia entre algoritmos.
pub mod nearest_neighbor;
pub mod triangle_insertion;
pub mod triangle_insertion_v2;

use crate::core::Node;
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
        assert_eq!(names.len(), 3);
    }
}
