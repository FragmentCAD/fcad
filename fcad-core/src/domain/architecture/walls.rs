use crate::domain::math::primitives::{Line, Point2D};
use crate::domain::{Geometry, Layer};
use bevy_ecs::prelude::*;

/// Genera y añade al `World` (ECS) un muro paramétrico representado por sus 4 líneas perimetrales.
/// Se usa una aproximación de geometría 2D basada en normales matemáticas puras,
/// asegurando independencia visual y compatibilidad nativa con cad/dxf.
pub fn generate_wall(
    world: &mut World,
    p1: Point2D,
    p2: Point2D,
    thickness: f64,
    layer_name: &str,
) {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let length = (dx * dx + dy * dy).sqrt();

    if length == 0.0 {
        return; // Manejo de caso borde: Si es un punto en el espacio, no genera geometría.
    }

    // Vector unitario director
    let ux = dx / length;
    let uy = dy / length;

    // Vector normal (perpendicular)
    let nx = -uy;
    let ny = ux;

    // Mitad de grosor hacia cada lado del eje central (p1 -> p2)
    let half_t = thickness / 2.0;

    // Computar esquinas matemáticamente:
    // c1 ------ c2   (Borde Superior +Normal)
    // |          |
    // p1 ------ p2   (Línea Eje abstracta)
    // |          |
    // c4 ------ c3   (Borde Inferior -Normal)

    let c1 = Point2D::new(p1.x + nx * half_t, p1.y + ny * half_t);
    let c2 = Point2D::new(p2.x + nx * half_t, p2.y + ny * half_t);
    let c3 = Point2D::new(p2.x - nx * half_t, p2.y - ny * half_t);
    let c4 = Point2D::new(p1.x - nx * half_t, p1.y - ny * half_t);

    let capa = Layer(layer_name.to_string());

    // Insertar el polígono descompuesto en 4 líneas perimetrales CAD-puras
    world.spawn((Geometry::Line(Line::new(c1, c2)), capa.clone()));
    world.spawn((Geometry::Line(Line::new(c2, c3)), capa.clone()));
    world.spawn((Geometry::Line(Line::new(c3, c4)), capa.clone()));
    world.spawn((Geometry::Line(Line::new(c4, c1)), capa));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Geometry;

    #[test]
    fn test_generate_parametric_wall() {
        let mut world = World::new();

        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(10.0, 0.0); // Eje Horizontal
        let thickness = 2.0;

        // Ejecutamos la función base del generador del framework
        generate_wall(&mut world, p1, p2, thickness, "A-WALL");

        // Evaluamos TDD en el ECS
        let query = world.query::<&Geometry>().iter(&world).collect::<Vec<_>>();

        assert_eq!(
            query.len(),
            4,
            "Un muro rectangular debe resolverse en 4 líneas puras"
        );

        // Sabiendo que nx = 0, ny = 1, el top-left es (0, 1), el bottom-let es (0, -1)
        for geom in query {
            if let Geometry::Line(line) = geom {
                // Afirmamos matemáticamente que ninguna de las 4 líneas
                // superó el espacio abstracto definido
                assert!(line.start.y <= 1.0 && line.start.y >= -1.0);
                assert!(line.end.y <= 1.0 && line.end.y >= -1.0);
                assert!(line.start.x <= 10.0 && line.start.x >= 0.0);
                assert!(line.end.x <= 10.0 && line.end.x >= 0.0);
            } else {
                panic!("Solo devió generarse lineas para el primitivo de muro");
            }
        }
    }
}
