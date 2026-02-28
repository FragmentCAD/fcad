use tauri::Manager;
use std::sync::{Arc, Mutex};
use fcad_core::infrastructure::ecs::spatial::SpatialIndex;
use fcad_core::application::tools::ToolManager;
use fcad_core::application::tools::space_tool::SpaceTool;
use fcad_core::application::snap::{SnapState, WorldGeometryProvider};

// Estado global para la aplicación
struct AppState {
    spatial_index: Mutex<SpatialIndex>,
    render_tx: Mutex<std::sync::mpsc::Sender<fcad_renderer::RenderMessage>>,
    tool_manager: Mutex<ToolManager>,
    world: Arc<Mutex<bevy_ecs::world::World>>,
    zoom: Mutex<f32>,
    current_theme: Mutex<fcad_core::domain::theme::Theme>,
}

#[tauri::command]
fn get_snap_state(state: tauri::State<'_, AppState>) -> SnapState {
    let tm = state.tool_manager.lock().unwrap();
    tm.snap_engine.state
}

#[tauri::command]
fn toggle_ortho(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    tm.snap_engine.state.ortho_enabled = !tm.snap_engine.state.ortho_enabled;
    tm.snap_engine.state.ortho_enabled
}

#[tauri::command]
fn toggle_osnaps(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    tm.snap_engine.state.osnaps_enabled = !tm.snap_engine.state.osnaps_enabled;
    tm.snap_engine.state.osnaps_enabled
}

#[tauri::command]
fn toggle_grid_snap(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    tm.snap_engine.state.grid_snap_enabled = !tm.snap_engine.state.grid_snap_enabled;
    tm.snap_engine.state.grid_snap_enabled
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
fn update_viewport_rect(window: tauri::Window, state: tauri::State<'_, AppState>, x: f64, y: f64, width: f64, height: f64) {
    let factor = window.scale_factor().unwrap_or(1.0);
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::ViewportUpdate(fcad_renderer::ViewportRect {
            x: (x * factor) as u32,
            y: (y * factor) as u32,
            width: (width * factor) as u32,
            height: (height * factor) as u32,
        }));
    }
}

/// Comando IPC para Zoom de cámara (enviado desde React onWheel).
#[tauri::command]
fn send_camera_zoom(window: tauri::Window, state: tauri::State<'_, AppState>, factor: f32, anchor_x: f32, anchor_y: f32) {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::CameraZoom {
            factor,
            anchor_x: anchor_x * scale,
            anchor_y: anchor_y * scale,
        });
    }
    // Sincronizar zoom en el estado para el SnapEngine
    let mut z = state.zoom.lock().unwrap();
    *z = (*z * factor).clamp(0.001, 100_000.0);
}

/// Comando IPC para Pan de cámara (enviado desde React middle-drag).
#[tauri::command]
fn send_camera_pan(window: tauri::Window, state: tauri::State<'_, AppState>, dx: f32, dy: f32) {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::CameraPan { 
            dx: dx * scale, 
            dy: dy * scale 
        });
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
fn send_tool_click(window: tauri::Window, state: tauri::State<'_, AppState>, button: String, x: f32, y: f32) -> String {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    use fcad_core::application::input::{InputEvent, MouseButton};
    let btn = match button.as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    let event = InputEvent::Click { 
        button: btn, 
        x: x * scale, 
        y: y * scale 
    };

    let mut tm = state.tool_manager.lock().unwrap();
    let index = state.spatial_index.lock().unwrap();
    let world = state.world.lock().unwrap();
    let zoom = state.zoom.lock().unwrap();
    
    let provider = WorldGeometryProvider { world: &world };
    let response = tm.process_input(&event, &index, &provider, *zoom);
    format!("{:?}", response)
}

#[tauri::command]
fn get_current_theme(state: tauri::State<'_, AppState>) -> fcad_core::domain::theme::Theme {
    state.current_theme.lock().unwrap().clone()
}

#[tauri::command]
fn get_themes_list() -> Vec<String> {
    let themes_dir = std::path::Path::new("..").join("..").join("fcad-assets").join("themes");
    let mut themes = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(themes_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") {
                    themes.push(name.replace(".json", ""));
                }
            }
        }
    }
    themes
}

#[tauri::command]
fn switch_theme(state: tauri::State<'_, AppState>, theme_name: String) -> Result<fcad_core::domain::theme::Theme, String> {
    let theme_path = std::path::Path::new("..").join("..").join("fcad-assets").join("themes").join(format!("{}.json", theme_name));
    
    let content = std::fs::read_to_string(&theme_path)
        .map_err(|e| format!("No se pudo leer el tema '{}': {}", theme_name, e))?;
    
    let theme: fcad_core::domain::theme::Theme = serde_json::from_str(&content)
        .map_err(|e| format!("Error al parsear el tema '{}': {}", theme_name, e))?;
    
    // Update stored theme in AppState
    if let Ok(mut current) = state.current_theme.lock() {
        *current = theme.clone();
    }
    
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::UpdateTheme(theme.clone()));
    }
    
    Ok(theme)
}

#[tauri::command]
fn get_layers(state: tauri::State<'_, AppState>) -> Vec<fcad_core::infrastructure::ecs::ncs::NcsLayerDef> {
    let world = state.world.lock().unwrap();
    use fcad_core::infrastructure::ecs::ncs::LayerStandards;
    
    if let Some(standards) = world.get_resource::<LayerStandards>() {
        standards.get_layers_by_discipline("A") // Hardcoded "A" for now per specs
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn get_adapted_layers(state: tauri::State<'_, AppState>) -> Vec<fcad_core::infrastructure::ecs::ncs::NcsLayerDef> {
    let world = state.world.lock().unwrap();
    let theme = state.current_theme.lock().unwrap().clone();
    use fcad_core::infrastructure::ecs::ncs::LayerStandards;
    
    if let Some(standards) = world.get_resource::<LayerStandards>() {
        standards.get_layers_by_discipline("A")
            .into_iter()
            .map(|mut layer| {
                layer.color_hex = theme.adapt_layer_color(&layer.color_hex);
                layer
            })
            .collect()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn set_active_layer(state: tauri::State<'_, AppState>, name: String) -> String {
    let mut world = state.world.lock().unwrap();
    use fcad_core::infrastructure::ecs::ncs::ActiveLayer;
    
    world.insert_resource(ActiveLayer(name.clone()));
    name
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (tx, rx) = std::sync::mpsc::channel();
    let world = Arc::new(Mutex::new(bevy_ecs::world::World::new()));
    
    tauri::Builder::default()
        .manage(AppState {
            spatial_index: Mutex::new(SpatialIndex::new()),
            render_tx: Mutex::new(tx.clone()),
            tool_manager: Mutex::new(ToolManager::new()),
            world: world.clone(),
            zoom: Mutex::new(1.0),
            current_theme: Mutex::new(fcad_core::domain::theme::Theme::default()),
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
            toggle_grid_snap,
            get_snap_state,
            get_current_theme,
            get_themes_list,
            switch_theme,
            get_layers,
            get_adapted_layers,
            set_active_layer,
        ])
        .setup(move |app| {
            let main_window = Arc::new(app.get_webview_window("main").unwrap());
            let size = main_window.inner_size().unwrap_or(tauri::PhysicalSize::new(800, 600));

            // --- Inicialización de Capas (NCS) ---
            let ncs_path = std::path::Path::new("..").join("..").join("fcad-assets").join("standards").join("layers").join("ncs_layers_A.yaml");
            let mut world_guard = world.lock().unwrap();
            
            use fcad_core::infrastructure::ecs::ncs::{LayerStandards, ActiveLayer};
            let mut standards = LayerStandards::new();
            if let Ok(content) = std::fs::read_to_string(ncs_path) {
                if let Err(e) = standards.load_from_yaml(&content) {
                    eprintln!("Error al cargar catálogo NCS: {}", e);
                } else {
                    println!("Catálogo NCS cargado: {} capas registradas.", standards.catalog.len());
                }
            } else {
                eprintln!("AVISO: No se encontró catálogo NCS inicial.");
            }
            
            world_guard.insert_resource(standards);
            world_guard.insert_resource(ActiveLayer::default());
            drop(world_guard);
            // ------------------------------------

            // Detección automática de tema basado en el sistema operativo
            let theme = match main_window.theme().unwrap_or(tauri::Theme::Dark) {
                tauri::Theme::Light => fcad_core::domain::theme::Theme::architect(),
                tauri::Theme::Dark => fcad_core::domain::theme::Theme::midnight(),
                _ => fcad_core::domain::theme::Theme::midnight(),
            };
            
            println!("FragmentCAD: Sistema detectado en modo {:?}. Aplicando tema: {}", 
                     main_window.theme().unwrap_or(tauri::Theme::Dark), 
                     theme.name);

            // Persist detected theme in AppState for get_current_theme
            if let Some(state) = app.try_state::<AppState>() {
                if let Ok(mut current) = state.current_theme.lock() {
                    *current = theme.clone();
                }
            }

            let tx_clone = tx.clone();
            
            // Enviar tema inicial al renderer
            let _ = tx_clone.send(fcad_renderer::RenderMessage::UpdateTheme(theme));

            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(size) = event {
                    let _ = tx_clone.send(fcad_renderer::RenderMessage::WindowResize(size.width, size.height));
                }
            });

            fcad_renderer::spawn_render_thread(main_window, size.width, size.height, world, rx);

            println!("FragmentCAD: Setup complete. Waiting for frontend bootstrap.");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

