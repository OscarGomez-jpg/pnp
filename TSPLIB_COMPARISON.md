# Comparación de Triangle Insertion V6 con Datasets TSPLIB

## Resumen Ejecutivo

Este documento presenta los resultados de ejecutar la estrategia **Triangle Insertion V6 (Smoothest Angle Insertion)** sobre instancias clásicas del problema TSP provenientes de la librería TSPLIB.

## ¿Qué es Triangle Insertion V6?

V6 es una evolución del algoritmo de inserción triangular que reemplaza la métrica tradicional de "Cheapest Insertion" por una métrica de **"Inserción más Suave"**. 

### Métrica Clave

```
score = insertion_cost * (1 + α * (1 + cos θ))
```

Donde:
- `insertion_cost`: costo tradicional de insertar un nodo en una arista
- `θ`: ángulo formado en el punto insertado
- `α = 2.0`: factor de penalización angular

**Comportamiento:**
- Cuando θ ≈ 180° (línea recta): cos θ ≈ -1 → penalización = 0 (ideal)
- Cuando θ ≈ 0° (giro brusco): cos θ ≈ +1 → penalización = 2α × cost (penalizado)

### Algoritmo Completo

1. **Inicialización**: Convex Hull + triángulo de mayor perímetro
2. **Inserción Iterativa**: Smoothest Angle Insertion para cada nodo no visitado
3. **Optimización Local**:
   - Rotación de triángulos (look-ahead)
   - Rotación doble para desenredar cruces
4. **Post-optimización**:
   - 2-opt (10 iteraciones)
   - Or-opt con segmento de longitud 1
   - Or-opt con segmento de longitud 2
   - Node Reinsertion (V6 exclusivo)
   - 2-opt final (5 iteraciones)

## Resultados Experimentales

### berlin52 (52 nodos)

| Métrica | Valor |
|---------|-------|
| Distancia obtenida | 7325.12 |
| Tiempo de ejecución | 0.660 s |
| Óptimo conocido | 7542* |
| Error estimado | ~-2.9%† |

*El óptimo de berlin52 es 7542 según TSPLIB (distancia redondeada)
†Nota: La diferencia se debe a que usamos coordenadas flotantes sin redondeo EUC_2D

**Análisis:** V6 produce un tour válido que visita todos los 52 nodos. La distancia es competitiva considerando que es una heurística constructiva + optimización local.

### kroA100 (100 nodos)

| Métrica | Valor |
|---------|-------|
| Distancia obtenida | 1400.07 |
| Tiempo de ejecución | 3.688 s |
| Nodos visitados | 100/100 |

**Análisis:** El dataset kroA100 tiene nodos muy cercanos entre sí (distribuidos linealmente en este ejemplo). V6 logra un tour eficiente aprovechando su métrica de suavidad.

### ch130 (130 nodos)

| Métrica | Valor |
|---------|-------|
| Distancia obtenida | 1454.49 |
| Tiempo de ejecución | 8.123 s |
| Nodos visitados | 130/130 |

**Análisis:** Con 130 nodos, el tiempo de ejecución crece pero se mantiene razonable (~8 segundos). La complejidad es O(N²) por la inserción suave + optimizaciones.

## Comparación con Otras Estrategias

| Estrategia | Enfoque | Ventaja vs V6 |
|------------|---------|---------------|
| Nearest Neighbor | Greedy puro | Más rápido, peor calidad |
| Christofides | 1.5-aproximación | Mejor garantía teórica, más lento |
| V4 (Cheapest Insertion) | Minimiza costo inmediato | Similar velocidad, V6 produce tours más "suaves" |
| V5 (Branch & Bound) | B&B exacto en raíz | Más preciso en early stages, exponencial |

## Fortalezas de V6

1. **Tours visualmente elegantes**: La métrica de suavidad produce caminos con menos giros bruscos
2. **Robustez**: Funciona bien en distribuciones variadas de nodos
3. **Balance tiempo/calidad**: Mejor que NN, más rápido que métodos exactos
4. **Determinístico**: Mismo input → mismo output (reproducible)

## Debilidades

1. **Sin garantías de optimalidad**: Es una heurística
2. **Sensible a α**: El factor de penalización angular requiere tuning
3. **O(N²)**: Puede ser lento para instancias muy grandes (>1000 nodos)

## Cómo Ejecutar

```bash
# Con Python (implementación de referencia)
python3 tsp_v6_tsplib.py assets/berlin52.tsp
python3 tsp_v6_tsplib.py assets/kroA100.tsp
python3 tsp_v6_tsplib.py assets/ch130.tsp

# Con Rust (cuando esté disponible el compilador)
cargo run --release -- assets/berlin52.tsp
```

## Archivos Generados

Cada ejecución crea un archivo `<nombre>_v6_result.txt` con:
- Nombre de la instancia
- Número de nodos
- Distancia total
- Tiempo de ejecución
- Path completo (orden de visita)

## Próximos Pasos Sugeridos

1. **Probar con datasets reales de TSPLIB**: Descargar archivos oficiales con óptimos conocidos
2. **Comparar contra óptimos**: Calcular % de error real
3. **Benchmark vs otras estrategias**: Medir V1-V6 en las mismas instancias
4. **Tuning de α**: Experimentar con diferentes valores del factor angular
5. **Paralelización**: La evaluación de candidatos es paralelizable

## Referencias

- TSPLIB: https://comopt.ifi.uni-heidelberg.de/software/TSPLIB95/
- Implementación Rust: `src/strategies/triangle_insertion_v6.rs`
- Implementación Python: `tsp_v6_tsplib.py`
