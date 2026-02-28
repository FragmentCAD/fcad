use bevy_ecs::entity::Entity;
use crate::infrastructure::ecs::spatial::SpatialIndex;
use crate::infrastructure::ecs::components::Geometry;
use serde::{Serialize, Deserialize};

/// Resultado de una operación de snap.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SnapType {
    None,
    Grid,
    Ortho,
    Endpoint,
    Midpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapResult {
    pub point: [f32; 2],
    pub snap_type: SnapType,
}

/// Trait para abstraer cómo obtenemos la geometría de una entidad.
/// Permite que el SnapEngine sea testeable sin un World de Bevy real.
pub trait GeometryProvider {
    fn get_geometry(&self, entity: Entity) -> Option<Geometry>;
}

/// Implementación de GeometryProvider que utiliza un World de Bevy.
pub struct WorldGeometryProvider<'a> {
    pub world: &'a bevy_ecs::world::World,
}

impl<'a> GeometryProvider for WorldGeometryProvider<'a> {
    fn get_geometry(&self, entity: Entity) -> Option<Geometry> {
        self.world.get::<Geometry>(entity).cloned()
    }
}

/// Estado de las restricciones de precisión del motor de snap.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SnapState {
    pub ortho_enabled: bool,
    pub osnaps_enabled: bool,
    pub grid_snap_enabled: bool,
    pub grid_size: f32,
}

impl Default for SnapState {
    fn default() -> Self {
        Self {
            ortho_enabled: true,
            osnaps_enabled: true,
            grid_snap_enabled: false,
            grid_size: 1.0,
        }
    }
}

/// Motor de precisión que intercepta coordenadas y las ajusta según restricciones.
pub struct SnapEngine {
    pub state: SnapState,
    pub snap_radius_px: f32,
}

impl SnapEngine {
    pub fn new() -> Self {
        Self {
            state: SnapState::default(),
            snap_radius_px: 12.0, // Radio de magnetismo en píxeles de pantalla (aprox)
        }
    }

    /// Toma una coordenada "cruda" y aplica las restricciones activas.
    pub fn snap_coordinate(
        &self,
        raw_x: f32,
        raw_y: f32,
        last_point: Option<[f32; 2]>,
        spatial_index: &SpatialIndex,
        provider: &impl GeometryProvider,
        zoom_level: f32,
    ) -> SnapResult {
        let mut result = SnapResult {
            point: [raw_x, raw_y],
            snap_type: SnapType::None,
        };

        // 1. Osnaps (Endpoints) - TIENEN PRIORIDAD MÁXIMA
        if self.state.osnaps_enabled {
            // Convertimos el radio de píxeles a unidades del mundo
            let world_threshold = self.snap_radius_px / zoom_level;
            
            let candidates = spatial_index.query_area(
                (raw_x - world_threshold) as f64,
                (raw_y - world_threshold) as f64,
                (raw_x + world_threshold) as f64,
                (raw_y + world_threshold) as f64,
            );

            let mut best_dist = world_threshold;
            let mut best_point = None;

            for entity in candidates {
                if let Some(geometry) = provider.get_geometry(entity) {
                    let points = match geometry {
                        Geometry::Line(l) => {
                            let end_pts = vec![[l.start.x as f32, l.start.y as f32], [l.end.x as f32, l.end.y as f32]];
                            let mid_pt = [(l.start.x as f32 + l.end.x as f32) / 2.0, (l.start.y as f32 + l.end.y as f32) / 2.0];
                            
                            // Primero buscamos en endpoints (prioridad mayor)
                            let mut pts = vec![(end_pts[0], SnapType::Endpoint), (end_pts[1], SnapType::Endpoint)];
                            pts.push((mid_pt, SnapType::Midpoint));
                            pts
                        },
                        Geometry::Point(p) => vec![([p.x as f32, p.y as f32], SnapType::Endpoint)],
                        _ => vec![],
                    };

                    for (p, stype) in points {
                        let dx = p[0] - raw_x;
                        let dy = p[1] - raw_y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < best_dist {
                            best_dist = dist;
                            best_point = Some((p, stype));
                        }
                    }
                }
            }

            if let Some((p, stype)) = best_point {
                result.point = p;
                result.snap_type = stype;
                return result; 
            }
        }

        // 2. Restricción Ortho / Polar (90° y 45°)
        if self.state.ortho_enabled {
            if let Some(origin) = last_point {
                let dx = raw_x - origin[0];
                let dy = raw_y - origin[1];
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 1e-4 {
                    let angle_rad = dy.atan2(dx);
                    let angle_deg = angle_rad.to_degrees().abs();
                    
                    // Normalizamos a [0, 90] para simplificar la lógica de "cercanía"
                    let norm_angle = angle_deg % 90.0;
                    
                    // Definimos umbrales para 90° y 45° (ej. 15 grados de tolerancia)
                    if norm_angle < 15.0 || norm_angle > 75.0 {
                        // Snap a 90° (H o V)
                        if dx.abs() > dy.abs() {
                            result.point[1] = origin[1];
                        } else {
                            result.point[0] = origin[0];
                        }
                        result.snap_type = SnapType::Ortho;
                    } else if (norm_angle - 45.0).abs() < 15.0 {
                        // Snap a 45°
                        let sign_x = if dx >= 0.0 { 1.0 } else { -1.0 };
                        let sign_y = if dy >= 0.0 { 1.0 } else { -1.0 };
                        let avg = (dx.abs() + dy.abs()) / 2.0;
                        result.point[0] = origin[0] + avg * sign_x;
                        result.point[1] = origin[1] + avg * sign_y;
                        result.snap_type = SnapType::Ortho; // Podríamos diferenciarlo como SnapType::Polar
                    }
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::math::primitives::Point2D;
    use crate::domain::math::primitives::Line as MathLine;
    use crate::infrastructure::ecs::spatial::calculate_aabb;
    use crate::infrastructure::ecs::spatial::SpatialEntity;

    struct MockProvider {
        geometries: std::collections::HashMap<Entity, Geometry>,
    }

    impl GeometryProvider for MockProvider {
        fn get_geometry(&self, entity: Entity) -> Option<Geometry> {
            self.geometries.get(&entity).cloned()
        }
    }

    #[test]
    fn test_ortho_horizontal_snap() {
        let engine = SnapEngine::new();
        let spatial_index = SpatialIndex::new();
        let provider = MockProvider { geometries: std::collections::HashMap::new() };
        
        let origin = [0.0, 0.0];
        let raw = [10.0, 2.0];
        
        let result = engine.snap_coordinate(raw[0], raw[1], Some(origin), &spatial_index, &provider, 1.0);
        
        assert_eq!(result.point[0], 10.0);
        assert_eq!(result.point[1], 0.0);
        assert_eq!(result.snap_type, SnapType::Ortho);
    }

    #[test]
    fn test_osnap_endpoint_priority() {
        let engine = SnapEngine::new();
        let mut spatial_index = SpatialIndex::new();
        let mut geometries = std::collections::HashMap::new();

        // Creamos una línea de (20, 20) a (30, 30)
        let entity = Entity::from_raw(1);
        let geom = Geometry::Line(MathLine::new(Point2D::new(20.0, 20.0), Point2D::new(30.0, 30.0)));
        let envelope = calculate_aabb(&geom);
        spatial_index.tree.insert(SpatialEntity { id: entity, envelope });
        geometries.insert(entity, geom);

        let provider = MockProvider { geometries };
        
        let origin = [0.0, 0.0];
        // Cursor cerca del endpoint (20, 20), pero también en un ángulo que activaría Ortho
        let raw = [20.5, 19.5];
        
        let result = engine.snap_coordinate(raw[0], raw[1], Some(origin), &spatial_index, &provider, 1.0);
        
        // Debe ganar el Endpoint (20, 20) sobre el Ortho (que forzaría 20.5, 0.0 o 0.0, 19.5)
        assert_eq!(result.point[0], 20.0);
        assert_eq!(result.point[1], 20.0);
        assert_eq!(result.snap_type, SnapType::Endpoint);
    }

    #[test]
    fn test_osnap_midpoint_snap() {
        let engine = SnapEngine::new();
        let mut spatial_index = SpatialIndex::new();
        let mut geometries = std::collections::HashMap::new();

        // Línea de (0, 0) a (10, 0). Midpoint es (5, 0).
        let entity = Entity::from_raw(1);
        let geom = Geometry::Line(MathLine::new(Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0)));
        let envelope = calculate_aabb(&geom);
        spatial_index.tree.insert(SpatialEntity { id: entity, envelope });
        geometries.insert(entity, geom);

        let provider = MockProvider { geometries };
        
        let raw = [5.2, 0.3];
        let result = engine.snap_coordinate(raw[0], raw[1], None, &spatial_index, &provider, 1.0);
        
        assert_eq!(result.point[0], 5.0);
        assert_eq!(result.point[1], 0.0);
        assert_eq!(result.snap_type, SnapType::Midpoint);
    }

    #[test]
    fn test_polar_45_snap() {
        let engine = SnapEngine::new();
        let spatial_index = SpatialIndex::new();
        let provider = MockProvider { geometries: std::collections::HashMap::new() };
        
        let origin = [0.0, 0.0];
        // Cursor cerca de 45°: (10, 9)
        let raw = [10.0, 9.0];
        
        let result = engine.snap_coordinate(raw[0], raw[1], Some(origin), &spatial_index, &provider, 1.0);
        
        // Debe promediar dx y dy para quedar en la diagonal: (9.5, 9.5)
        assert_eq!(result.point[0], 9.5);
        assert_eq!(result.point[1], 9.5);
        assert_eq!(result.snap_type, SnapType::Ortho);
    }
}
