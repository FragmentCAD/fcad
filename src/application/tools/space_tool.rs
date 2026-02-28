use super::{Tool, ToolResponse, ToolResult};
use crate::application::input::{InputEvent, MouseButton};

/// Estado interno de la SpaceTool.
#[derive(Debug, Clone, PartialEq)]
enum SpaceToolState {
    /// Esperando el primer clic para empezar.
    WaitingForStart,
    /// Dibujando: acumulando vértices del polígono.
    Drawing { vertices: Vec<[f32; 2]> },
}

/// SpaceTool: Herramienta semántica para demarcar recintos arquitectónicos.
///
/// El usuario coloca puntos sucesivos que forman un polígono cerrado.
/// Al cerrar el polígono (clic derecho o doble clic cerca del primer punto),
/// se genera un `ToolResult::Space` con los vértices y un tipo semántico.
pub struct SpaceTool {
    state: SpaceToolState,
    /// Posición actual del cursor (para rubber-banding).
    current_mouse: [f32; 2],
    /// Distancia umbral (en unidades de mundo) para "cerrar" el polígono al hacer clic cerca del primer punto.
    close_threshold: f32,
}

impl SpaceTool {
    pub fn new() -> Self {
        Self {
            state: SpaceToolState::WaitingForStart,
            current_mouse: [0.0, 0.0],
            close_threshold: 5.0,
        }
    }

    /// Comprueba si el punto `p` está cerca del primer vértice de una lista de vértices.
    fn is_near_first(vertices: &[[f32; 2]], p: [f32; 2], threshold: f32) -> bool {
        if let Some(first) = vertices.first() {
            let dx = p[0] - first[0];
            let dy = p[1] - first[1];
            return (dx * dx + dy * dy).sqrt() < threshold;
        }
        false
    }

    /// Genera las líneas temporales (rubber-banding) del polígono en construcción.
    fn build_ephemeral_lines(&self) -> Vec<([f32; 2], [f32; 2])> {
        if let SpaceToolState::Drawing { ref vertices } = self.state {
            let mut lines = Vec::new();
            // Líneas entre vértices consecutivos
            for window in vertices.windows(2) {
                lines.push((window[0], window[1]));
            }
            // Línea desde el último vértice hasta el cursor actual
            if let Some(last) = vertices.last() {
                lines.push((*last, self.current_mouse));
            }
            lines
        } else {
            Vec::new()
        }
    }
}

impl Tool for SpaceTool {
    fn name(&self) -> &str {
        "space"
    }

    fn on_start(&mut self) {
        self.state = SpaceToolState::WaitingForStart;
        self.current_mouse = [0.0, 0.0];
    }

    fn on_input(&mut self, event: &InputEvent) -> ToolResponse {
        match event {
            InputEvent::PointerMove { x, y } => {
                self.current_mouse = [*x, *y];
                if matches!(self.state, SpaceToolState::Drawing { .. }) {
                    ToolResponse::EphemeralLines(self.build_ephemeral_lines())
                } else {
                    ToolResponse::Ignored
                }
            }
            InputEvent::Click {
                button: MouseButton::Left,
                x,
                y,
            } => {
                let point = [*x, *y];
                match &mut self.state {
                    SpaceToolState::WaitingForStart => {
                        self.state = SpaceToolState::Drawing {
                            vertices: vec![point],
                        };
                        ToolResponse::Consumed
                    }
                    SpaceToolState::Drawing { vertices } => {
                        // Si el clic está cerca del primer punto y hay al menos 3 vértices, cerrar
                        if vertices.len() >= 3 && Self::is_near_first(vertices, point, self.close_threshold) {
                            let result = ToolResult::Space {
                                vertices: vertices.clone(),
                                space_kind: "Unspecified".to_string(),
                            };
                            self.state = SpaceToolState::WaitingForStart;
                            ToolResponse::Completed(result)
                        } else {
                            vertices.push(point);
                            ToolResponse::Consumed
                        }
                    }
                }
            }
            InputEvent::Click {
                button: MouseButton::Right,
                ..
            } => {
                // Clic derecho cierra el polígono si hay al menos 3 vértices
                if let SpaceToolState::Drawing { ref vertices } = self.state {
                    if vertices.len() >= 3 {
                        let result = ToolResult::Space {
                            vertices: vertices.clone(),
                            space_kind: "Unspecified".to_string(),
                        };
                        self.state = SpaceToolState::WaitingForStart;
                        return ToolResponse::Completed(result);
                    }
                }
                ToolResponse::Ignored
            }
            InputEvent::KeyDown { key } if key == "Escape" => {
                self.on_cancel();
                ToolResponse::Consumed
            }
            _ => ToolResponse::Ignored,
        }
    }

    fn on_cancel(&mut self) {
        self.state = SpaceToolState::WaitingForStart;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_tool_starts_waiting() {
        let tool = SpaceTool::new();
        assert_eq!(tool.state, SpaceToolState::WaitingForStart);
    }

    #[test]
    fn test_first_click_starts_drawing() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        let response = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 10.0,
            y: 20.0,
        });

        assert_eq!(response, ToolResponse::Consumed);
        assert!(matches!(tool.state, SpaceToolState::Drawing { .. }));
    }

    #[test]
    fn test_subsequent_clicks_add_vertices() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
        });

        if let SpaceToolState::Drawing { ref vertices } = tool.state {
            assert_eq!(vertices.len(), 3);
        } else {
            panic!("Expected Drawing state");
        }
    }

    #[test]
    fn test_close_polygon_near_start() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        // Triángulo
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 50.0,
            y: 100.0,
        });

        // Clic cerca del primer punto para cerrar
        let response = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 1.0,
            y: 1.0,
        });

        assert!(
            matches!(response, ToolResponse::Completed(ToolResult::Space { ref vertices, .. }) if vertices.len() == 3),
            "Expected Completed with 3 vertices, got: {:?}",
            response
        );
        assert_eq!(tool.state, SpaceToolState::WaitingForStart);
    }

    #[test]
    fn test_right_click_closes_polygon_with_3_or_more() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
        });

        let response = tool.on_input(&InputEvent::Click {
            button: MouseButton::Right,
            x: 0.0,
            y: 0.0,
        });

        assert!(matches!(response, ToolResponse::Completed(_)));
    }

    #[test]
    fn test_right_click_ignored_with_less_than_3_vertices() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        });

        let response = tool.on_input(&InputEvent::Click {
            button: MouseButton::Right,
            x: 0.0,
            y: 0.0,
        });

        assert_eq!(response, ToolResponse::Ignored);
    }

    #[test]
    fn test_rubber_banding_during_drawing() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });

        let response = tool.on_input(&InputEvent::PointerMove { x: 50.0, y: 30.0 });

        assert!(
            matches!(response, ToolResponse::EphemeralLines(ref lines) if lines.len() == 1),
            "Expected 1 ephemeral line from vertex to cursor, got: {:?}",
            response
        );
    }

    #[test]
    fn test_rubber_banding_with_multiple_segments() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });
        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        });

        let response = tool.on_input(&InputEvent::PointerMove { x: 100.0, y: 100.0 });

        // Debería haber 2 líneas: segment 0->1, y segment 1->cursor
        assert!(
            matches!(response, ToolResponse::EphemeralLines(ref lines) if lines.len() == 2),
            "Expected 2 ephemeral lines (1 segment + 1 rubber-band), got: {:?}",
            response
        );
    }

    #[test]
    fn test_escape_cancels_drawing() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        });

        let response = tool.on_input(&InputEvent::KeyDown {
            key: "Escape".to_string(),
        });

        assert_eq!(response, ToolResponse::Consumed);
        assert_eq!(tool.state, SpaceToolState::WaitingForStart);
    }

    #[test]
    fn test_pointer_move_ignored_before_drawing() {
        let mut tool = SpaceTool::new();
        tool.on_start();

        let response = tool.on_input(&InputEvent::PointerMove { x: 50.0, y: 30.0 });
        assert_eq!(response, ToolResponse::Ignored);
    }
}
