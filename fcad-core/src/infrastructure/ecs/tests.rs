#[cfg(test)]
mod tests {
    use bevy_ecs::prelude::*;
    use rstar::AABB;
    use crate::domain::{Geometry, Layer, Deleted};
    use crate::infrastructure::ecs::spatial::{SpatialIndex, sync_spatial_index_system};
    use crate::domain::math::primitives::{Line, Point2D};

    #[test]
    fn test_ecs_insert_and_query_layer() {
        let mut world = World::new();

        // 1. Arrange: Insert 1,000 lines into the ECS.
        // 500 will be in A-WALL, 500 in A-DOOR
        for i in 0..1000 {
            let line = Line::new(Point2D::new(0.0, 0.0), Point2D::new(i as f64, i as f64));
            let layer_name = if i % 2 == 0 { "A-WALL".to_string() } else { "A-DOOR".to_string() };
            
            world.spawn((
                Geometry::Line(line),
                Layer(layer_name)
            ));
        }

        // 2. Act: Query only active entities in A-WALL layer
        let mut query = world.query_filtered::<&Layer, Without<Deleted>>();
        
        let a_wall_count = query
            .iter(&world)
            .filter(|layer| layer.0 == "A-WALL")
            .count();

        // 3. Assert
        assert_eq!(a_wall_count, 500, "Should exactly retrieve 500 A-WALL entities from memory");
    }

    #[test]
    fn test_tombstoning_deletion() {
        let mut world = World::new();
        
        let entity = world.spawn((
            Geometry::Point(Point2D::new(1.0, 1.0)),
            Layer("E-LITE".to_string())
        )).id();

        assert_eq!(world.query::<&Layer>().iter(&world).count(), 1);

        world.entity_mut(entity).insert(Deleted);
        assert!(world.entities().contains(entity));
        
        let active_count = world.query_filtered::<&Layer, Without<Deleted>>().iter(&world).count();
        assert_eq!(active_count, 0, "Deleted entities should be filtered out from active queries");
    }

    #[test]
    fn test_spatial_index_reactivity() {
        let mut world = World::new();
        world.insert_resource(SpatialIndex::new());

        // Create a schedule to run our systems
        let mut schedule = Schedule::default();
        schedule.add_systems(sync_spatial_index_system);

        // 1. Spawn a line exactly intersecting the 10x10 bounding box
        let line_id = world.spawn((
            Geometry::Line(Line::new(Point2D::new(5.0, 5.0), Point2D::new(15.0, 15.0))),
            Layer("A-WALL".to_string())
        )).id();

        // 2. Spawn a point far away from the box
        world.spawn((
            Geometry::Point(Point2D::new(100.0, 100.0)),
            Layer("E-LITE".to_string())
        ));

        // Trigger the ECS system to sync entities into the R-Tree
        schedule.run(&mut world);

        // 3. Perform a Spatial Query (Hit Testing a 10x10 box at the origin)
        let search_envelope = AABB::from_corners([0.0, 0.0], [10.0, 10.0]);
        let spatial_index = world.resource::<SpatialIndex>();
        
        let intersections: Vec<_> = spatial_index.tree
            .locate_in_envelope_intersecting(&search_envelope)
            .collect();

        // 4. Assertions
        assert_eq!(intersections.len(), 1, "Should only find the line intersecting the box");
        assert_eq!(intersections[0].id, line_id, "The intersecting entity must be the line");
        
        // --- Test Logical Deletion (Tombstoning) spatial sync ---
        
        world.entity_mut(line_id).insert(Deleted);
        schedule.run(&mut world); // Run sync again
        
        let spatial_index_after_delete = world.resource::<SpatialIndex>();
        let intersections_after_delete: Vec<_> = spatial_index_after_delete.tree
            .locate_in_envelope_intersecting(&search_envelope)
            .collect();
            
        assert_eq!(intersections_after_delete.len(), 0, "Deleted entity should have been removed from the R-Tree");
    }
}
