use super::{Tool, ToolResponse, ToolResult};
use crate::application::input::{InputEvent, MouseButton};
use crate::infrastructure::ecs::spatial::SpatialIndex;

pub struct RectTool {
    first_point: Option<[f32; 2]>,
}

impl RectTool {
    pub fn new() -> Self {
        Self { first_point: None }
    }
}

impl Tool for RectTool {
    fn name(&self) -> &str {
        "rect"
    }

    fn on_start(&mut self) {
        self.first_point = None;
    }

    fn on_input(&mut self, event: &InputEvent, _spatial_index: &SpatialIndex) -> ToolResponse {
        match event {
            InputEvent::Click { button, x, y } if *button == MouseButton::Left => {
                if let Some(p1) = self.first_point {
                    let result = ToolResult::Rectangle { p1, p2: [*x, *y] };
                    self.first_point = None;
                    ToolResponse::Completed(result)
                } else {
                    self.first_point = Some([*x, *y]);
                    ToolResponse::Consumed
                }
            }
            InputEvent::PointerMove { x, y } => {
                if let Some(p1) = self.first_point {
                    // Rubber-band: 4 lines of the rectangle
                    let p2 = [*x, *y];
                    let lines = vec![
                        (p1, [p2[0], p1[1]]),
                        ([p2[0], p1[1]], p2),
                        (p2, [p1[0], p2[1]]),
                        ([p1[0], p2[1]], p1),
                    ];
                    ToolResponse::EphemeralLines(lines)
                } else {
                    ToolResponse::Ignored
                }
            }
            _ => ToolResponse::Ignored,
        }
    }

    fn on_cancel(&mut self) {
        self.first_point = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::input::MouseButton;

    #[test]
    fn test_rect_tool_flow() {
        let mut tool = RectTool::new();
        let si = SpatialIndex::new();

        // 1. First click
        let resp1 = tool.on_input(
            &InputEvent::Click {
                button: MouseButton::Left,
                x: 0.0,
                y: 0.0,
            },
            &si,
        );
        assert_eq!(resp1, ToolResponse::Consumed);

        // 2. Move (Rubber-banding)
        let resp2 = tool.on_input(&InputEvent::PointerMove { x: 10.0, y: 5.0 }, &si);
        if let ToolResponse::EphemeralLines(lines) = resp2 {
            assert_eq!(lines.len(), 4);
        } else {
            panic!("Expected rubber-banding lines");
        }

        // 3. Second click
        let resp3 = tool.on_input(
            &InputEvent::Click {
                button: MouseButton::Left,
                x: 10.0,
                y: 5.0,
            },
            &si,
        );
        assert_eq!(
            resp3,
            ToolResponse::Completed(ToolResult::Rectangle {
                p1: [0.0, 0.0],
                p2: [10.0, 5.0],
            })
        );
    }
}
