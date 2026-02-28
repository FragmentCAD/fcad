use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Clone)]
pub struct NcsDictionary {
    pub layers: HashMap<String, [f32; 4]>,
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

impl NcsDictionary {
    pub fn new() -> Self {
        Self { layers: HashMap::new() }
    }

    pub fn load_from_yaml(yaml_content: &str) -> Result<Self, serde_yaml::Error> {
        let ncs_file: NcsFile = serde_yaml::from_str(yaml_content)?;
        let mut map = HashMap::new();
        for layer in ncs_file.layers {
            map.insert(layer.name, hex_to_rgba(&layer.color_hex));
        }
        Ok(Self { layers: map })
    }

    pub fn get_color(&self, layer_name: &str) -> [f32; 4] {
        self.layers.get(layer_name).copied().unwrap_or([1.0, 1.0, 1.0, 1.0])
    }
}
