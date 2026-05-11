use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct NcsLayerDef {
    pub name: String,
    pub description: String,
    pub color_hex: String,
    pub line_weight: f32,
    pub line_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NcsFile {
    pub layers: Vec<NcsLayerDef>,
}

/// Recurso que almacena el catálogo completo de capas disponibles.
#[derive(Resource, Debug, Clone, Default)]
pub struct LayerStandards {
    pub catalog: HashMap<String, NcsLayerDef>,
    pub active_discipline: String,
}

/// Recurso que rastrea la capa activa actualmente para el dibujo.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveLayer(pub String);

impl Default for ActiveLayer {
    fn default() -> Self {
        Self("0".to_string())
    }
}

pub fn hex_to_rgba(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return [1.0, 1.0, 1.0, 1.0];
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255) as f32 / 255.0;
    [r, g, b, 1.0]
}

impl LayerStandards {
    pub fn new() -> Self {
        Self {
            catalog: HashMap::new(),
            active_discipline: "A".to_string(),
        }
    }

    pub fn load_from_yaml(&mut self, yaml_content: &str) -> Result<(), serde_yaml::Error> {
        let ncs_file: NcsFile = serde_yaml::from_str(yaml_content)?;
        for layer in ncs_file.layers {
            self.catalog.insert(layer.name.clone(), layer);
        }
        Ok(())
    }

    pub fn get_layer(&self, name: &str) -> Option<&NcsLayerDef> {
        self.catalog.get(name)
    }

    pub fn get_layers_by_discipline(&self, discipline: &str) -> Vec<NcsLayerDef> {
        self.catalog
            .values()
            .filter(|l| l.name.starts_with(discipline))
            .cloned()
            .collect()
    }

    pub fn get_color(&self, layer_name: &str) -> [f32; 4] {
        self.catalog
            .get(layer_name)
            .map(|l| hex_to_rgba(&l.color_hex))
            .unwrap_or([1.0, 1.0, 1.0, 1.0])
    }
}

/// Sistema ECS que detecta entidades nuevas (con geometría) que NO tienen capa,
/// y les asigna la capa activa actualmente.
pub fn assign_active_layer_system(
    mut commands: Commands,
    active_layer: Res<ActiveLayer>,
    // Usamos Changed o Added para detectar nuevas geometrías
    query: Query<Entity, (With<crate::domain::Geometry>, Without<crate::domain::Layer>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(crate::domain::Layer(active_layer.0.clone()));
    }
}
