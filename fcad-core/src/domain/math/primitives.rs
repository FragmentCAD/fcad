use serde::{Deserialize, Serialize};

/// Un punto en el espacio 2D de doble precisión (f64)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2D {
    pub x: f64,
    pub y: f64,
}

impl Point2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Calcula la distancia euclidiana hacia otro punto
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Una línea recta definida por dos puntos
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Line {
    pub start: Point2D,
    pub end: Point2D,
}

impl Line {
    pub fn new(start: Point2D, end: Point2D) -> Self {
        Self { start, end }
    }

    /// Retorna la longitud exacta de la línea
    pub fn length(&self) -> f64 {
        self.start.distance_to(&self.end)
    }
}

/// Un círculo definido por su centro y radio
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2D,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2D, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Retorna el área del círculo
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

/// Un arco de círculo definido en radianes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point2D,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

impl Arc {
    pub fn new(center: Point2D, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Self { center, radius, start_angle, end_angle }
    }
}
