use bevy_ecs::prelude::*;
use fcad_core::domain::math::primitives::{Line, Point2D};
use fcad_core::domain::{ColorOverride, Geometry, Layer};
use fcad_core::infrastructure::ecs::ncs::LayerStandards;
use std::collections::HashMap;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineInstance {
    pub start: [f32; 2],
    pub end: [f32; 2],
    pub color: [f32; 4],
    pub thickness: f32,
    pub _padding: [f32; 3],
}

/// Manejador de sincronización diferencial entre ECS y Hardware (VRAM).
pub struct RenderOptimizer {
    /// Mapeo de Entidad ECS a posiciones en el Array contiguo (soporta geometrías multi-línea como Rectángulos)
    pub entity_to_index: HashMap<Entity, Vec<usize>>,
    /// Array contiguo de instancias listo para subir a VRAM
    pub instances: Vec<LineInstance>,
    /// Índices de ranuras liberadas (para reutilizar cuando se borran entidades)
    pub free_slots: Vec<usize>,

    // Tracking interno de lo que se debe subir a VRAM este frame
    pub dirty_ranges: Vec<usize>,

    pub ncs_dict: LayerStandards,
}

impl Default for RenderOptimizer {
    fn default() -> Self {
        Self::new(LayerStandards::new())
    }
}

impl RenderOptimizer {
    pub fn new(ncs_dict: LayerStandards) -> Self {
        Self {
            entity_to_index: HashMap::new(),
            instances: Vec::new(),
            free_slots: Vec::new(),
            dirty_ranges: Vec::new(),
            ncs_dict,
        }
    }

    /// Sistema para sincronizar ECS a la configuración local del optimizador
    pub fn sync_system<'a>(
        &mut self,
        added: impl IntoIterator<
            Item = (
                Entity,
                &'a Geometry,
                Option<&'a Layer>,
                Option<&'a ColorOverride>,
            ),
        >,
        changed: impl IntoIterator<
            Item = (
                Entity,
                &'a Geometry,
                Option<&'a Layer>,
                Option<&'a ColorOverride>,
            ),
        >,
        removed: impl IntoIterator<Item = Entity>,
    ) {
        self.dirty_ranges.clear();

        // 1. Procesar Eliminaciones (Tombstoning)
        for entity in removed {
            if let Some(indices) = self.entity_to_index.remove(&entity) {
                for index in indices {
                    self.instances[index].thickness = 0.0;
                    self.free_slots.push(index);
                    self.dirty_ranges.push(index);
                }
            }
        }

        // 2. Procesar Modificaciones
        for (entity, geometry, layer, color_override) in changed {
            // Solo procesar como cambio si la entidad ya existe en el optimizador
            if self.entity_to_index.contains_key(&entity) {
                // Simplificación: Eliminamos y re-añadimos para cambios de tipo o tamaño de slots
                // En el futuro podemos optimizar si el número de líneas coincide
                if let Some(indices) = self.entity_to_index.remove(&entity) {
                    for index in indices {
                        self.instances[index].thickness = 0.0;
                        self.free_slots.push(index);
                        // Don't mark as dirty here - we'll mark it when we add the new geometry
                    }
                }
                // Re-procesar como adición (marcará dirty en add_geometry)
                self.add_geometry(entity, geometry, layer, color_override);
            }
        }

        // 3. Procesar Adiciones
        for (entity, geometry, layer, color_override) in added {
            self.add_geometry(entity, geometry, layer, color_override);
        }
    }

    fn add_geometry(
        &mut self,
        entity: Entity,
        geometry: &Geometry,
        layer: Option<&Layer>,
        color_override: Option<&ColorOverride>,
    ) {
        let lines = match geometry {
            Geometry::Line(l) => vec![*l],
            Geometry::Rectangle(r) => vec![
                Line::new(r.p1, Point2D::new(r.p2.x, r.p1.y)),
                Line::new(Point2D::new(r.p2.x, r.p1.y), r.p2),
                Line::new(r.p2, Point2D::new(r.p1.x, r.p2.y)),
                Line::new(Point2D::new(r.p1.x, r.p2.y), r.p1),
            ],
            _ => vec![], // Otros tipos no soportados aún por el renderer 2D simple
        };

        if lines.is_empty() {
            return;
        }

        let mut current_entity_indices = Vec::new();

        for line in lines {
            let instance = self.convert_line(&line, layer, color_override);
            let index = if let Some(free_index) = self.free_slots.pop() {
                self.instances[free_index] = instance;
                free_index
            } else {
                let new_index = self.instances.len();
                self.instances.push(instance);
                new_index
            };

            current_entity_indices.push(index);
            self.dirty_ranges.push(index);
        }

        self.entity_to_index.insert(entity, current_entity_indices);
    }

    fn convert_line(
        &self,
        line: &Line,
        layer: Option<&Layer>,
        color_override: Option<&ColorOverride>,
    ) -> LineInstance {
        let mut final_color = [1.0, 1.0, 1.0, 1.0];

        if let Some(l) = layer {
            final_color = self.ncs_dict.get_color(&l.0);
        }

        if let Some(c) = color_override {
            final_color = fcad_core::infrastructure::ecs::ncs::hex_to_rgba(&c.0);
        }

        LineInstance {
            start: [line.start.x as f32, line.start.y as f32],
            end: [line.end.x as f32, line.end.y as f32],
            color: final_color,
            thickness: 1.0,
            _padding: [0.0; 3],
        }
    }

    /// Método simulado para escribir diferencialmente a la GPU (Buffer)
    pub fn write_to_vram(&mut self, queue: &wgpu::Queue, buffer: &wgpu::Buffer) {
        // En un motor AAA, aquí agruparías rangos contiguos en `dirty_ranges`
        // y ejecutarías queue.write_buffer() solo para esos subyacentes bloques de bytes.

        let size_of_instance = std::mem::size_of::<LineInstance>() as u64;

        for &index in &self.dirty_ranges {
            let offset = index as u64 * size_of_instance;
            let data = bytemuck::bytes_of(&self.instances[index]);
            queue.write_buffer(buffer, offset, data);
        }

        // Limpiamos los dirty luego de confirmar el vaciado VRAM
        self.dirty_ranges.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fcad_core::domain::math::primitives::Point2D;

    #[test]
    fn test_differential_sync_optimizer() {
        let mut world = World::new();
        let mut optimizer = RenderOptimizer::new(LayerStandards::new());

        // Inicializamos 10,000 líneas estáticas simuladas
        for i in 0..10_000 {
            world.spawn(Geometry::Line(Line {
                start: Point2D {
                    x: i as f64,
                    y: 0.0,
                },
                end: Point2D {
                    x: i as f64,
                    y: 10.0,
                },
            }));
        }

        // Insertamos 1 línea Dinámica
        let dynamic_entity = world
            .spawn(Geometry::Line(Line {
                start: Point2D { x: -10.0, y: -10.0 },
                end: Point2D { x: -10.0, y: -10.0 },
            }))
            .id();

        // Ejecutamos Sync 1 (Added) que capturará todo
        let mut added_query = world.query_filtered::<(Entity, &Geometry, Option<&Layer>, Option<&ColorOverride>), Added<Geometry>>();
        let mut changed_query = world.query_filtered::<(Entity, &Geometry, Option<&Layer>, Option<&ColorOverride>), Changed<Geometry>>();
        let mut deleted_query = world.query_filtered::<Entity, Added<Deleted>>();

        optimizer.sync_system(
            added_query.iter(&world),
            changed_query.iter(&world),
            deleted_query.iter(&world),
        );

        // Afirmar que 10,001 líneas se insertaron
        assert_eq!(optimizer.instances.len(), 10001);
        assert_eq!(optimizer.dirty_ranges.len(), 10001); // Se enviarán todas al inicio

        // Vaciamos "dirty" simulando haber escrito el VRAM
        optimizer.dirty_ranges.clear();
        world.clear_trackers(); // Reinicia el tracker Changed/Added de Bevy ECS

        // Mover solo la línea dinámica (Simular paso del tiempo)
        let mut dynamic_geom = world.get_mut::<Geometry>(dynamic_entity).unwrap();
        if let Geometry::Line(ref mut line) = *dynamic_geom {
            line.end.x = -30.0;
        }

        // Ejecutamos Sync 2 (Frame 2 - Diferencial)
        optimizer.sync_system(
            added_query.iter(&world),
            changed_query.iter(&world),
            deleted_query.iter(&world),
        );

        // Afirmamos optimización crítica! Solo debería haber 1 en el Dirty Range
        assert_eq!(
            optimizer.dirty_ranges.len(),
            1,
            "GPU Saturada! Se actualizó más de 1 línea"
        );

        // Verificamos el ruteo interno
        let updated_indices = &optimizer.entity_to_index[&dynamic_entity];
        assert_eq!(optimizer.instances[updated_indices[0]].end[0], -30.0);
    }
}
