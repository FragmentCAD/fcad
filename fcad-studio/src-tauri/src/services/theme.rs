use fcad_core::domain::theme::Theme;
use std::path::Path;

pub struct ThemeService;

impl ThemeService {
    pub fn get_themes_list() -> Vec<String> {
        let themes_dir = Path::new("..").join("..").join("fcad-assets").join("themes");
        let mut themes = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(themes_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".json") {
                        themes.push(name.replace(".json", ""));
                    }
                }
            }
        }
        themes
    }

    pub fn load_theme(theme_name: &str) -> Result<Theme, String> {
        let theme_path = Path::new("..").join("..").join("fcad-assets").join("themes").join(format!("{}.json", theme_name));
        
        let content = std::fs::read_to_string(&theme_path)
            .map_err(|e| format!("No se pudo leer el tema '{}': {}", theme_name, e))?;
        
        let theme: Theme = serde_json::from_str(&content)
            .map_err(|e| format!("Error al parsear el tema '{}': {}", theme_name, e))?;
        
        Ok(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_non_existent_theme() {
        let result = ThemeService::load_theme("does-not-exist");
        assert!(result.is_err());
    }
}
