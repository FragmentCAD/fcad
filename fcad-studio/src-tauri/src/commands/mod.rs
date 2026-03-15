use tauri;
use crate::state::AppState;
use fcad_core::application::snap::{SnapState, WorldGeometryProvider};
use crate::services::theme::ThemeService;
use crate::services::snap::SnapService;
use crate::services::tool::ToolService;

#[tauri::command]
pub fn get_snap_state(state: tauri::State<'_, AppState>) -> SnapState {
    let tm = state.tool_manager.lock().unwrap();
    tm.snap_engine.state
}

#[tauri::command]
pub fn toggle_ortho(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    SnapService::toggle_ortho(&mut tm.snap_engine.state)
}

#[tauri::command]
pub fn toggle_osnaps(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    SnapService::toggle_osnaps(&mut tm.snap_engine.state)
}

#[tauri::command]
pub fn toggle_grid_snap(state: tauri::State<'_, AppState>) -> bool {
    let mut tm = state.tool_manager.lock().unwrap();
    SnapService::toggle_grid_snap(&mut tm.snap_engine.state)
}

#[tauri::command]
pub fn hit_test(state: tauri::State<'_, AppState>, x: f64, y: f64) -> Vec<String> {
    let index = state.spatial_index.lock().unwrap();
    let results = index.query_point(x, y);
    
    let ids: Vec<String> = results.iter().map(|e| format!("{:?}", e)).collect();
    
    if !ids.is_empty() {
        println!("Hit Test at ({}, {}): Intersected IDs: {:?}", x, y, ids);
    }
    
    ids
}

#[tauri::command]
pub fn update_viewport_rect(window: tauri::Window, state: tauri::State<'_, AppState>, x: f64, y: f64, width: f64, height: f64) {
    let factor = window.scale_factor().unwrap_or(1.0);
    
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::ViewportUpdate(fcad_renderer::ViewportRect {
            x: (x * factor) as u32,
            y: (y * factor) as u32,
            width: (width * factor) as u32,
            height: (height * factor) as u32,
        }));
    }

    let mut world = state.world.lock().unwrap();
    if let Some(mut cam) = world.get_resource_mut::<fcad_core::domain::viewport::Camera>() {
        cam.screen_width = (width * factor) as f32;
        cam.screen_height = (height * factor) as f32;
    }
}

#[tauri::command]
pub fn send_camera_zoom(window: tauri::Window, state: tauri::State<'_, AppState>, factor: f32, anchor_x: f32, anchor_y: f32) {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    
    let mut world = state.world.lock().unwrap();
    if let Some(mut cam) = world.get_resource_mut::<fcad_core::domain::viewport::Camera>() {
        cam.zoom_at(factor, anchor_x * scale, anchor_y * scale);
    }
}

#[tauri::command]
pub fn send_camera_pan(window: tauri::Window, state: tauri::State<'_, AppState>, dx: f32, dy: f32) {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    
    let mut world = state.world.lock().unwrap();
    if let Some(mut cam) = world.get_resource_mut::<fcad_core::domain::viewport::Camera>() {
        cam.pan(dx * scale, dy * scale);
    }
}

#[tauri::command]
pub fn set_active_tool(state: tauri::State<'_, AppState>, tool_name: String) -> String {
    let mut tm = state.tool_manager.lock().unwrap();
    ToolService::set_tool(&mut tm, &tool_name)
}

#[tauri::command]
pub fn get_active_tool(state: tauri::State<'_, AppState>) -> String {
    let tm = state.tool_manager.lock().unwrap();
    ToolService::get_active_tool(&tm)
}

#[tauri::command]
pub fn send_tool_click(window: tauri::Window, state: tauri::State<'_, AppState>, button: String, x: f32, y: f32) -> String {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    let world_pos = {
        let world = state.world.lock().unwrap();
        if let Some(cam) = world.get_resource::<fcad_core::domain::viewport::Camera>() {
            cam.unproject(x * scale, y * scale)
        } else {
            glam::Vec2::new(x * scale, y * scale)
        }
    };

    use fcad_core::application::input::{InputEvent, MouseButton};
    use fcad_core::application::tools::{ToolManagerResponse, ToolResponse, ToolResult};
    use fcad_core::domain::{Geometry, Layer, math::primitives::Point2D, math::primitives::Line};
    use fcad_core::infrastructure::ecs::ncs::ActiveLayer;

    let btn = match button.as_str() {
        "left" => MouseButton::Left,
        "right" => MouseButton::Right,
        "middle" => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    let event = InputEvent::Click { 
        button: btn, 
        x: world_pos.x, 
        y: world_pos.y 
    };

    let mut tm = state.tool_manager.lock().unwrap();
    let index = state.spatial_index.lock().unwrap();
    let mut world = state.world.lock().unwrap();
    let zoom = if let Some(cam) = world.get_resource::<fcad_core::domain::viewport::Camera>() { cam.zoom } else { 1.0 };
    
    let provider = WorldGeometryProvider { world: &world };
    let response = tm.process_input(&event, &index, &provider, zoom);

    if let ToolManagerResponse::Tool(ToolResponse::Completed(result), _) = &response {
        match result {
            ToolResult::Line { start, end } => {
                let active_layer = world.get_resource::<ActiveLayer>().cloned().unwrap_or_default();
                world.spawn((
                    Geometry::Line(Line {
                        start: Point2D::new(start[0] as f64, start[1] as f64),
                        end: Point2D::new(end[0] as f64, end[1] as f64),
                    }),
                    Layer(active_layer.0),
                ));
            }
            ToolResult::Rectangle { p1, p2 } => {
                use fcad_core::domain::math::primitives::Rectangle;
                let active_layer = world.get_resource::<ActiveLayer>().cloned().unwrap_or_default();
                world.spawn((
                    Geometry::Rectangle(Rectangle {
                        p1: Point2D::new(p1[0] as f64, p1[1] as f64),
                        p2: Point2D::new(p2[0] as f64, p2[1] as f64),
                    }),
                    Layer(active_layer.0),
                ));
            }
            ToolResult::Deleted(entities) => {
                for entity in entities {
                    if world.get_entity(*entity).is_some() {
                        world.despawn(*entity);
                    }
                }
            }
            ToolResult::Space { vertices, space_kind } => {
                println!("Space completed: {} with {} vertices", space_kind, vertices.len());
            }
        }
    }

    format!("{:?}", response)
}

#[tauri::command]
pub fn send_tool_move(window: tauri::Window, state: tauri::State<'_, AppState>, x: f32, y: f32) -> String {
    let scale = window.scale_factor().unwrap_or(1.0) as f32;
    let world_pos = {
        let world = state.world.lock().unwrap();
        if let Some(cam) = world.get_resource::<fcad_core::domain::viewport::Camera>() {
            cam.unproject(x * scale, y * scale)
        } else {
            glam::Vec2::new(x * scale, y * scale)
        }
    };

    use fcad_core::application::input::InputEvent;
    use fcad_core::application::tools::{ToolManagerResponse, ToolResponse};

    let event = InputEvent::PointerMove { 
        x: world_pos.x, 
        y: world_pos.y 
    };

    let mut tm = state.tool_manager.lock().unwrap();
    let index = state.spatial_index.lock().unwrap();
    let world = state.world.lock().unwrap();
    let zoom = if let Some(cam) = world.get_resource::<fcad_core::domain::viewport::Camera>() { cam.zoom } else { 1.0 };
    
    let provider = WorldGeometryProvider { world: &world };
    let response = tm.process_input(&event, &index, &provider, zoom);

    if let ToolManagerResponse::Tool(resp, _) = &response {
        if let Ok(tx) = state.render_tx.lock() {
            let vertices: Vec<fcad_renderer::Vertex> = match resp {
                ToolResponse::EphemeralLines(lines) => {
                    lines.iter()
                        .flat_map(|(start, end)| {
                            vec![
                                fcad_renderer::Vertex {
                                    position: [start[0], start[1], 0.0],
                                    color: [0.0, 1.0, 1.0, 1.0],
                                },
                                fcad_renderer::Vertex {
                                    position: [end[0], end[1], 0.0],
                                    color: [0.0, 1.0, 1.0, 1.0],
                                },
                            ]
                        })
                        .collect()
                }
                _ => Vec::new(),
            };
            let _ = tx.send(fcad_renderer::RenderMessage::UpdateEphemeral(vertices));
        }
    }

    format!("{:?}", response)
}

#[tauri::command]
pub fn get_current_theme(state: tauri::State<'_, AppState>) -> fcad_core::domain::theme::Theme {
    state.current_theme.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_themes_list() -> Vec<String> {
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
pub fn switch_theme(state: tauri::State<'_, AppState>, theme_name: String) -> Result<fcad_core::domain::theme::Theme, String> {
    let theme_path = std::path::Path::new("..").join("..").join("fcad-assets").join("themes").join(format!("{}.json", theme_name));
    
    let content = std::fs::read_to_string(&theme_path)
        .map_err(|e| format!("No se pudo leer el tema '{}': {}", theme_name, e))?;
    
    let theme: fcad_core::domain::theme::Theme = serde_json::from_str(&content)
        .map_err(|e| format!("Error al parsear el tema '{}': {}", theme_name, e))?;
    
    if let Ok(mut current) = state.current_theme.lock() {
        *current = theme.clone();
    }
    
    if let Ok(tx) = state.render_tx.lock() {
        let _ = tx.send(fcad_renderer::RenderMessage::UpdateTheme(theme.clone()));
    }
    
    Ok(theme)
}

#[tauri::command]
pub fn get_layers(state: tauri::State<'_, AppState>) -> Vec<fcad_core::infrastructure::ecs::ncs::NcsLayerDef> {
    let world = state.world.lock().unwrap();
    use fcad_core::infrastructure::ecs::ncs::LayerStandards;
    
    if let Some(standards) = world.get_resource::<LayerStandards>() {
        standards.get_layers_by_discipline("A")
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn get_adapted_layers(state: tauri::State<'_, AppState>) -> Vec<fcad_core::infrastructure::ecs::ncs::NcsLayerDef> {
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
pub fn set_active_layer(state: tauri::State<'_, AppState>, name: String) -> String {
    let mut world = state.world.lock().unwrap();
    use fcad_core::infrastructure::ecs::ncs::ActiveLayer;
    
    world.insert_resource(ActiveLayer(name.clone()));
    name
}
