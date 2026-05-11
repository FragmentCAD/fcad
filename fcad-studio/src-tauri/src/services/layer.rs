use crate::runtime::authority_dispatcher::MutationRequest;
use bevy_ecs::world::World;
use fcad_core::domain::theme::Theme;
use fcad_core::infrastructure::ecs::ncs::{LayerStandards, NcsLayerDef};

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
            standards
                .get_layers_by_discipline("A")
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

    pub fn set_active_layer_request(name: String) -> MutationRequest {
        MutationRequest::SetActiveLayer(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::authority_dispatcher::{dispatch_mutation, MutationRequest};
    use bevy_ecs::world::World;
    use fcad_core::infrastructure::ecs::ncs::ActiveLayer;

    #[test]
    fn test_set_active_layer() {
        let mut world = World::new();
        let name = "A-WALL".to_string();
        let request = LayerService::set_active_layer_request(name.clone());
        assert!(matches!(request, MutationRequest::SetActiveLayer(_)));
        dispatch_mutation(&mut world, request);
        let active = world.get_resource::<ActiveLayer>().unwrap();
        assert_eq!(active.0, name);
    }
}
