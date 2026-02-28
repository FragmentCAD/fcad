/// Enumeración de eventos de entrada (ratón/teclado) que el frontend envía al backend.
/// Estos eventos son la interfaz entre React (Tauri IPC) y el ToolManager en Rust.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// Evento de entrada unificado para el ToolManager.
/// React captura estos eventos del DOM y los serializa como payloads compactos vía Tauri IPC.
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Movimiento continuo del cursor (coordenadas de pantalla en píxeles).
    PointerMove { x: f32, y: f32 },
    /// Arrastre continuo del cursor (delta de movimiento en píxeles).
    PointerDrag { button: MouseButton, dx: f32, dy: f32 },
    /// Scroll de la rueda del ratón. `delta_y` positivo = zoom in.
    /// `anchor_x`, `anchor_y` = posición del cursor en pantalla donde se ancla el zoom.
    Scroll { delta_y: f32, anchor_x: f32, anchor_y: f32 },
    /// Clic simple de un botón del ratón (coordenadas de pantalla).
    Click { button: MouseButton, x: f32, y: f32 },
    /// Tecla presionada.
    KeyDown { key: String },
    /// Tecla liberada.
    KeyUp { key: String },
}

impl InputEvent {
    /// Devuelve `true` si el evento es un comando de navegación de cámara
    /// (scroll para zoom, arrastre de botón central para pan).
    pub fn is_navigation(&self) -> bool {
        matches!(
            self,
            InputEvent::Scroll { .. }
                | InputEvent::PointerDrag {
                    button: MouseButton::Middle,
                    ..
                }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_is_navigation() {
        let event = InputEvent::Scroll {
            delta_y: 1.0,
            anchor_x: 100.0,
            anchor_y: 200.0,
        };
        assert!(event.is_navigation());
    }

    #[test]
    fn test_middle_drag_is_navigation() {
        let event = InputEvent::PointerDrag {
            button: MouseButton::Middle,
            dx: 5.0,
            dy: -3.0,
        };
        assert!(event.is_navigation());
    }

    #[test]
    fn test_left_click_is_not_navigation() {
        let event = InputEvent::Click {
            button: MouseButton::Left,
            x: 50.0,
            y: 50.0,
        };
        assert!(!event.is_navigation());
    }

    #[test]
    fn test_pointer_move_is_not_navigation() {
        let event = InputEvent::PointerMove { x: 10.0, y: 20.0 };
        assert!(!event.is_navigation());
    }

    #[test]
    fn test_left_drag_is_not_navigation() {
        let event = InputEvent::PointerDrag {
            button: MouseButton::Left,
            dx: 1.0,
            dy: 1.0,
        };
        assert!(!event.is_navigation());
    }
}
