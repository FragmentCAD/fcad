use bevy_ecs::{entity::Entity, query::Without, world::World};
use fcad_core::domain::math::primitives::{Line, Point2D, Rectangle};
use fcad_core::domain::{Deleted, Geometry, Layer};
use fcad_core::infrastructure::ecs::ncs::ActiveLayer;
use fcad_core::infrastructure::ecs::spatial::{calculate_aabb, SpatialEntity, SpatialIndex};

#[derive(Debug, Clone)]
pub enum MutationRequest {
    CreateLine { start: [f32; 2], end: [f32; 2] },
    CreateRectangle { p1: [f32; 2], p2: [f32; 2] },
    DeleteEntities(Vec<Entity>),
    SetActiveLayer(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpatialSyncAction {
    None,
    Incremental(Vec<Entity>),
    RebuildAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderHint {
    GeometryChanged,
    LayerChanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationOutcome {
    pub changed_entities: Vec<Entity>,
    pub spatial_sync: SpatialSyncAction,
    pub render_hint: Option<RenderHint>,
}

impl MutationOutcome {
    pub fn noop() -> Self {
        Self {
            changed_entities: Vec::new(),
            spatial_sync: SpatialSyncAction::None,
            render_hint: None,
        }
    }

    pub fn changed_entity(entity: Entity, render_hint: RenderHint) -> Self {
        Self {
            changed_entities: vec![entity],
            spatial_sync: SpatialSyncAction::Incremental(vec![entity]),
            render_hint: Some(render_hint),
        }
    }

    pub fn changed_entities(entities: Vec<Entity>, render_hint: RenderHint) -> Self {
        if entities.is_empty() {
            Self::noop()
        } else {
            Self {
                changed_entities: entities.clone(),
                spatial_sync: SpatialSyncAction::Incremental(entities),
                render_hint: Some(render_hint),
            }
        }
    }

    pub fn resource_changed(render_hint: RenderHint) -> Self {
        Self {
            changed_entities: Vec::new(),
            spatial_sync: SpatialSyncAction::None,
            render_hint: Some(render_hint),
        }
    }

    pub fn emitted_consequence(&self) -> bool {
        !self.changed_entities.is_empty()
            || !matches!(self.spatial_sync, SpatialSyncAction::None)
            || self.render_hint.is_some()
    }
}

pub fn dispatch_mutation(world: &mut World, request: MutationRequest) -> MutationOutcome {
    match request {
        MutationRequest::CreateLine { start, end } => {
            let layer = active_layer_name(world);
            let entity = world
                .spawn((
                    Geometry::Line(Line {
                        start: Point2D::new(start[0] as f64, start[1] as f64),
                        end: Point2D::new(end[0] as f64, end[1] as f64),
                    }),
                    Layer(layer),
                ))
                .id();

            MutationOutcome::changed_entity(entity, RenderHint::GeometryChanged)
        }
        MutationRequest::CreateRectangle { p1, p2 } => {
            let layer = active_layer_name(world);
            let entity = world
                .spawn((
                    Geometry::Rectangle(Rectangle {
                        p1: Point2D::new(p1[0] as f64, p1[1] as f64),
                        p2: Point2D::new(p2[0] as f64, p2[1] as f64),
                    }),
                    Layer(layer),
                ))
                .id();

            MutationOutcome::changed_entity(entity, RenderHint::GeometryChanged)
        }
        MutationRequest::DeleteEntities(entities) => {
            let mut changed = Vec::new();

            for entity in entities {
                if let Some(mut entity_ref) = world.get_entity_mut(entity) {
                    if !entity_ref.contains::<Deleted>() {
                        entity_ref.insert(Deleted);
                        changed.push(entity);
                    }
                }
            }

            MutationOutcome::changed_entities(changed, RenderHint::GeometryChanged)
        }
        MutationRequest::SetActiveLayer(name) => {
            if world
                .get_resource::<ActiveLayer>()
                .map(|active| active.0 == name)
                .unwrap_or(false)
            {
                return MutationOutcome::noop();
            }

            world.insert_resource(ActiveLayer(name));
            MutationOutcome::resource_changed(RenderHint::LayerChanged)
        }
    }
}

pub fn derive_spatial_index(world: &mut World) -> SpatialIndex {
    let mut spatial_index = SpatialIndex::new();
    let mut query = world.query_filtered::<(Entity, &Geometry), Without<Deleted>>();

    for (entity, geometry) in query.iter(world) {
        let envelope = calculate_aabb(geometry);
        spatial_index.tree.insert(SpatialEntity {
            id: entity,
            envelope,
        });
        spatial_index.entity_bounds.insert(entity, envelope);
    }

    spatial_index
}

fn active_layer_name(world: &World) -> String {
    world
        .get_resource::<ActiveLayer>()
        .cloned()
        .unwrap_or_default()
        .0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_line_returns_geometry_consequence() {
        let mut world = World::new();
        world.insert_resource(ActiveLayer("A-WALL".to_string()));

        let outcome = dispatch_mutation(
            &mut world,
            MutationRequest::CreateLine {
                start: [0.0, 0.0],
                end: [10.0, 10.0],
            },
        );

        assert!(outcome.emitted_consequence());
        assert_eq!(outcome.render_hint, Some(RenderHint::GeometryChanged));
        assert_eq!(outcome.changed_entities.len(), 1);
        assert_eq!(world.query::<&Geometry>().iter(&world).count(), 1);
    }

    #[test]
    fn repeated_active_layer_change_is_noop() {
        let mut world = World::new();
        world.insert_resource(ActiveLayer("A-WALL".to_string()));

        let outcome = dispatch_mutation(
            &mut world,
            MutationRequest::SetActiveLayer("A-WALL".to_string()),
        );

        assert!(!outcome.emitted_consequence());
    }

    #[test]
    fn derived_index_tracks_create_and_logical_delete() {
        let mut world = World::new();
        let create = dispatch_mutation(
            &mut world,
            MutationRequest::CreateRectangle {
                p1: [0.0, 0.0],
                p2: [10.0, 10.0],
            },
        );

        let index = derive_spatial_index(&mut world);
        assert_eq!(index.query_point(5.0, 5.0), create.changed_entities);

        dispatch_mutation(
            &mut world,
            MutationRequest::DeleteEntities(create.changed_entities),
        );
        let index = derive_spatial_index(&mut world);
        assert!(index.query_point(5.0, 5.0).is_empty());
    }
}
