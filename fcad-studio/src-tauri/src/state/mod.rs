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
