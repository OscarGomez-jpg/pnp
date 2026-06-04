#!/usr/bin/env python3
"""
Implementación en Python de Triangle Insertion V6 (Smoothest Angle Insertion)
para comparar con datasets TSPLIB como berlin52, kroA100, ch130, etc.

Este script replica la lógica del archivo triangle_insertion_v6.rs en Python.
"""

import math
from typing import List, Tuple, Optional
from dataclasses import dataclass


@dataclass
class Node:
    x: float
    y: float
    
    def distance_to(self, other: 'Node') -> float:
        return math.sqrt((self.x - other.x)**2 + (self.y - other.y)**2)


def path_distance(path: List[int], nodes: List[Node]) -> float:
    """Calcula la distancia total de un camino cerrado"""
    if len(path) < 2:
        return 0.0
    dist = 0.0
    for i in range(len(path)):
        dist += nodes[path[i]].distance_to(nodes[path[(i + 1) % len(path)]])
    return dist


def insertion_cost(edge_start: int, edge_end: int, node_idx: int, nodes: List[Node]) -> float:
    """Calcula el costo de insertar un nodo en una arista"""
    p_start = nodes[edge_start]
    p_end = nodes[edge_end]
    p_new = nodes[node_idx]
    
    new_cost = p_start.distance_to(p_new) + p_new.distance_to(p_end)
    old_cost = p_start.distance_to(p_end)
    return new_cost - old_cost


def convex_hull(nodes: List[Node]) -> List[int]:
    """Algoritmo de Convex Hull (Monotone Chain)"""
    if len(nodes) < 3:
        return list(range(len(nodes)))
    
    indexed = list(range(len(nodes)))
    indexed.sort(key=lambda i: (nodes[i].x, nodes[i].y))
    
    def cross(o: int, a: int, b: int) -> float:
        return (nodes[a].x - nodes[o].x) * (nodes[b].y - nodes[o].y) - \
               (nodes[a].y - nodes[o].y) * (nodes[b].x - nodes[o].x)
    
    lower = []
    for idx in indexed:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], idx) <= 0.0:
            lower.pop()
        lower.append(idx)
    
    upper = []
    for idx in reversed(indexed):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], idx) <= 0.0:
            upper.pop()
        upper.append(idx)
    
    lower.pop()
    upper.pop()
    return lower + upper


def triangle_perimeter(a: int, b: int, c: int, nodes: List[Node]) -> float:
    return nodes[a].distance_to(nodes[b]) + \
           nodes[b].distance_to(nodes[c]) + \
           nodes[c].distance_to(nodes[a])


def best_triangle_from_hull(nodes: List[Node]) -> List[int]:
    hull = convex_hull(nodes)
    if len(hull) < 3:
        return list(range(min(len(nodes), 3)))
    
    n = len(hull)
    best_triangle = [hull[0], hull[1], hull[2]]
    best_perimeter = triangle_perimeter(hull[0], hull[1], hull[2], nodes)
    
    for i in range(n):
        for j in range(i + 1, n):
            for k in range(j + 1, n):
                p = triangle_perimeter(hull[i], hull[j], hull[k], nodes)
                if p > best_perimeter:
                    best_perimeter = p
                    best_triangle = [hull[i], hull[j], hull[k]]
    
    return best_triangle


def insertion_angle(i: int, j: int, u: int, nodes: List[Node]) -> float:
    """Calcula el ángulo formado en el punto u cuando se inserta entre i y j"""
    p_i = nodes[i]
    p_j = nodes[j]
    p_u = nodes[u]
    
    v1 = (p_i.x - p_u.x, p_i.y - p_u.y)
    v2 = (p_j.x - p_u.x, p_j.y - p_u.y)
    
    len1 = math.sqrt(v1[0]**2 + v1[1]**2)
    len2 = math.sqrt(v2[0]**2 + v2[1]**2)
    
    if len1 < 1e-5 or len2 < 1e-5:
        return 0.0
    
    cos_theta = (v1[0] * v2[0] + v1[1] * v2[1]) / (len1 * len2)
    cos_theta = max(-1.0, min(1.0, cos_theta))
    return math.acos(cos_theta)


def smoothest_insertion(path: List[int], unvisited: List[int], nodes: List[Node]) -> Tuple[int, int]:
    """
    Inserción por Ángulo más Suave (V6 Core).
    score = insertion_cost * (1 + α * (1 + cos θ))
    """
    alpha = 2.0
    
    best_node = unvisited[0]
    best_pos = 1
    best_score = float('inf')
    
    for candidate in unvisited:
        for i in range(len(path)):
            next_idx = (i + 1) % len(path)
            
            cost = insertion_cost(path[i], next_idx, candidate, nodes)
            
            p_i = nodes[path[i]]
            p_next = nodes[next_idx]
            p_u = nodes[candidate]
            
            v1 = (p_i.x - p_u.x, p_i.y - p_u.y)
            v2 = (p_next.x - p_u.x, p_next.y - p_u.y)
            
            len1 = math.sqrt(v1[0]**2 + v1[1]**2)
            len2 = math.sqrt(v2[0]**2 + v2[1]**2)
            
            if len1 > 1e-5 and len2 > 1e-5:
                cos_theta = (v1[0] * v2[0] + v1[1] * v2[1]) / (len1 * len2)
                cos_theta = max(-1.0, min(1.0, cos_theta))
            else:
                cos_theta = 1.0
            
            # cos_theta = -1 (línea recta, ideal) → penalty = 0
            # cos_theta = +1 (giro en U, terrible) → penalty = 2α * cost
            score = cost * (1.0 + alpha * (1.0 + cos_theta))
            
            if score < best_score:
                best_score = score
                best_node = candidate
                best_pos = i + 1
    
    return best_node, best_pos


def optimize_2opt(path: List[int], nodes: List[Node], max_iterations: int = 100) -> bool:
    """Optimización 2-opt"""
    improved = False
    
    for _ in range(max_iterations):
        local_improved = False
        
        for i in range(len(path) - 2):
            for j in range(i + 2, len(path)):
                if i == 0 and j == len(path) - 1:
                    continue
                
                p1 = nodes[path[i]]
                p2 = nodes[path[i + 1]]
                p3 = nodes[path[j]]
                p4 = nodes[path[(j + 1) % len(path)]]
                
                current = p1.distance_to(p2) + p3.distance_to(p4)
                swapped = p1.distance_to(p3) + p2.distance_to(p4)
                
                if swapped < current - 0.01:
                    path[i + 1:j + 1] = path[i + 1:j + 1][::-1]
                    local_improved = True
                    improved = True
        
        if not local_improved:
            break
    
    return improved


def optimize_or_opt(path: List[int], nodes: List[Node], seg_len: int) -> bool:
    """Optimización Or-opt"""
    n = len(path)
    if n < seg_len + 2:
        return False
    
    improved = True
    
    while improved:
        improved = False
        current_dist = path_distance(path, nodes)
        
        for i in range(n):
            seg = [path[(i + k) % n] for k in range(seg_len)]
            
            reduced = []
            for k in range(n):
                if k < i or k >= i + seg_len:
                    reduced.append(path[k])
            
            m = len(reduced)
            if m < 2:
                continue
            
            for j in range(m + 1):
                candidate = reduced[:min(j, m)] + seg + reduced[j:]
                
                if len(candidate) != n:
                    continue
                
                dist = path_distance(candidate, nodes)
                if dist < current_dist - 0.01:
                    path[:] = candidate
                    improved = True
                    break
            
            if improved:
                break
    
    return improved


def optimize_node_reinsertion(path: List[int], nodes: List[Node]) -> bool:
    """Saca cada nodo del tour y lo re-inserta en la posición óptima"""
    if len(path) < 4:
        return False
    
    ever_improved = False
    improved = True
    
    while improved:
        improved = False
        current_dist = path_distance(path, nodes)
        
        for idx in range(len(path)):
            node = path[idx]
            
            reduced = path[:idx] + path[idx + 1:]
            
            best_pos = idx
            best_dist = current_dist
            
            for j in range(len(reduced) + 1):
                candidate = reduced[:j] + [node] + reduced[j:]
                dist = path_distance(candidate, nodes)
                if dist < best_dist - 0.01:
                    best_dist = dist
                    best_pos = j
            
            if best_dist < current_dist - 0.01:
                new_path = reduced[:best_pos] + [node] + reduced[best_pos:]
                path[:] = new_path
                improved = True
                ever_improved = True
                break
    
    return ever_improved


def triangle_insertion_v6(nodes: List[Node]) -> List[int]:
    """
    Implementación completa de Triangle Insertion V6
    """
    if len(nodes) < 3:
        return list(range(len(nodes)))
    
    # Paso 1: Inicialización con triángulo del convex hull
    triangle = best_triangle_from_hull(nodes)
    path = triangle.copy()
    
    # Paso 2: Inserción iterativa
    while len(path) < len(nodes):
        unvisited = [i for i in range(len(nodes)) if i not in path]
        
        # Inserción por ángulo más suave
        best_node, best_pos = smoothest_insertion(path, unvisited, nodes)
        path.insert(best_pos, best_node)
    
    # Paso 3: Post-optimización
    optimize_2opt(path, nodes, 10)
    optimize_or_opt(path, nodes, 1)
    optimize_or_opt(path, nodes, 2)
    optimize_node_reinsertion(path, nodes)
    optimize_2opt(path, nodes, 5)
    
    return path


def parse_tsp_file(filepath: str) -> Tuple[str, List[Node], Optional[float]]:
    """Parsea un archivo TSPLIB"""
    name = ""
    dimension = 0
    nodes = []
    optimal = None
    in_node_section = False
    
    with open(filepath, 'r') as f:
        for line in f:
            line = line.strip()
            
            if not line or line.startswith('#'):
                continue
            
            if line.startswith("NAME:"):
                name = line.split(':', 1)[1].strip()
            elif line.startswith("DIMENSION:"):
                dimension = int(line.split(':', 1)[1].strip())
            elif line.startswith("OPTIMAL:"):
                optimal = float(line.split(':', 1)[1].strip())
            elif line.startswith("NODE_COORD_SECTION"):
                in_node_section = True
            elif line.startswith("EOF"):
                in_node_section = False
            elif in_node_section:
                parts = line.split()
                if len(parts) >= 3:
                    idx = int(parts[0])
                    x = float(parts[1])
                    y = float(parts[2])
                    nodes.append(Node(x, y))
    
    if len(nodes) != dimension:
        print(f"Advertencia: nodos ({len(nodes)}) != DIMENSION ({dimension})")
    
    return name, nodes, optimal


def main():
    import sys
    
    if len(sys.argv) < 2:
        print("Uso: python tsp_v6_tsplib.py <archivo.tsp>")
        print("Ejemplos:")
        print("  python tsp_v6_tsplib.py assets/berlin52.tsp")
        print("  python tsp_v6_tsplib.py assets/kroA100.tsp")
        print("  python tsp_v6_tsplib.py assets/ch130.tsp")
        sys.exit(1)
    
    filepath = sys.argv[1]
    
    print(f"Cargando {filepath}...")
    name, nodes, optimal = parse_tsp_file(filepath)
    print(f"Instancia: {name}")
    print(f"Nodos: {len(nodes)}")
    
    if optimal is not None:
        print(f"Distancia óptima conocida: {optimal:.2f}")
    
    print("\nEjecutando Triangle Insertion V6...")
    import time
    start = time.time()
    path = triangle_insertion_v6(nodes)
    elapsed = time.time() - start
    
    distance = path_distance(path, nodes)
    print(f"\nResultado:")
    print(f"  Distancia: {distance:.2f}")
    print(f"  Tiempo: {elapsed:.3f} segundos")
    
    if optimal is not None:
        error = ((distance - optimal) / optimal) * 100
        print(f"  Error respecto al óptimo: {error:.2f}%")
    
    print(f"\nPrimeros 10 nodos del path: {path[:10]}")
    
    # Guardar resultado
    output_file = filepath.replace('.tsp', '_v6_result.txt')
    with open(output_file, 'w') as f:
        f.write(f"Instance: {name}\n")
        f.write(f"Nodes: {len(nodes)}\n")
        f.write(f"Distance: {distance:.2f}\n")
        if optimal:
            f.write(f"Optimal: {optimal:.2f}\n")
            f.write(f"Error: {((distance - optimal) / optimal) * 100:.2f}%\n")
        f.write(f"Time: {elapsed:.3f}s\n")
        f.write(f"Path: {path}\n")
    
    print(f"\nResultado guardado en: {output_file}")


if __name__ == "__main__":
    main()
