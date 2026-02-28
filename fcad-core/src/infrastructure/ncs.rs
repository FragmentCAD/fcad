use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definición de un Estándar de Capa (NCS - National CAD Standard)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerStandard {
    pub name: String,
    pub description: String,
    pub color_hex: String,
    pub line_weight: f32,
    pub line_type: String,
}

/// Trait de inyección de dependencias para aislar la lectura del disco.
/// El fcad-core es puro y no lee los YAML directamente; en cambio,
/// recibe un objeto que implementa este Trait.
pub trait StandardsProvider: Send + Sync {
    /// Obtiene el estándar de una capa por su nombre exacto (ej. "A-WALL")
    fn get_layer_standard(&self, layer_name: &str) -> Option<LayerStandard>;

    /// Obtiene todos los estándares de capas registrados.
    fn get_all_layer_standards(&self) -> Vec<LayerStandard>;
}

/// Implementación en memoria para pruebas TDD sin tocar el disco.
pub struct InMemoryStandardsProvider {
    layers: HashMap<String, LayerStandard>,
}

impl InMemoryStandardsProvider {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    /// Método auxiliar para registrar capas en los tests
    pub fn add_layer(&mut self, standard: LayerStandard) {
        self.layers.insert(standard.name.clone(), standard);
    }
}

impl StandardsProvider for InMemoryStandardsProvider {
    fn get_layer_standard(&self, layer_name: &str) -> Option<LayerStandard> {
        self.layers.get(layer_name).cloned()
    }

    fn get_all_layer_standards(&self) -> Vec<LayerStandard> {
        self.layers.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_standards_provider() {
        let mut provider = InMemoryStandardsProvider::new();
        
        provider.add_layer(LayerStandard {
            name: "A-WALL".to_string(),
            description: "Architectural Walls".to_string(),
            color_hex: "#FF0000".to_string(),
            line_weight: 0.35,
            line_type: "Continuous".to_string(),
        });
        
        provider.add_layer(LayerStandard {
            name: "A-DOOR".to_string(),
            description: "Architectural Doors".to_string(),
            color_hex: "#00FF00".to_string(),
            line_weight: 0.18,
            line_type: "Continuous".to_string(),
        });

        // Test individual lookup
        let wall_std = provider.get_layer_standard("A-WALL").expect("Layer A-WALL should exist");
        assert_eq!(wall_std.color_hex, "#FF0000");

        let missing = provider.get_layer_standard("X-MISSING");
        assert!(missing.is_none());

        // Test all lookups
        let all_layers = provider.get_all_layer_standards();
        assert_eq!(all_layers.len(), 2);
    }
}
