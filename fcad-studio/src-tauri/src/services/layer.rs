use fcad_core::infrastructure::ecs::ncs::{LayerStandards, NcsLayerDef, ActiveLayer};
use fcad_core::domain::theme::Theme;
use bevy_ecs::world::World;

pub struct LayerService;

impl LayerService {
    pub fn get_layers(world: &World) -> Vec<NcsLayerDef> {
        if let Some(standards) = world.get_resource::<LayerStandards>() {
            standards.get_layers_by_discipline("A")
        } else {
            Vec::new()
        }
    }

    pub fn get_adapted_layers(world: &World, theme: &Theme) -> Vec<NcsLayerDef> {
        if let Some(standards) = world.get_resource::<LayerStandards>() {
            standards.get_layers_by_discipline("A")
                .into_iter()
                .map(|mut layer| {
                    layer.color_hex = theme.adapt_layer_color(&layer.color_hex);
                    layer
                })
                .collect()
        } else {
            Vec::new()
        }
    }
    
    pub fn set_active_layer(world: &mut World, name: String) -> String {
        world.insert_resource(ActiveLayer(name.clone()));
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::world::World;
    use fcad_core::infrastructure::ecs::ncs::{LayerStandards, ActiveLayer};

    #[test]
    fn test_set_active_layer() {
        let mut world = World::new();
        let name = "A-WALL".to_string();
        LayerService::set_active_layer(&mut world, name.clone());
        let active = world.get_resource::<ActiveLayer>().unwrap();
        assert_eq!(active.0, name);
    }
}
