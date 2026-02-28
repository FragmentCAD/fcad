#[cfg(test)]
mod tests {
    use crate::infrastructure::ecs::ncs::{LayerStandards, NcsLayerDef};

    #[test]
    fn test_ncs_yaml_full_parsing() {
        let yaml = r##"
layers:
  - name: "A-WALL"
    description: "Architectural Walls"
    color_hex: "#FF0000"
    line_weight: 0.35
    line_type: "Continuous"
"##;
        let mut standards = LayerStandards::new();
        standards.load_from_yaml(yaml).expect("Failed to parse YAML");

        let wall = standards.get_layer("A-WALL").expect("Layer not found");
        assert_eq!(wall.name, "A-WALL");
        assert_eq!(wall.line_weight, 0.35);
        assert_eq!(wall.line_type, "Continuous");
        
        let color = standards.get_color("A-WALL");
        assert_eq!(color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_ncs_get_layer_not_found() {
        let standards = LayerStandards::new();
        assert!(standards.get_layer("NON-EXISTENT").is_none());
        assert_eq!(standards.get_color("NON-EXISTENT"), [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_get_layers_by_discipline() {
        let mut standards = LayerStandards::new();
        let yaml = r##"
layers:
  - name: "A-WALL"
    description: "Architectural Walls"
    color_hex: "#000000"
    line_weight: 0.35
    line_type: "Continuous"
  - name: "S-BEAM"
    description: "Structural Beam"
    color_hex: "#0000FF"
    line_weight: 0.5
    line_type: "Continuous"
"##;
        standards.load_from_yaml(yaml).unwrap();
        
        // Esta función aún no existe -> Fallo de compilación o ejecución
        let a_layers = standards.get_layers_by_discipline("A");
        assert_eq!(a_layers.len(), 1);
        assert_eq!(a_layers[0].name, "A-WALL");
    }

    #[test]
    fn test_active_layer_resource_default() {
        use crate::infrastructure::ecs::ncs::ActiveLayer;
        let active = ActiveLayer::default();
        assert_eq!(active.0, "0"); // Capa por defecto es "0"
    }
}
