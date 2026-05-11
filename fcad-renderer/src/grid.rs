use super::Vertex;
use fcad_core::domain::viewport::Camera;

/// Genera los vértices para una grilla "infinita" basada en la vista actual de la cámara.
pub fn generate_grid_vertices(camera: &Camera, grid_spacing: f32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 1. Calcular los límites del mundo visibles en el viewport actual.
    // Usamos un pequeño margen (padding) para evitar que las líneas desaparezcan bruscamente en los bordes.
    let top_left = camera.unproject(0.0, 0.0);
    let bottom_right = camera.unproject(camera.screen_width, camera.screen_height);

    let min_x = top_left.x.min(bottom_right.x);
    let max_x = top_left.x.max(bottom_right.x);
    let min_y = top_left.y.min(bottom_right.y);
    let max_y = top_left.y.max(bottom_right.y);

    // 2. Determinar el espaciado dinámico basado en el zoom.
    // Si el zoom es muy pequeño, mostramos grillas más grandes (10, 100, etc.)
    // Si el zoom es muy grande, mostramos grillas más pequeñas (0.1, 0.01)
    let mut effective_spacing = grid_spacing;
    while effective_spacing * camera.zoom < 10.0 {
        effective_spacing *= 10.0;
    }
    while effective_spacing * camera.zoom > 100.0 {
        effective_spacing /= 10.0;
    }

    // 3. Generar líneas verticales (X constante)
    let mut x = (min_x / effective_spacing).floor() * effective_spacing;
    while x <= max_x {
        let alpha = if (x / (effective_spacing * 10.0)).abs() < 1e-4 {
            0.4
        } else {
            0.1
        };
        let color = [0.5, 0.5, 0.5, alpha]; // Color de grilla tenue

        let start_idx = vertices.len() as u16;
        vertices.push(Vertex {
            position: [x, min_y, 0.0],
            color,
        });
        vertices.push(Vertex {
            position: [x, max_y, 0.0],
            color,
        });

        indices.push(start_idx);
        indices.push(start_idx + 1);

        x += effective_spacing;
    }

    // 4. Generar líneas horizontales (Y constante)
    let mut y = (min_y / effective_spacing).floor() * effective_spacing;
    while y <= max_y {
        let alpha = if (y / (effective_spacing * 10.0)).abs() < 1e-4 {
            0.4
        } else {
            0.1
        };
        let color = [0.5, 0.5, 0.5, alpha];

        let start_idx = vertices.len() as u16;
        vertices.push(Vertex {
            position: [min_x, y, 0.0],
            color,
        });
        vertices.push(Vertex {
            position: [max_x, y, 0.0],
            color,
        });

        indices.push(start_idx);
        indices.push(start_idx + 1);

        y += effective_spacing;
    }

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_generation_produces_vertices() {
        let camera = Camera::new(800.0, 600.0);
        let (vertices, indices) = generate_grid_vertices(&camera, 1.0);

        assert!(!vertices.is_empty(), "La grilla debería tener vértices");
        assert!(!indices.is_empty(), "La grilla debería tener índices");
        // En una pantalla de 800x600 con espaciado 10, esperaríamos aprox 80 + 60 líneas (x2 vértices)
        assert!(
            vertices.len() > 10,
            "Debería haber un número razonable de vértices"
        );
    }

    #[test]
    fn test_grid_spacing_scales_with_zoom() {
        let mut camera = Camera::new(800.0, 600.0);

        // Zoom normal (1.0) -> Espaciado 1.0 -> 10.0 (según el loop while)
        let (_, indices_norm) = generate_grid_vertices(&camera, 1.0);

        // Zoom out extremo (0.01) -> Debería mostrar menos líneas (grilla más grande)
        camera.zoom = 0.01;
        let (_, indices_out) = generate_grid_vertices(&camera, 1.0);

        assert!(
            indices_out.len() <= indices_norm.len() * 2,
            "La grilla no debería saturarse de líneas al alejar el zoom"
        );
    }
}
