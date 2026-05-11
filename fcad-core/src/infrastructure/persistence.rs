use bevy_ecs::prelude::*;
use bincode::Options;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};

use crate::domain::{ColorOverride, Deleted, Geometry, Layer};

/// Represents an entity's physical state for saving/loading.
/// We don't save Bevy's internal IDs because across sessions they can be volatile.
#[derive(Serialize, Deserialize)]
pub struct SerializedEntity {
    pub geometry: Option<Geometry>,
    pub layer: Option<Layer>,
    pub color_override: Option<ColorOverride>,
    pub deleted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectData {
    pub entities: Vec<SerializedEntity>,
}

/// Saves the pure ECS state to a binary file using bincode, avoiding OOM using limit.
pub fn save_project(path: &str, world: &mut World) -> Result<(), Box<dyn std::error::Error>> {
    let mut entities = Vec::new();

    let mut query = world.query::<(
        Option<&Geometry>,
        Option<&Layer>,
        Option<&ColorOverride>,
        Has<Deleted>,
    )>();

    for (geometry, layer, color, deleted) in query.iter(world) {
        // Enforce save of CAD-like entities only.
        if geometry.is_none() && layer.is_none() {
            continue;
        }
        entities.push(SerializedEntity {
            geometry: geometry.cloned(),
            layer: layer.cloned(),
            color_override: color.cloned(),
            deleted,
        });
    }

    let project = ProjectData { entities };

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    // 512 MB strict limit to avoid OOM
    let bincode_opts = bincode::DefaultOptions::new().with_limit(512 * 1024 * 1024);
    bincode_opts.serialize_into(&mut writer, &project)?;

    Ok(())
}

/// Loads the pure ECS state from a binary file using bincode.
pub fn load_project(path: &str, world: &mut World) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let bincode_opts = bincode::DefaultOptions::new().with_limit(512 * 1024 * 1024);
    let project: ProjectData = bincode_opts.deserialize_from(&mut reader)?;

    // Clear all existing entities from the world to load anew
    world.clear_entities();

    for se in project.entities {
        let mut builder = world.spawn_empty();
        if let Some(geom) = se.geometry {
            builder.insert(geom);
        }
        if let Some(layer) = se.layer {
            builder.insert(layer);
        }
        if let Some(color) = se.color_override {
            builder.insert(color);
        }
        if se.deleted {
            builder.insert(Deleted);
        }
    }

    Ok(())
}

/// Command Pattern for Undo/Redo (Deltas)
#[derive(Clone)]
pub enum Command {
    SpawnEntity(Entity),
    DeleteEntity(Entity),
    ModifyGeometry {
        entity: Entity,
        old_geometry: Geometry,
        new_geometry: Geometry,
    },
}

#[derive(Resource, Default)]
pub struct CommandHistory {
    pub undo_stack: Vec<Command>,
    pub redo_stack: Vec<Command>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, cmd: Command) {
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, world: &mut World) {
        if let Some(cmd) = self.undo_stack.pop() {
            match cmd.clone() {
                Command::SpawnEntity(e) => {
                    if let Some(mut entity) = world.get_entity_mut(e) {
                        entity.insert(Deleted);
                    }
                    self.redo_stack.push(cmd);
                }
                Command::DeleteEntity(e) => {
                    if let Some(mut entity) = world.get_entity_mut(e) {
                        entity.remove::<Deleted>();
                    }
                    self.redo_stack.push(cmd);
                }
                Command::ModifyGeometry {
                    entity,
                    old_geometry,
                    ..
                } => {
                    if let Some(mut e) = world.get_entity_mut(entity) {
                        e.insert(old_geometry);
                    }
                    self.redo_stack.push(cmd);
                }
            }
        }
    }

    pub fn redo(&mut self, world: &mut World) {
        if let Some(cmd) = self.redo_stack.pop() {
            match cmd.clone() {
                Command::SpawnEntity(e) => {
                    if let Some(mut entity) = world.get_entity_mut(e) {
                        entity.remove::<Deleted>();
                    }
                    self.undo_stack.push(cmd);
                }
                Command::DeleteEntity(e) => {
                    if let Some(mut entity) = world.get_entity_mut(e) {
                        entity.insert(Deleted);
                    }
                    self.undo_stack.push(cmd);
                }
                Command::ModifyGeometry {
                    entity,
                    new_geometry,
                    ..
                } => {
                    if let Some(mut e) = world.get_entity_mut(entity) {
                        e.insert(new_geometry);
                    }
                    self.undo_stack.push(cmd);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::math::primitives::{Line, Point2D};
    use crate::domain::{Deleted, Geometry, Layer};

    use std::fs;

    #[test]
    fn test_bincode_persistence() {
        let mut world = World::new();
        world.spawn((
            Geometry::Line(Line::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0))),
            Layer("A-WALL".to_string()),
        ));

        let path = "test_project.bin";
        save_project(path, &mut world).unwrap();

        let mut new_world = World::new();
        load_project(path, &mut new_world).unwrap();

        let count = new_world.query::<&Geometry>().iter(&new_world).count();
        assert_eq!(
            count, 1,
            "The loaded project should contain 1 geometry entity"
        );

        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_undo_redo_commands_memory() {
        let mut world = World::new();
        let mut history = CommandHistory::new();

        // Spawn 10 lines and record commands
        let mut entities = vec![];
        for i in 0..10 {
            let entity = world
                .spawn((
                    Geometry::Line(Line::new(
                        Point2D::new(0.0, 0.0),
                        Point2D::new(i as f64, 0.0),
                    )),
                    Layer("A-WALL".to_string()),
                ))
                .id();

            history.push(Command::SpawnEntity(entity));
            entities.push(entity);
        }

        let active_count = world
            .query_filtered::<Entity, Without<Deleted>>()
            .iter(&world)
            .count();
        assert_eq!(active_count, 10, "Should have 10 active entities initially");

        // Apply Undo 5 times
        for _ in 0..5 {
            history.undo(&mut world);
        }

        // Half of entities should now be tombstoned
        let active_count = world
            .query_filtered::<Entity, Without<Deleted>>()
            .iter(&world)
            .count();
        let deleted_count = world
            .query_filtered::<Entity, With<Deleted>>()
            .iter(&world)
            .count();

        assert_eq!(active_count, 5, "5 entities should be active after 5 undos");
        assert_eq!(
            deleted_count, 5,
            "5 entities should be logically deleted (tombstoned)"
        );

        // Apply Redo 3 times
        for _ in 0..3 {
            history.redo(&mut world);
        }

        let active_count = world
            .query_filtered::<Entity, Without<Deleted>>()
            .iter(&world)
            .count();
        assert_eq!(active_count, 8, "8 entities should be active after 3 redos");

        // We modified the state without needing to clone/duplicate the entire ECS World!
    }
}
