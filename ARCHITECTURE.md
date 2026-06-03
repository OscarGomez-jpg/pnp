# Arquitectura del Visualizador Interactivo TSP

## 📋 Estructura General

El proyecto ha sido **refactorizado en módulos especializados** para facilitar mantenimiento, testing y expansión:

```
src/
├── lib.rs              # Punto de entrada de la librería (expone módulos)
├── main.rs             # Orquestación UI + punto de entrada del programa
├── core.rs             # Tipos fundamentales y utilidades matemáticas
├── scenario.rs         # Generación de escenarios de prueba
├── ui.rs               # Renderizado y controles de interfaz
└── strategies/
    ├── mod.rs          # DSL declarativo + registry de estrategias
    ├── nearest_neighbor.rs    # Algoritmo Greedy clásico
    └── triangle_insertion.rs  # Triángulo + Inserción Inteligente

tests/
└── integration_tests.rs  # Suite completa de tests end-to-end
```

---

## 🎯 Módulos Explicados

### 1. **`core.rs`** - Tipos y Utilidades
Define las estructuras fundamentales del problema:

```rust
struct Node { pos: Vec2 }  // Un punto en el espacio 2D
```

**Funciones principales:**
- `path_distance()` - Calcula distancia total de un tour cerrado
- `insertion_cost()` - Costo de insertar un nodo en una arista
- Tests unitarios para validar cálculos matemáticos

### 2. **`scenario.rs`** - Generador de Casos de Prueba
Proporciona escenarios predefinidos para comparación algorítmica:

- **Manual**: Agregar nodos manualmente con clicks
- **CirculoPerfecto**: 12 puntos en círculo (óptimo evidente)
- **RejillaCuadrada**: 4×4 puntos en grid (óptimo simétrico)

```rust
pub fn generate_scenario(scenario: TestScenario, width: f32, height: f32) -> Vec<Node>
```

### 3. **`strategies/mod.rs`** - Sistema Declarativo (DSL)

Este es el **corazón del sistema de extensibilidad**.

#### Trait `Strategy`
```rust
pub trait Strategy: Send {
    fn execute_step(&mut self, path: &mut Vec<usize>, nodes: &[Node]) -> bool;
    fn name(&self) -> &str;
    fn reset(&mut self);
}
```

#### Registry Declarativo
```rust
pub struct StrategyRegistry {
    strategies: HashMap<String, StrategyDescriptor>,
}

impl StrategyRegistry {
    pub fn register(&mut self, descriptor: StrategyDescriptor) { }
    pub fn get_strategy(&self, id: &str) -> Option<Box<dyn Strategy>> { }
}
```

**Ventajas:**
✅ Agregar nuevas estrategias sin modificar `main.rs`  
✅ Comparación automática entre múltiples algoritmos  
✅ Cada estrategia es independiente y testeable  

### 4. **`strategies/nearest_neighbor.rs`** - Greedy Clásico
Algoritmo simple de referencia:
1. Comienza en nodo 0
2. En cada paso, selecciona el vecino más cercano no visitado
3. Repeats hasta visitar todos los nodos

### 5. **`strategies/triangle_insertion.rs`** - Tu Algoritmo Inteligente
Tu estrategia optimizada con look-ahead:
1. Inicia con triángulo obligado [0, 1, 2]
2. Query: Evalúa los 2 nodos más cercanos (P₁, P₂)
3. Calcula costo de inserción para cada uno
4. Inserta el que menos impacto tenga en el perímetro

### 6. **`ui.rs`** - Interfaz y Renderizado
Encapsula toda la lógica de UI/UX:
- `render_hud()` - Interfaz de estado y controles
- `render_graph()` - Renderizado de nodos y camino
- `handle_*_input()` - Funciones de input

### 7. **`main.rs`** - Orquestador Principal
Integra todos los módulos:
1. Carga registry de estrategias
2. Loop principal: input → algoritmo → render
3. Manejo de estados (Edit/Running/Finished)

---

## 🚀 Cómo Agregar una Nueva Estrategia

### Paso 1: Crear archivo `src/strategies/my_strategy.rs`

```rust
use super::Strategy;
use crate::core::Node;

pub struct MyStrategy {
    // Tu estado interno aquí
}

impl MyStrategy {
    pub fn new() -> Self {
        Self { }
    }
}

impl Strategy for MyStrategy {
    fn execute_step(&mut self, path: &mut Vec<usize>, nodes: &[Node]) -> bool {
        // Lógica del paso
        false  // true = terminado
    }

    fn name(&self) -> &str {
        "Mi Estrategia Genial"
    }

    fn reset(&mut self) {
        // Reiniciar estado interno
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_strategy() {
        // Tus tests
    }
}
```

### Paso 2: Agregar al módulo `src/strategies/mod.rs`

```rust
pub mod my_strategy;  // Agregar esta línea

// En create_registry():
registry.register(StrategyDescriptor {
    id: "my_strategy".to_string(),
    name: "Mi Estrategia Genial".to_string(),
    factory: || Box::new(my_strategy::MyStrategy::new()),
});
```

### Paso 3: Tests Automáticos
¡Listo! Ya puedes:
- Ejecutar con `[E]` para cambiar de estrategia
- Comparar visualmente contra otras estrategias
- Agregar tests de integración en `tests/integration_tests.rs`

---

## 🧪 Suite de Tests

### Unitarios (20 tests)
```bash
cargo test --lib
```

- **core**: Operaciones matemáticas (distancia, costo de inserción)
- **scenario**: Generación de escenarios
- **strategies**: Cada estrategia se prueba independientemente
- **ui**: Configuración y estado de UI

### Integración (9 tests)
```bash
cargo test --test integration_tests
```

- **Completitud**: Ambas estrategias completan todos los escenarios
- **Consistencia**: Distancias calculadas correctamente
- **Correctitud**: Ciclos válidos sin nodos duplicados
- **Registry**: Sistema de registro funciona

### Todos los Tests
```bash
cargo test
```
Resultado: **29 tests ✅ sin errores**

---

## 📊 Comparación de Estrategias

| Aspecto | Nearest Neighbor | Triangle Insertion |
|---------|----------------|--------------------|
| **Complejidad** | O(n²) | O(n²) |
| **Inicialización** | 1 nodo | 3 nodos (triángulo) |
| **Look-ahead** | No | Sí (2 candidatos) |
| **Óptimo para** | Baselines simples | Puntos desordenados |
| **Tests** | ✅ 3 unitarios | ✅ 4 unitarios |

---

## 🎮 Controles de Usuario

| Tecla | Acción |
|-------|--------|
| **[E]** | Cambiar estrategia (cicla entre todas las registradas) |
| **[T]** | Cambiar escenario de prueba |
| **[ESPACIO]** | Ejecutar/pausar algoritmo |
| **[C]** | Resetear a modo manual vacío |
| **[CLICK]** | Agregar nodos manualmente (modo Manual) |

---

## 🏗️ Flujo de Ejecución

```
main() 
  ├─ create_registry()      # Cargar todas las estrategias
  │
  ├─ loop:
  │   ├─ handle_input()     # Detectar cambios de estrategia/escenario
  │   ├─ execute_step()     # Un paso del algoritmo actual
  │   ├─ render_graph()     # Dibujar nodos y camino
  │   └─ render_hud()       # Mostrar información de estado
  │
  └─ next_frame()           # Animar con delay configurable
```

---

## 📈 Próximas Mejoras Sugeridas

1. **Nuevas estrategias**:
   - 2-Opt (mejora local)
   - Christofides (garantía de aproximación)
   - Algoritmo genético
   - Simulated Annealing

2. **Métricas**:
   - Comparativa de distancias en tiempo real
   - Historial de evolución del tour
   - Estadísticas: tiempo, iteraciones, ratio vs óptimo conocido

3. **UI mejorada**:
   - Panel lateral con resultados
   - Exportar soluciones a CSV
   - Reproducción paso-a-paso

4. **Optimizaciones**:
   - Soporte para +1000 nodos
   - Cálculos paralelos con rayon
   - Caché de distancias

---

## 🔧 Comandos Útiles

```bash
# Compilar
cargo build

# Ejecutar
cargo run

# Tests
cargo test                  # Todos los tests
cargo test --lib          # Solo unitarios
cargo test --test integration_tests  # Solo integración
cargo test --release      # Modo optimizado

# Coverage
cargo tarpaulin --out Html  # Reporte de cobertura

# Lint
cargo clippy
```

---

**Última actualización**: 3 de junio de 2026  
**Versión**: 0.1.0  
**Estado**: ✅ Refactorizado y testeable
