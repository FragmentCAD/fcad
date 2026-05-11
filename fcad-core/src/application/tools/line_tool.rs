use super::{Tool, ToolResponse, ToolResult};
use crate::application::input::{InputEvent, MouseButton};
use crate::infrastructure::ecs::spatial::SpatialIndex;

/// Estado interno de la LineTool.
#[derive(Debug, Clone, PartialEq)]
enum LineToolState {
    /// Esperando el primer clic.
    WaitingForStart,
    /// Dibujando: se ha fijado el punto inicial.
    Drawing { start_point: [f32; 2] },
}

/// LineTool: Herramienta básica para dibujar segmentos de línea.
pub struct LineTool {
    state: LineToolState,
    current_mouse: [f32; 2],
}

impl LineTool {
    pub fn new() -> Self {
        Self {
            state: LineToolState::WaitingForStart,
            current_mouse: [0.0, 0.0],
        }
    }
}

impl Tool for LineTool {
    fn name(&self) -> &str {
        "line"
    }

    fn on_start(&mut self) {
        self.state = LineToolState::WaitingForStart;
    }

    fn on_input(&mut self, event: &InputEvent, _spatial_index: &SpatialIndex) -> ToolResponse {
        match event {
            InputEvent::PointerMove { x, y } => {
                self.current_mouse = [*x, *y];
                if let LineToolState::Drawing { start_point } = self.state {
                    ToolResponse::EphemeralLines(vec![(start_point, self.current_mouse)])
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
                match self.state {
                    LineToolState::WaitingForStart => {
                        self.state = LineToolState::Drawing { start_point: point };
                        ToolResponse::Consumed
                    }
                    LineToolState::Drawing { start_point } => {
                        let result = ToolResult::Line {
                            start: start_point,
                            end: point,
                        };
                        self.state = LineToolState::WaitingForStart;
                        ToolResponse::Completed(result)
                    }
                }
            }
            InputEvent::KeyDown { key } if key == "Escape" => {
                self.on_cancel();
                ToolResponse::Consumed
            }
            InputEvent::Click {
                button: MouseButton::Right,
                ..
            } => {
                self.on_cancel();
                ToolResponse::Consumed
            }
            _ => ToolResponse::Ignored,
        }
    }

    fn on_cancel(&mut self) {
        self.state = LineToolState::WaitingForStart;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_tool_cycle() {
        let mut tool = LineTool::new();
        
        let si = SpatialIndex::new();
        // 1. Move before start
        let resp = tool.on_input(&InputEvent::PointerMove { x: 10.0, y: 10.0 }, &si);
        assert_eq!(resp, ToolResponse::Ignored);

        // 2. First Click
        let resp = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 0.0,
            y: 0.0,
        }, &si);
        assert_eq!(resp, ToolResponse::Consumed);
        assert_eq!(tool.state, LineToolState::Drawing { start_point: [0.0, 0.0] });

        // 3. Move during drawing (Rubber-banding)
        let resp = tool.on_input(&InputEvent::PointerMove { x: 50.0, y: 50.0 }, &si);
        if let ToolResponse::EphemeralLines(lines) = resp {
            assert_eq!(lines.len(), 1);
            assert_eq!(lines[0], ([0.0, 0.0], [50.0, 50.0]));
        } else {
            panic!("Expected EphemeralLines");
        }

        // 4. Second Click (Completion)
        let resp = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 0.0,
        }, &si);
        assert_eq!(
            resp,
            ToolResponse::Completed(ToolResult::Line {
                start: [0.0, 0.0],
                end: [100.0, 0.0]
            })
        );
        assert_eq!(tool.state, LineToolState::WaitingForStart);
    }
}
