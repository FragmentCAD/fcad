use dxf::Drawing;
use dxf::entities::{EntityType, Entity};
use bevy_ecs::prelude::*;
use crate::domain::{Geometry, Layer, Deleted};
use crate::domain::math::primitives::{Point2D, Line, Circle, Arc as DxfArc};


/// Carga un archivo DXF y extrae únicamente la geometría 2D relevante
/// (LINE, CIRCLE, ARC, LWPOLYLINE, INSERT) hacia el ECS, descartando el resto.
pub fn import_dxf(path: &str, world: &mut World) -> Result<(), dxf::DxfError> {
    let drawing = Drawing::load_file(path)?;
    
    for entity in drawing.entities() {
        parse_entity(entity, &drawing, world, 0.0, 0.0);
    }
    
    Ok(())
}

fn parse_entity(entity: &Entity, drawing: &Drawing, world: &mut World, offset_x: f64, offset_y: f64) {
    let layer_name = entity.common.layer.clone();
    let layer = Layer(layer_name);

    match &entity.specific {
        EntityType::Line(l) => {
            let start = Point2D::new(l.p1.x + offset_x, l.p1.y + offset_y);
            let end = Point2D::new(l.p2.x + offset_x, l.p2.y + offset_y);
            world.spawn((Geometry::Line(Line::new(start, end)), layer));
        },
        EntityType::Circle(c) => {
            let center = Point2D::new(c.center.x + offset_x, c.center.y + offset_y);
            world.spawn((Geometry::Circle(Circle::new(center, c.radius)), layer));
        },
        EntityType::Arc(a) => {
            let center = Point2D::new(a.center.x + offset_x, a.center.y + offset_y);
            let start_angle = a.start_angle.to_radians();
            let end_angle = a.end_angle.to_radians();
            world.spawn((Geometry::Arc(DxfArc::new(center, a.radius, start_angle, end_angle)), layer));
        },
        EntityType::LwPolyline(poly) => {
            // Descompone la polilínea en líneas individuales para simplificar el modelo base
            if poly.vertices.len() < 2 { return; }
            for i in 0..poly.vertices.len() - 1 {
                let p1 = &poly.vertices[i];
                let p2 = &poly.vertices[i+1];
                let start = Point2D::new(p1.x + offset_x, p1.y + offset_y);
                let end = Point2D::new(p2.x + offset_x, p2.y + offset_y);
                world.spawn((Geometry::Line(Line::new(start, end)), layer.clone()));
            }
            if (poly.flags & 1) != 0 {
                let first = &poly.vertices[0];
                let last = &poly.vertices[poly.vertices.len() - 1];
                let start = Point2D::new(last.x + offset_x, last.y + offset_y);
                let end = Point2D::new(first.x + offset_x, first.y + offset_y);
                world.spawn((Geometry::Line(Line::new(start, end)), layer));
            }
        },
        EntityType::Insert(insert) => {
            // Resuelve y despliega bloques referenciados
            let block_name = &insert.name;
            for block in drawing.blocks() {
                if block.name == *block_name {
                    let insert_offset_x = offset_x + insert.location.x;
                    let insert_offset_y = offset_y + insert.location.y;
                    for block_entity in &block.entities {
                        parse_entity(block_entity, drawing, world, insert_offset_x, insert_offset_y);
                    }
                    break;
                }
            }
        },
        // Silenciosamente descarta 3DFaces, Splines y otras mallas avanzadas
        _ => {}
    }
}

/// Recorre el ECS activo y exporta la geometría relevante de vuelta a un archivo DXF estándar,
/// manteniendo la organización por capas y filtrando entidades lógicamente eliminadas.
pub fn export_dxf(path: &str, world: &mut World) -> Result<(), dxf::DxfError> {
    let mut drawing = Drawing::new();
    
    let mut query = world.query_filtered::<(&Geometry, &Layer), Without<Deleted>>();
    
    for (geom, layer) in query.iter(world) {
        let mut entity = match geom {
            Geometry::Point(_) => {
                continue;
            },
            Geometry::Line(l) => {
                let mut line = dxf::entities::Line::default();
                line.p1 = dxf::Point::new(l.start.x, l.start.y, 0.0);
                line.p2 = dxf::Point::new(l.end.x, l.end.y, 0.0);
                Entity::new(EntityType::Line(line))
            },
            Geometry::Circle(c) => {
                let mut circle = dxf::entities::Circle::default();
                circle.center = dxf::Point::new(c.center.x, c.center.y, 0.0);
                circle.radius = c.radius;
                Entity::new(EntityType::Circle(circle))
            },
            Geometry::Arc(a) => {
                let mut arc = dxf::entities::Arc::default();
                arc.center = dxf::Point::new(a.center.x, a.center.y, 0.0);
                arc.radius = a.radius;
                arc.start_angle = a.start_angle.to_degrees();
                arc.end_angle = a.end_angle.to_degrees();
                Entity::new(EntityType::Arc(arc))
            },
            Geometry::Rectangle(r) => {
                let mut poly = dxf::entities::LwPolyline::default();
                poly.vertices = vec![
                    dxf::LwPolylineVertex { x: r.p1.x, y: r.p1.y, ..Default::default() },
                    dxf::LwPolylineVertex { x: r.p2.x, y: r.p1.y, ..Default::default() },
                    dxf::LwPolylineVertex { x: r.p2.x, y: r.p2.y, ..Default::default() },
                    dxf::LwPolylineVertex { x: r.p1.x, y: r.p2.y, ..Default::default() },
                ];
                poly.flags = 1; // Closed
                Entity::new(EntityType::LwPolyline(poly))
            },
        };
        
        entity.common.layer = layer.0.clone();
        drawing.add_entity(entity);
    }
    
    drawing.save_file(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_dxf_fuzzing_filter() {
        // En lugar de requerir un archivo de disco externo (que podría no existir en CI),
        // fabricamos un Drawing de dxf con tipos mixtos (basura 3D y líneas válidas)
        // y lo serializamos a un temporal.
        let mut drawing = Drawing::new();
        
        // 1. Añadimos algo de basura 3D (Face3D)
        let mut face = dxf::entities::Face3D::default();
        face.first_corner = dxf::Point::new(0.0, 0.0, 10.0); // Z != 0
        let face_entity = Entity::new(EntityType::Face3D(face));
        drawing.add_entity(face_entity);
        
        // 2. Añadimos una línea válida 2D
        let mut line = dxf::entities::Line::default();
        line.p1 = dxf::Point::new(0.0, 0.0, 0.0);
        line.p2 = dxf::Point::new(10.0, 0.0, 0.0);
        let mut line_entity = Entity::new(EntityType::Line(line));
        line_entity.common.layer = "A-WALL".to_string();
        drawing.add_entity(line_entity);
        
        let path = "test_fuzz.dxf";
        drawing.save_file(path).unwrap();

        // Verificamos que se parsee sin panicking y extraiga solo 1
        let mut world = World::new();
        import_dxf(path, &mut world).unwrap();
        
        let geoms_count = world.query::<&Geometry>().iter(&world).count();
        assert_eq!(geoms_count, 1, "Debería haber ignorado la malla 3D y rescatado únicamente la línea 2D");
        
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_dxf_bidirectionality_export() {
        let path = "test_bidirectional.dxf";
        
        // 1. Arrange: Creamos un mundo poblado
        let mut world1 = World::new();
        world1.spawn((Geometry::Line(Line::new(Point2D::new(0.0, 0.0), Point2D::new(5.0, 5.0))), Layer("A-WALL".to_string())));
        world1.spawn((Geometry::Circle(Circle::new(Point2D::new(10.0, 10.0), 3.0)), Layer("A-DOOR".to_string())));
        
        // Añadimos otro que luego es "Deleted" (y que debe ser ignorado en e port)
        let deleted_id = world1.spawn((Geometry::Line(Line::new(Point2D::new(0.0, 0.0), Point2D::new(1.0, 1.0))), Layer("A-TRASH".to_string()))).id();
        world1.entity_mut(deleted_id).insert(Deleted);
        
        // 2. Act: Exportamos a DXF
        export_dxf(path, &mut world1).expect("Failed to export DXF");
        
        // 3. Act de retorno: Importamos el DXF exportado a un mundo COMPLETAMENTE NUEVO
        let mut world2 = World::new();
        import_dxf(path, &mut world2).expect("Failed to import DXF");
        
        // 4. Assert: El nuevo mundo debe tener exactamente 2 entidades
        let count_active_w1 = world1.query_filtered::<&Geometry, Without<Deleted>>().iter(&world1).count();
        let count_active_w2 = world2.query_filtered::<&Geometry, Without<Deleted>>().iter(&world2).count();
        
        assert_eq!(count_active_w1, 2, "El mundo 1 debía tener 2 componentes activos");
        assert_eq!(count_active_w2, 2, "El mundo 2 debía reconstruir exactamente los 2 mismos componentes");
        assert_eq!(count_active_w1, count_active_w2, "La asimetría de componentes quebró la presunción de bidireccionalidad");

        fs::remove_file(path).unwrap();
    }
}
