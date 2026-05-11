use fcad_core::application::tools::ToolManager;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub render_tx: Mutex<std::sync::mpsc::Sender<fcad_renderer::RenderMessage>>,
    pub tool_manager: Mutex<ToolManager>,
    pub world: Arc<Mutex<bevy_ecs::world::World>>,
    pub current_theme: Mutex<fcad_core::domain::theme::Theme>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::world::World;
    use fcad_core::application::tools::ToolManager;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_app_state_creation() {
        let (tx, _rx) = std::sync::mpsc::channel();
        let _state = AppState {
            render_tx: Mutex::new(tx),
            tool_manager: Mutex::new(ToolManager::new()),
            world: Arc::new(Mutex::new(World::new())),
            current_theme: Mutex::new(fcad_core::domain::theme::Theme::default()),
        };
    }
}
