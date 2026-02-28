use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub accent: String,
    pub grid_major: String,
    pub grid_minor: String,
    pub selection: String,
}

impl Default for Theme {
    fn default() -> Self {
        // En un entorno Tauri/Studio, podríamos consultar el modo del sistema.
        // Como fallback razonable en Core, Midnight es el estándar CAD (oscuro).
        Self::midnight()
    }
}

impl Theme {
    pub fn midnight() -> Self {
        Self {
            name: "Midnight".to_string(),
            background: "#000000".to_string(),
            foreground: "#F0F0F0".to_string(),
            primary: "#3B82F6".to_string(),
            accent: "#1E293B".to_string(),
            grid_major: "#262626".to_string(),
            grid_minor: "#141414".to_string(),
            selection: "#60A5FA".to_string(),
        }
    }

    pub fn architect() -> Self {
        Self {
            name: "Architect".to_string(),
            background: "#FDFDFB".to_string(),
            foreground: "#1A202C".to_string(),
            primary: "#2D3748".to_string(),
            accent: "#CBD5E0".to_string(),
            grid_major: "#E2E8F0".to_string(),
            grid_minor: "#EDF2F7".to_string(),
            selection: "#4A5568".to_string(),
        }
    }
}
