use bevy_ecs::prelude::*;
use crate::domain::math::primitives::{Point2D, Line, Circle, Arc, Rectangle};
use serde::{Serialize, Deserialize};

/// Un enum que engloba cualquier tipo de geometría primitiva soportada por el ECS.
#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Geometry {
    Point(Point2D),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Rectangle(Rectangle),
}

/// Componente que define a qué capa semántica (NCS) pertenece la entidad.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Layer(pub String);

/// Componente opcional que define un color de renderizado explícito (hex o RGBA).
/// Si no está presente, el renderizador debería usar el color por defecto de la capa.
#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorOverride(pub String);

/// Componente de marcado temporal para el Patrón Comando (Tombstoning).
/// Las entidades con este componente están lógicamente borradas y no deben renderizarse
/// ni ser indexadas espacialmente.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Deleted;

/// Componente de marcado para entidades seleccionadas en el viewport.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Selected;

/// Recurso global para el estado del proyecto actual
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub file_name: String,
    pub discipline: Option<String>,
}

impl ProjectMetadata {
    pub fn new(file_name: &str) -> Self {
        let discipline = if file_name.starts_with("A-") {
            Some("Arquitectónico".to_string())
        } else if file_name.starts_with("S-") {
            Some("Estructural".to_string())
        } else if file_name == "Untitled.fcad" {
            None
        } else {
            Some("Genérico".to_string())
        };

        Self {
            file_name: file_name.to_string(),
            discipline,
        }
    }
}
