pub mod space_tool;

use super::input::InputEvent;

/// Respuesta que una herramienta devuelve al ToolManager tras procesar un evento.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolResponse {
    /// La herramienta no hizo nada con este evento.
    Ignored,
    /// La herramienta consumió el evento y no necesita feedback visual.
    Consumed,
    /// La herramienta pide dibujar geometría temporal (rubber-banding).
    /// Contiene pares de puntos `[(x1,y1), (x2,y2)]` representando líneas temporales.
    EphemeralLines(Vec<([f32; 2], [f32; 2])>),
    /// La herramienta completó su ciclo y generó un resultado final.
    Completed(ToolResult),
}

/// Resultado final de una herramienta al completar su ciclo.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolResult {
    /// Un recinto semántico definido por sus vértices.
    Space {
        vertices: Vec<[f32; 2]>,
        space_kind: String,
    },
}

/// Trait que define el ciclo de vida de una herramienta CAD interactiva.
/// Cada herramienta implementa cómo reacciona ante eventos de entrada del usuario.
pub trait Tool: Send + Sync {
    /// Nombre identificador de la herramienta (e.g., "space", "line").
    fn name(&self) -> &str;

    /// Se invoca cuando la herramienta se activa.
    fn on_start(&mut self);

    /// Procesa un evento de entrada y retorna la respuesta correspondiente.
    fn on_input(&mut self, event: &InputEvent) -> ToolResponse;

    /// Cancela la operación actual y resetea el estado de la herramienta.
    fn on_cancel(&mut self);
}

/// El ToolManager es el enrutador central de eventos.
/// Decide si un evento es de navegación (cámara) o de herramienta.
pub struct ToolManager {
    active_tool: Option<Box<dyn Tool>>,
}

impl ToolManager {
    pub fn new() -> Self {
        Self { active_tool: None }
    }

    /// Establece la herramienta activa. Cancela la anterior si existe.
    pub fn set_tool(&mut self, tool: Box<dyn Tool>) {
        if let Some(ref mut old) = self.active_tool {
            old.on_cancel();
        }
        let mut t = tool;
        t.on_start();
        self.active_tool = Some(t);
    }

    /// Desactiva la herramienta actual.
    pub fn clear_tool(&mut self) {
        if let Some(ref mut t) = self.active_tool {
            t.on_cancel();
        }
        self.active_tool = None;
    }

    /// Devuelve el nombre de la herramienta activa, si hay alguna.
    pub fn active_tool_name(&self) -> Option<&str> {
        self.active_tool.as_ref().map(|t| t.name())
    }

    /// Procesa un evento de entrada.
    /// - Si es un evento de navegación, devuelve `NavigationEvent` para que el caller actualice la cámara.
    /// - Si hay herramienta activa, le delega el evento.
    /// - Si no hay herramienta ni navegación, lo ignora.
    pub fn process_input(&mut self, event: &InputEvent) -> ToolManagerResponse {
        // Prioridad 1: Navegación (siempre funciona, incluso con herramienta activa)
        if event.is_navigation() {
            return ToolManagerResponse::Navigation(event.clone());
        }

        // Prioridad 2: Herramienta activa
        if let Some(ref mut tool) = self.active_tool {
            let response = tool.on_input(event);
            return ToolManagerResponse::Tool(response);
        }

        // Sin herramienta ni navegación
        ToolManagerResponse::Unhandled
    }
}

/// Respuesta del ToolManager al caller (normalmente el bridge de Tauri).
#[derive(Debug, Clone)]
pub enum ToolManagerResponse {
    /// El evento es de navegación. El caller debe actualizar la cámara.
    Navigation(InputEvent),
    /// El evento fue procesado por la herramienta activa.
    Tool(ToolResponse),
    /// Nadie procesó el evento.
    Unhandled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::input::{InputEvent, MouseButton};

    // Mock Tool para testing
    struct MockTool {
        started: bool,
        cancelled: bool,
        last_input: Option<InputEvent>,
    }

    impl MockTool {
        fn new() -> Self {
            Self {
                started: false,
                cancelled: false,
                last_input: None,
            }
        }
    }

    impl Tool for MockTool {
        fn name(&self) -> &str {
            "mock"
        }

        fn on_start(&mut self) {
            self.started = true;
        }

        fn on_input(&mut self, event: &InputEvent) -> ToolResponse {
            self.last_input = Some(event.clone());
            ToolResponse::Consumed
        }

        fn on_cancel(&mut self) {
            self.cancelled = true;
        }
    }

    #[test]
    fn test_tool_manager_starts_empty() {
        let tm = ToolManager::new();
        assert!(tm.active_tool_name().is_none());
    }

    #[test]
    fn test_set_tool_activates_and_calls_on_start() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));
        assert_eq!(tm.active_tool_name(), Some("mock"));
    }

    #[test]
    fn test_clear_tool_deactivates() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));
        tm.clear_tool();
        assert!(tm.active_tool_name().is_none());
    }

    #[test]
    fn test_scroll_always_returns_navigation() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));

        let scroll = InputEvent::Scroll {
            delta_y: 1.0,
            anchor_x: 100.0,
            anchor_y: 200.0,
        };
        let response = tm.process_input(&scroll);
        assert!(matches!(response, ToolManagerResponse::Navigation(_)));
    }

    #[test]
    fn test_middle_drag_always_returns_navigation() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));

        let drag = InputEvent::PointerDrag {
            button: MouseButton::Middle,
            dx: 10.0,
            dy: 5.0,
        };
        let response = tm.process_input(&drag);
        assert!(matches!(response, ToolManagerResponse::Navigation(_)));
    }

    #[test]
    fn test_left_click_goes_to_tool() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));

        let click = InputEvent::Click {
            button: MouseButton::Left,
            x: 50.0,
            y: 50.0,
        };
        let response = tm.process_input(&click);
        assert!(matches!(
            response,
            ToolManagerResponse::Tool(ToolResponse::Consumed)
        ));
    }

    #[test]
    fn test_no_tool_returns_unhandled() {
        let mut tm = ToolManager::new();

        let click = InputEvent::Click {
            button: MouseButton::Left,
            x: 50.0,
            y: 50.0,
        };
        let response = tm.process_input(&click);
        assert!(matches!(response, ToolManagerResponse::Unhandled));
    }

    #[test]
    fn test_setting_new_tool_cancels_old() {
        let mut tm = ToolManager::new();
        tm.set_tool(Box::new(MockTool::new()));
        // Setting a new tool should cancel the old one
        tm.set_tool(Box::new(MockTool::new()));
        assert_eq!(tm.active_tool_name(), Some("mock"));
    }
}
