use glam::{Mat4, Vec2, Vec3, Vec4};

/// Estructura que maneja la cámara 2D (Pan & Zoom) y la conversión de coordenadas espaciales.
#[derive(Debug, Clone, bevy_ecs::system::Resource)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            screen_width: 800.0,
            screen_height: 600.0,
        }
    }
}

impl Camera {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            screen_width: width,
            screen_height: height,
        }
    }

    pub fn build_projection_matrix(&self) -> Mat4 {
        let half_width = self.screen_width / 2.0 / self.zoom;
        let half_height = self.screen_height / 2.0 / self.zoom;
        
        Mat4::orthographic_rh(
            -half_width, 
            half_width, 
            -half_height, 
            half_height, 
            -1.0,
            1.0,
        )
    }

    pub fn build_view_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, 0.0))
    }

    pub fn build_view_projection_matrix(&self) -> Mat4 {
        self.build_projection_matrix() * self.build_view_matrix()
    }

    pub fn unproject(&self, screen_x: f32, screen_y: f32) -> Vec2 {
        let ndc_x = (screen_x / self.screen_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / self.screen_height) * 2.0; 
        
        let ndc_point = Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
        
        let vp_matrix = self.build_view_projection_matrix();
        let inverse_vp = vp_matrix.inverse();
        
        let world_point = inverse_vp * ndc_point;
        
        Vec2::new(world_point.x, world_point.y)
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.position.x -= dx / self.zoom;
        self.position.y += dy / self.zoom;
    }

    pub fn zoom_at(&mut self, factor: f32, screen_x: f32, screen_y: f32) {
        let world_before = self.unproject(screen_x, screen_y);

        self.zoom = (self.zoom * factor).clamp(0.001, 100_000.0);

        let world_after = self.unproject(screen_x, screen_y);

        self.position.x += world_before.x - world_after.x;
        self.position.y += world_before.y - world_after.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_unproject_center() {
        let mut camera = Camera::new(1024.0, 768.0);
        camera.zoom = 2.0;

        let mouse_x = 512.0;
        let mouse_y = 384.0;

        let world_coord = camera.unproject(mouse_x, mouse_y);

        assert_eq!(world_coord.x, 0.0, "La X del mundo en el centro de pantalla falló");
        assert_eq!(world_coord.y, 0.0, "La Y del mundo en el centro de pantalla falló");
    }
    
    #[test]
    fn test_camera_unproject_offset() {
        let mut camera = Camera::new(100.0, 100.0);
        camera.zoom = 1.0;
        camera.position = Vec2::new(10.0, 10.0);
        
        let center = camera.unproject(50.0, 50.0);
        assert!((center.x - 10.0).abs() < f32::EPSILON * 10.0);
        assert!((center.y - 10.0).abs() < f32::EPSILON * 10.0);
    }

    #[test]
    fn test_pan_moves_camera() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 1.0;

        // Pan 100 pixels a la derecha en pantalla -> cámara se mueve a la izquierda
        camera.pan(100.0, 0.0);
        assert!((camera.position.x - (-100.0)).abs() < 0.01,
            "Pan derecho debería mover la cámara X negativamente, got: {}", camera.position.x);
        assert!((camera.position.y - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_pan_scaled_by_zoom() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 2.0; // Zoom x2

        // Pan 100px con zoom x2 = 50 unidades de mundo
        camera.pan(100.0, 0.0);
        assert!((camera.position.x - (-50.0)).abs() < 0.01,
            "Pan con zoom 2x debería desplazar la mitad, got: {}", camera.position.x);
    }

    #[test]
    fn test_zoom_at_center_preserves_center() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 1.0;

        // Zoom al centro de la pantalla (400, 300)
        let world_before = camera.unproject(400.0, 300.0);
        camera.zoom_at(2.0, 400.0, 300.0);
        let world_after = camera.unproject(400.0, 300.0);

        assert!((world_before.x - world_after.x).abs() < 0.01,
            "El punto mundo bajo el centro debe permanecer fijo. Before: {}, After: {}", world_before.x, world_after.x);
        assert!((world_before.y - world_after.y).abs() < 0.01,
            "El punto mundo bajo el centro debe permanecer fijo. Before: {}, After: {}", world_before.y, world_after.y);
    }

    #[test]
    fn test_zoom_at_corner_preserves_corner() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 1.0;

        // Zoom en la esquina superior izquierda
        let world_before = camera.unproject(0.0, 0.0);
        camera.zoom_at(3.0, 0.0, 0.0);
        let world_after = camera.unproject(0.0, 0.0);

        assert!((world_before.x - world_after.x).abs() < 0.5,
            "Esquina mundo X debe permanecer estable. Before: {}, After: {}", world_before.x, world_after.x);
        assert!((world_before.y - world_after.y).abs() < 0.5,
            "Esquina mundo Y debe permanecer estable. Before: {}, After: {}", world_before.y, world_after.y);
    }

    #[test]
    fn test_zoom_limits() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 1.0;

        // Intentar zoom extremadamente pequeño
        camera.zoom_at(0.0001, 400.0, 300.0);
        assert!(camera.zoom >= 0.001, "El zoom no debe bajar de 0.001, got: {}", camera.zoom);

        // Intentar zoom extremadamente grande
        camera.zoom = 1.0;
        camera.zoom_at(999_999.0, 400.0, 300.0);
        assert!(camera.zoom <= 100_000.0, "El zoom no debe superar 100000, got: {}", camera.zoom);
    }

    #[test]
    fn test_pan_then_zoom_preserves_cursor_anchor() {
        let mut camera = Camera::new(800.0, 600.0);
        camera.zoom = 1.0;

        // Pan y luego zoom en un punto arbitrario
        camera.pan(200.0, -100.0);

        let cursor_x = 300.0;
        let cursor_y = 200.0;
        let world_before = camera.unproject(cursor_x, cursor_y);
        camera.zoom_at(1.5, cursor_x, cursor_y);
        let world_after = camera.unproject(cursor_x, cursor_y);

        assert!((world_before.x - world_after.x).abs() < 0.5,
            "Zoom después de pan debe anclar correctamente X. Before: {}, After: {}", world_before.x, world_after.x);
        assert!((world_before.y - world_after.y).abs() < 0.5,
            "Zoom después de pan debe anclar correctamente Y. Before: {}, After: {}", world_before.y, world_after.y);
    }
}
