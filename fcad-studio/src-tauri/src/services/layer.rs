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
