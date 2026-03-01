use bevy_ecs::prelude::*;
use fcad_core::domain::{Geometry, Layer, ColorOverride};
use fcad_core::domain::math::primitives::Line;
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
    /// Mapeo de Entidad ECS a posición en el Array contiguo
    pub entity_to_index: HashMap<Entity, usize>,
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
        added: impl IntoIterator<Item = (Entity, &'a Geometry, Option<&'a Layer>, Option<&'a ColorOverride>)>,
        changed: impl IntoIterator<Item = (Entity, &'a Geometry, Option<&'a Layer>, Option<&'a ColorOverride>)>,
        removed: impl IntoIterator<Item = Entity>,
    ) {
        self.dirty_ranges.clear();

        // 1. Procesar Eliminaciones (Tombstoning)
        for entity in removed {
            if let Some(index) = self.entity_to_index.remove(&entity) {
                // Para re-utilizar la ranura y no compactar todo el arreglo (evita writes masivos a VRAM)
                // Ocultamos la geometría desplazándola y haciéndola transparente o marcando su grosor 0
                self.instances[index].thickness = 0.0;
                self.free_slots.push(index);
                self.dirty_ranges.push(index);
            }
        }

        // 2. Procesar Modificaciones
        for (entity, geometry, layer, color_override) in changed {
            if let Some(&index) = self.entity_to_index.get(&entity) {
                if let Geometry::Line(line) = geometry {
                    self.instances[index] = self.convert_line(&line, layer, color_override);
                    self.dirty_ranges.push(index);
                } else {
                    // TODO: Implement support for other geometry types in the renderer
                }
            }
        }

        // 3. Procesar Adiciones
        for (entity, geometry, layer, color_override) in added {
            if let Geometry::Line(line) = geometry {
                let instance = self.convert_line(&line, layer, color_override);
                let index = if let Some(free_index) = self.free_slots.pop() {
                    self.instances[free_index] = instance;
                    free_index
                } else {
                    let new_index = self.instances.len();
                    self.instances.push(instance);
                    new_index
                };
                
                self.entity_to_index.insert(entity, index);
                self.dirty_ranges.push(index);
            } else {
                // TODO: Implement support for other geometry types in the renderer
            }
        }
    }

    fn convert_line(&self, line: &Line, layer: Option<&Layer>, color_override: Option<&ColorOverride>) -> LineInstance {
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
                start: Point2D { x: i as f64, y: 0.0 },
                end: Point2D { x: i as f64, y: 10.0 },
            }));
        }

        // Insertamos 1 línea Dinámica
        let dynamic_entity = world.spawn(Geometry::Line(Line {
            start: Point2D { x: -10.0, y: -10.0 },
            end: Point2D { x: -10.0, y: -10.0 },
        })).id();
        
        // Ejecutamos Sync 1 (Added) que capturará todo
        let mut added_query = world.query_filtered::<(Entity, &Geometry, Option<&Layer>, Option<&ColorOverride>), Added<Geometry>>();
        let mut changed_query = world.query_filtered::<(Entity, &Geometry, Option<&Layer>, Option<&ColorOverride>), Changed<Geometry>>();
        let mut deleted_query = world.query_filtered::<Entity, Added<Deleted>>();

        optimizer.sync_system(
            added_query.iter(&world),
            changed_query.iter(&world),
            deleted_query.iter(&world)
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
            deleted_query.iter(&world)
        );

        // Afirmamos optimización crítica! Solo debería haber 1 en el Dirty Range
        assert_eq!(optimizer.dirty_ranges.len(), 1, "GPU Saturada! Se actualizó más de 1 línea");
        
        // Verificamos el ruteo interno
        let updated_index = optimizer.entity_to_index[&dynamic_entity];
        assert_eq!(optimizer.instances[updated_index].end[0], -30.0);
    }
}
