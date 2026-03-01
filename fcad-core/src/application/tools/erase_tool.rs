use super::{Tool, ToolResponse, ToolResult};
use crate::application::input::{InputEvent, MouseButton};
use crate::infrastructure::ecs::spatial::{SpatialIndex, SpatialEntity};

pub struct EraseTool;

impl EraseTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for EraseTool {
    fn name(&self) -> &str {
        "erase"
    }

    fn on_start(&mut self) {}

    fn on_input(&mut self, event: &InputEvent, spatial_index: &SpatialIndex) -> ToolResponse {
        match event {
            InputEvent::Click { button, x, y } if *button == MouseButton::Left => {
                let hits = spatial_index.query_point(*x as f64, *y as f64);
                if !hits.is_empty() {
                    ToolResponse::Completed(ToolResult::Deleted(hits))
                } else {
                    ToolResponse::Consumed
                }
            }
            _ => ToolResponse::Ignored,
        }
    }

    fn on_cancel(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::*;
    use rstar::AABB;

    #[test]
    fn test_erase_tool_hits_entity() {
        let mut tool = EraseTool::new();
        let mut si = SpatialIndex::new();
        
        let entity_id = Entity::from_raw(123);
        let bounds = AABB::from_point([5.0, 5.0]);
        si.tree.insert(SpatialEntity { id: entity_id, envelope: bounds });
        
        // 1. Click on point (5,5)
        let resp = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 5.0,
            y: 5.0,
        }, &si);
        
        if let ToolResponse::Completed(ToolResult::Deleted(entities)) = resp {
            assert_eq!(entities.len(), 1);
            assert_eq!(entities[0], entity_id);
        } else {
            panic!("Expected ToolResult::Deleted");
        }
    }

    #[test]
    fn test_erase_tool_misses() {
        let mut tool = EraseTool::new();
        let si = SpatialIndex::new();
        
        let resp = tool.on_input(&InputEvent::Click {
            button: MouseButton::Left,
            x: 100.0,
            y: 100.0,
        }, &si);
        
        assert_eq!(resp, ToolResponse::Consumed);
    }
}
