# 🚀 Traveler - Visualizador Interactivo del TSP

Un entorno de pruebas visual para algoritmos del **Problema del Viajero (TSP)** en Rust con rendering interactivo en tiempo real.

## ✨ Características

- **Algoritmos implementados**:
  - ✅ Vecino Más Cercano (Greedy clásico)
  - ✅ Triángulo + Inserción Inteligente (Tu algoritmo con look-ahead)

- **Escenarios de prueba predefinidos**:
  - 🔵 **Círculo Perfecto** (12 puntos, óptimo obvio)
  - 🔲 **Rejilla Cuadrada** (4×4, óptimo simétrico)
  - ✏️ **Manual** (agregar nodos con clicks)

- **Testing completo**:
  - ✅ 20 tests unitarios
  - ✅ 9 tests de integración
  - ✅ Cobertura total de módulos

- **Arquitectura modular**:
  - Fácil agregar nuevas estrategias
  - DSL declarativo para algoritmos
  - Cada módulo es independiente y testeable

---

## 🎮 Controles

| Tecla | Acción |
|-------|--------|
| **[E]** | Cambiar estrategia de algoritmo |
| **[T]** | Cambiar escenario de prueba |
| **[ESPACIO]** | Ejecutar / Pausar / Reiniciar |
| **[C]** | Resetear a modo manual |
| **[CLICK]** | Agregar nodos (modo Manual) |

---

## 📦 Instalación y Uso

### Requisitos
- Rust 1.70+
- Cargo

### Clonar y ejecutar

```bash
cd /home/osgomez/Code/crust-projects/traveler

# Compilar y ejecutar
cargo run

# O directamente
cargo run --release  # Versión optimizada
```

### Ejecutar tests

```bash
# Todos los tests
cargo test

# Solo unitarios
cargo test --lib

# Solo integración
cargo test --test integration_tests

# Con output
cargo test -- --nocapture
```

---

## 📁 Estructura del Proyecto

```
src/
├── lib.rs                    # Librería principal
├── main.rs                   # Programa interactivo
├── core.rs                   # Tipos y utilidades matemáticas
├── scenario.rs               # Generación de escenarios
├── ui.rs                     # Interfaz y renderizado
└── strategies/
    ├── mod.rs                # DSL + registry (el corazón extensible)
    ├── nearest_neighbor.rs   # Algoritmo Greedy
    └── triangle_insertion.rs # Tu algoritmo inteligente

tests/
└── integration_tests.rs      # Suite de tests end-to-end

ARCHITECTURE.md               # Documentación técnica detallada
```

---

## 🛠️ Agregar una Nueva Estrategia

Supongamos que quieres implementar **2-Opt (Local Search)**:

### 1. Crear `src/strategies/two_opt.rs`

```rust
use super::Strategy;
use crate::core::Node;

pub struct TwoOpt {
    improved: bool,
}

impl TwoOpt {
    pub fn new() -> Self {
        Self { improved: true }
    }
}

impl Strategy for TwoOpt {
    fn execute_step(&mut self, path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        // Implementar un paso del algoritmo
        // Retornar true si terminó
        false
    }

    fn name(&self) -> &str {
        "2-Opt (Local Search)"
    }

    fn reset(&mut self) {
        self.improved = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_opt_improves() {
        // Tus tests
    }
}
```

### 2. Registrar en `src/strategies/mod.rs`

```rust
pub mod two_opt;  // Agregar esta línea

// En la función create_registry():
registry.register(StrategyDescriptor {
    id: "two_opt".to_string(),
    name: "2-Opt (Local Search)".to_string(),
    factory: || Box::new(two_opt::TwoOpt::new()),
});
```

### 3. ¡Listo! 
- Presiona `[E]` para cambiar a tu estrategia
- Ejecuta `cargo test` para verificar que todo compila
- Tu algoritmo aparece automáticamente en la comparativa

---

## 📊 Ejemplo de Salida

```
=== CONTROLES DE EXPERIMENTACIÓN ===
 [E] Estrategia actual: Triángulo + Inserción Inteligente (Tu Query)
 [T] Escenario de Test: Test: Círculo Perfecto (Óptimo obvio)

ESTADO: EJECUTANDO PASO A PASO...
Ciudades: 12 | Distancia: 2275.43 px

[C] Resetear a modo Manual vacio
```

---

## 🧪 Test Coverage

### Unitarios (20)
```
✓ core::test_node_creation
✓ core::test_distance_to
✓ core::test_path_distance_*
✓ core::test_insertion_cost
✓ scenario::test_generate_*
✓ scenario::test_scenario_names
✓ strategies::nearest_neighbor::test_*
✓ strategies::triangle_insertion::test_*
✓ strategies::test_registry_*
✓ ui::test_*
```

### Integración (9)
```
✓ test_triangle_insertion_solves_simple_problem
✓ test_nearest_neighbor_solves_simple_problem
✓ test_both_strategies_complete_circle_scenario
✓ test_both_strategies_complete_grid_scenario
✓ test_path_distance_is_consistent
✓ test_strategy_reset_works
✓ test_multiple_strategies_in_registry
✓ test_strategy_names_are_descriptive
✓ test_node_positioning_in_scenarios
```

---

## 📚 Documentación Detallada

Para información técnica completa sobre la arquitectura, módulos y cómo extender el proyecto:

→ Consulta [ARCHITECTURE.md](ARCHITECTURE.md)

---

## 🎯 Roadmap

- [ ] Agregar algoritmo 2-Opt
- [ ] Implementar Christofides
- [ ] Algoritmo genético
- [ ] Panel de métricas en tiempo real
- [ ] Exportar soluciones a archivo
- [ ] Soporte para 1000+ nodos

---

## 📝 Notas

- El algoritmo de **Triángulo + Inserción** es el más eficiente para los escenarios de prueba
- El **Círculo Perfecto** tiene un óptimo evidente: seguir el perímetro
- Los tests incluyen validación de ciclos, distancias y completitud
- Cada estrategia es independiente y puede ser testeada aisladamente

---

## 📧 Soporte

Consulta `src/strategies/mod.rs` para entender el sistema de registry.  
Consulta `tests/integration_tests.rs` para ejemplos de testing.

---

**Versión**: 0.1.0  
**Estado**: ✅ Refactorizado, modular y completamente testeado  
**Fecha**: Junio 2026
