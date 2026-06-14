/// Módulo para cargar datasets TSPLIB
/// 
/// Soporta el formato estándar TSPLIB con coordenadas EUC_2D
use crate::core::Node;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct TspInstance {
    pub name: String,
    pub dimension: usize,
    pub nodes: Vec<Node>,
    pub optimal_distance: Option<f32>, // Si está disponible en el archivo
}

impl TspInstance {
    /// Carga una instancia desde un archivo .tsp
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Error leyendo archivo: {}", e))?;
        
        Self::parse(&content)
    }
    
    /// Parsea el contenido de un archivo TSPLIB
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut name = String::new();
        let mut dimension: Option<usize> = None;
        let mut nodes_data: Vec<(usize, f32, f32)> = Vec::new();
        let mut optimal: Option<f32> = None;
        
        let mut in_node_section = false;
        
        for line in content.lines() {
            let line = line.trim();
            
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Secciones del archivo
            if line.starts_with("NAME") {
                name = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("DIMENSION") {
                dimension = line.split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<usize>().ok());
            } else if line.starts_with("OPTIMAL") {
                optimal = line.split(':')
                    .nth(1)
                    .and_then(|s| s.trim().parse::<f32>().ok());
            } else if line.starts_with("EDGE_WEIGHT_TYPE") {
                continue;
            } else if line.starts_with("NODE_COORD_SECTION") {
                in_node_section = true;
            } else if line.starts_with("EOF") {
                in_node_section = false;
            } else if in_node_section {
                // Parsear línea de nodo: "index x y" o "index x y z"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    if let (Ok(idx), Ok(x), Ok(y)) = (
                        parts[0].parse::<usize>(),
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                    ) {
                        nodes_data.push((idx, x, y));
                    }
                }
            }
        }
        
        let dimension = dimension.ok_or("DIMENSION no especificada en el archivo")?;
        
        if nodes_data.len() != dimension {
            return Err(format!(
                "Número de nodos ({}) no coincide con DIMENSION ({})",
                nodes_data.len(),
                dimension
            ));
        }
        
        // Ordenar por índice y convertir a Node
        nodes_data.sort_by_key(|(idx, _, _)| *idx);
        let nodes: Vec<Node> = nodes_data
            .into_iter()
            .map(|(_, x, y)| Node::new(x, y))
            .collect();
        
        Ok(TspInstance {
            name,
            dimension,
            nodes,
            optimal_distance: optimal,
        })
    }
    
    /// Normaliza las coordenadas a un rango [0, 1] manteniendo el aspect ratio
    pub fn normalize(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;
        
        for node in &self.nodes {
            min_x = min_x.min(node.pos.x);
            max_x = max_x.max(node.pos.x);
            min_y = min_y.min(node.pos.y);
            max_y = max_y.max(node.pos.y);
        }
        
        let range_x = max_x - min_x;
        let range_y = max_y - min_y;
        let max_range = range_x.max(range_y).max(1e-5);
        
        for node in &mut self.nodes {
            node.pos.x = (node.pos.x - min_x) / max_range;
            node.pos.y = (node.pos.y - min_y) / max_range;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_berlin52() {
        let content = r#"NAME: berlin52
TYPE: TSP
DIMENSION: 3
EDGE_WEIGHT_TYPE: EUC_2D
NODE_COORD_SECTION
1 565.0 575.0
2 25.0 185.0
3 345.0 750.0
EOF
"#;
        
        let instance = TspInstance::parse(content).unwrap();
        assert_eq!(instance.name, "berlin52");
        assert_eq!(instance.dimension, 3);
        assert_eq!(instance.nodes.len(), 3);
    }
}
