use tauri::Manager;
use std::sync::{Arc, Mutex};
use fcad_core::infrastructure::ecs::spatial::SpatialIndex;
use fcad_core::application::tools::ToolManager;

pub mod commands;
pub mod state;
pub mod services;
pub mod models;

use crate::state::AppState;

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
            current_theme: Mutex::new(fcad_core::domain::theme::Theme::default()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::hit_test,
            commands::update_viewport_rect,
            commands::send_camera_zoom,
            commands::send_camera_pan,
            commands::set_active_tool,
            commands::get_active_tool,
            commands::send_tool_click,
            commands::send_tool_move,
            commands::toggle_grid_snap,
            commands::get_snap_state,
            commands::get_current_theme,
            commands::get_themes_list,
            commands::switch_theme,
            commands::get_layers,
            commands::get_adapted_layers,
            commands::set_active_layer,
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
            world_guard.insert_resource(fcad_core::domain::viewport::Camera::new(size.width as f32, size.height as f32));
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
