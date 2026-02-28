use tauri::Manager;
use std::sync::Mutex;
use fcad_core::infrastructure::ecs::spatial::SpatialIndex;
use fcad_core::application::tools::ToolManager;
use fcad_core::application::tools::space_tool::SpaceTool;

// Estado global para la aplicación
struct AppState {
    spatial_index: Mutex<SpatialIndex>,
    render_tx: Mutex<std::sync::mpsc::Sender<fcad_renderer::RenderMessage>>,
    tool_manager: Mutex<ToolManager>,
}

#[tauri::command]
fn hit_test(state: tauri::State<'_, AppState>, x: f64, y: f64) -> Vec<String> {
    let index = state.spatial_index.lock().unwrap();
    let results = index.query_point(x, y);
    
    // Convertimos los Entity IDs a String para enviarlos al frontend
    let ids: Vec<String> = results.iter().map(|e| format!("{:?}", e)).collect();
    
    if !ids.is_empty() {
        println!("Hit Test at ({}, {}): Intersected IDs: {:?}", x, y, ids);
    }
    
    ids
}

#[tauri::command]
fn update_viewport_rect(state: tauri::State<'_, AppState>, x: f64, y: f64, width: f64, height: f64) {
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::ViewportUpdate(fcad_renderer::ViewportRect {
            x: x as u32,
            y: y as u32,
            width: width as u32,
            height: height as u32,
        }));
    }
}

/// Comando IPC para Zoom de cámara (enviado desde React onWheel).
#[tauri::command]
fn send_camera_zoom(state: tauri::State<'_, AppState>, factor: f32, anchor_x: f32, anchor_y: f32) {
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::CameraZoom {
            factor,
            anchor_x,
            anchor_y,
        });
    }
}

/// Comando IPC para Pan de cámara (enviado desde React middle-drag).
#[tauri::command]
fn send_camera_pan(state: tauri::State<'_, AppState>, dx: f32, dy: f32) {
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::CameraPan { dx, dy });
    }
}

/// Activa una herramienta por nombre. Actualmente soporta: "space", "none".
#[tauri::command]
fn set_active_tool(state: tauri::State<'_, AppState>, tool_name: String) -> String {
    let mut tm = state.tool_manager.lock().unwrap();
    match tool_name.as_str() {
        "space" => {
            tm.set_tool(Box::new(SpaceTool::new()));
            "space".to_string()
        }
        "none" | "" => {
            tm.clear_tool();
            "none".to_string()
        }
        other => format!("unknown tool: {}", other),
    }
}

/// Devuelve el nombre de la herramienta activa.
#[tauri::command]
fn get_active_tool(state: tauri::State<'_, AppState>) -> String {
    let tm = state.tool_manager.lock().unwrap();
    tm.active_tool_name().unwrap_or("none").to_string()
}

/// Envía un clic del usuario al ToolManager.
/// Devuelve un JSON con la respuesta de la herramienta.
#[tauri::command]
fn send_tool_click(state: tauri::State<'_, AppState>, button: String, x: f32, y: f32) -> String {
    use fcad_core::application::input::{InputEvent, MouseButton};
    let btn = match button.as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    let event = InputEvent::Click { button: btn, x, y };
    let mut tm = state.tool_manager.lock().unwrap();
    let response = tm.process_input(&event);
    format!("{:?}", response)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Inicializamos un índice espacial con una línea de prueba (mock)
    let mock_index = SpatialIndex::new();
    println!("Initializing SpatialIndex with MVP Mock data...");
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    tauri::Builder::default()
        .manage(AppState {
            spatial_index: Mutex::new(mock_index),
            render_tx: Mutex::new(tx.clone()),
            tool_manager: Mutex::new(ToolManager::new()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            hit_test,
            update_viewport_rect,
            send_camera_zoom,
            send_camera_pan,
            set_active_tool,
            get_active_tool,
            send_tool_click,
        ])
        .setup(move |app| {
            let main_window = std::sync::Arc::new(app.get_webview_window("main").unwrap());
            
            // Pasamos un tamaño arbitrario o el actual
            let size = main_window.inner_size().unwrap_or(tauri::PhysicalSize::new(800, 600));
            
            let tx_clone = tx.clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(size) = event {
                    let _ = tx_clone.send(fcad_renderer::RenderMessage::WindowResize(size.width, size.height));
                }
            });

            fcad_renderer::spawn_render_thread(main_window, size.width, size.height, rx);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

