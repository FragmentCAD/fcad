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

#[cfg(test)]
mod tests {
    use super::*;
    use fcad_core::domain::viewport::Camera;
    use std::sync::mpsc::channel;

    #[test]
    fn test_update_viewport_rect() {
        let (tx, rx) = channel();
        let mut cam = Camera::default();
        let x = 10;
        let y = 20;
        let width = 800;
        let height = 600;

        ViewportService::update_viewport_rect(&tx, &mut cam, x, y, width, height);

        let msg = rx.try_recv().unwrap();
        if let RenderMessage::ViewportUpdate(rect) = msg {
            assert_eq!(rect.x, x);
            assert_eq!(rect.y, y);
            assert_eq!(rect.width, width);
            assert_eq!(rect.height, height);
        } else {
            panic!("Expected ViewportUpdate message");
        }

        assert_eq!(cam.screen_width, width as f32);
        assert_eq!(cam.screen_height, height as f32);
    }
}
