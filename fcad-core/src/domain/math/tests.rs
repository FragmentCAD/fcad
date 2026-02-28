#[cfg(test)]
mod tests {
    use crate::domain::math::primitives::{Point2D, Line, Circle};

    #[test]
    fn test_point_distance() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        assert_eq!(p1.distance_to(&p2), 5.0);
    }

    #[test]
    fn test_line_length() {
        let line = Line::new(Point2D::new(-1.0, -1.0), Point2D::new(2.0, 3.0));
        assert_eq!(line.length(), 5.0);
    }

    #[test]
    fn test_circle_area() {
        let circle = Circle::new(Point2D::new(0.0, 0.0), 10.0);
        let expected_area = std::f64::consts::PI * 100.0;
        assert_eq!(circle.area(), expected_area);
    }
}
