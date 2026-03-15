use fcad_core::domain::viewport::Camera;
use fcad_renderer::RenderMessage;
use std::sync::mpsc::Sender;

pub struct ViewportService;

impl ViewportService {
    pub fn update_viewport_rect(tx: &Sender<RenderMessage>, cam: &mut Camera, x: u32, y: u32, width: u32, height: u32) {
        let _ = tx.send(RenderMessage::ViewportUpdate(fcad_renderer::ViewportRect {
            x,
            y,
            width,
            height,
        }));

        cam.screen_width = width as f32;
        cam.screen_height = height as f32;
    }
}
