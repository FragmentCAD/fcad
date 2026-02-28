use bevy_ecs::prelude::*;
use rstar::{RTree, RTreeObject, AABB, PointDistance};
use std::collections::HashMap;
use crate::domain::{Geometry, Deleted};

/// Wrapper to store an ECS Entity ID alongside its AABB inside the RTree.
#[derive(Debug, Clone, PartialEq)]
pub struct SpatialEntity {
    pub id: Entity,
    pub envelope: AABB<[f64; 2]>,
}

impl RTreeObject for SpatialEntity {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for SpatialEntity {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.envelope.distance_2(point)
    }
}

/// The main R-Tree Resource injected into the Bevy World.
/// Stores `SpatialEntity` elements for ultra-fast geographical hit-testing.
#[derive(Resource, Default)]
pub struct SpatialIndex {
    pub tree: RTree<SpatialEntity>,
    pub entity_bounds: HashMap<Entity, AABB<[f64; 2]>>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        Self {
            tree: RTree::new(),
            entity_bounds: HashMap::new(),
        }
    }

    /// Returns a list of Entity IDs whose Bounding Box contains the given point.
    pub fn query_point(&self, x: f64, y: f64) -> Vec<Entity> {
        let point = [x, y];
        self.tree
            .locate_all_at_point(&point)
            .map(|e| e.id)
            .collect()
    }

    /// Returns a list of Entity IDs whose Bounding Box intersects the given area.
    /// Useful for 'click tolerance' or selection windows.
    pub fn query_area(&self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Vec<Entity> {
        let envelope = AABB::from_corners([min_x, min_y], [max_x, max_y]);
        self.tree
            .locate_in_envelope(&envelope)
            .map(|e| e.id)
            .collect()
    }
}

/// Helper function to calculate the AABB (Bounding Box) of our custom Geometry.
pub fn calculate_aabb(geometry: &Geometry) -> AABB<[f64; 2]> {
    match geometry {
        Geometry::Point(p) => AABB::from_point([p.x, p.y]),
        Geometry::Line(l) => {
            let min_x = f64::min(l.start.x, l.end.x);
            let min_y = f64::min(l.start.y, l.end.y);
            let max_x = f64::max(l.start.x, l.end.x);
            let max_y = f64::max(l.start.y, l.end.y);
            AABB::from_corners([min_x, min_y], [max_x, max_y])
        }
        Geometry::Circle(c) => {
            let min_x = c.center.x - c.radius;
            let min_y = c.center.y - c.radius;
            let max_x = c.center.x + c.radius;
            let max_y = c.center.y + c.radius;
            AABB::from_corners([min_x, min_y], [max_x, max_y])
        }
        Geometry::Arc(a) => {
            // Simplified AABB for Arc: treats it as its bounding circle. 
            // In a production CAD, this would precisely compute bezier extremes.
            let min_x = a.center.x - a.radius;
            let min_y = a.center.y - a.radius;
            let max_x = a.center.x + a.radius;
            let max_y = a.center.y + a.radius;
            AABB::from_corners([min_x, min_y], [max_x, max_y])
        }
    }
}

/// Reactively syncs newly Added or Changed geometries into the SpatialIndex Resource.
pub fn sync_spatial_index_system(
    mut spatial_index: ResMut<SpatialIndex>,
    query: Query<(Entity, &Geometry), Or<(Added<Geometry>, Changed<Geometry>)>>,
    deleted_query: Query<Entity, Added<Deleted>>,
) {
    for (entity, geometry) in query.iter() {
        // We remove any potential old footprint to prevent duplicates upon 'Changed'
        if let Some(old_bounds) = spatial_index.entity_bounds.get(&entity).copied() {
            spatial_index.tree.remove(&SpatialEntity { id: entity, envelope: old_bounds });
        }
        
        // Insert the fresh AABB
        let envelope = calculate_aabb(geometry);
        spatial_index.tree.insert(SpatialEntity { id: entity, envelope });
        spatial_index.entity_bounds.insert(entity, envelope);
    }

    // Handle Tombstoning: Remove from spatial index if the entity was logically deleted
    for entity in deleted_query.iter() {
        if let Some(old_bounds) = spatial_index.entity_bounds.remove(&entity) {
            spatial_index.tree.remove(&SpatialEntity { id: entity, envelope: old_bounds });
        }
    }
}

