use crate::domain::math::primitives::{Line, Point2D};
use crate::domain::Geometry;
use crate::infrastructure::ecs::spatial::{sync_spatial_index_system, SpatialIndex};
use bevy_ecs::prelude::*;

#[test]
fn test_hit_test_line_intersection() {
    let mut world = World::new();
    world.insert_resource(SpatialIndex::new());

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_spatial_index_system);

    // 1. Create a diagonal line from (0,0) to (10,10)
    world.spawn(Geometry::Line(Line {
        start: Point2D { x: 0.0, y: 0.0 },
        end: Point2D { x: 10.0, y: 10.0 },
    }));

    // 2. Sync spatial index
    schedule.run(&mut world);

    // 3. Query a point in the middle (5.0, 5.0)
    let spatial_index = world.resource::<SpatialIndex>();

    println!("Entities in index: {}", spatial_index.entity_bounds.len());
    for (id, bounds) in &spatial_index.entity_bounds {
        println!("Entity {:?} has bounds {:?}", id, bounds);
    }

    let results = spatial_index.query_point(5.0, 5.0);
    println!("Results for (5,5): {:?}", results);

    let results_total = spatial_index.query_area(-1.0, -1.0, 11.0, 11.0);
    println!("Results for total area: {:?}", results_total);

    assert_eq!(
        results_total.len(),
        1,
        "Should at least find it when querying the whole area"
    );
    assert_eq!(results.len(), 1, "Should find one entity at (5,5)");
}
