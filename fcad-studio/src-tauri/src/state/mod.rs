use std::sync::{Arc, Mutex};
use fcad_core::infrastructure::ecs::spatial::SpatialIndex;
use fcad_core::application::tools::ToolManager;

pub struct AppState {
    pub spatial_index: Mutex<SpatialIndex>,
    pub render_tx: Mutex<std::sync::mpsc::Sender<fcad_renderer::RenderMessage>>,
    pub tool_manager: Mutex<ToolManager>,
    pub world: Arc<Mutex<bevy_ecs::world::World>>,
    pub current_theme: Mutex<fcad_core::domain::theme::Theme>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use bevy_ecs::world::World;
    use fcad_core::application::tools::ToolManager;
    use fcad_core::infrastructure::ecs::spatial::SpatialIndex;

    #[test]
    fn test_app_state_creation() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let _state = AppState {
            spatial_index: Mutex::new(SpatialIndex::new()),
            render_tx: Mutex::new(tx),
            tool_manager: Mutex::new(ToolManager::new()),
            world: Arc::new(Mutex::new(World::new())),
            current_theme: Mutex::new(fcad_core::domain::theme::Theme::default()),
        };
    }
}
