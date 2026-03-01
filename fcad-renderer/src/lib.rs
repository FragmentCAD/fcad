use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub mod tessellator;
pub mod optimizer;
pub mod grid;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ViewportRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub enum RenderMessage {
    ViewportUpdate(ViewportRect),
    WindowResize(u32, u32),
    /// Actualización del tema visual
    UpdateTheme(fcad_core::domain::theme::Theme),
    /// Actualización de geometría temporal (feedback visual)
    UpdateEphemeral(Vec<Vertex>),
}

pub struct Renderer<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    // Grid
    pub grid_pipeline: wgpu::RenderPipeline,
    pub grid_vertex_buffer: wgpu::Buffer,
    pub grid_index_buffer: wgpu::Buffer,
    pub num_grid_indices: u32,
    // Ephemeral (Feedback)
    pub ephemeral_vertex_buffer: wgpu::Buffer,
    pub num_ephemeral_verts: u32,
    // Theme
    pub active_theme: fcad_core::domain::theme::Theme,
}

impl<'window> Renderer<'window> {
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32, optimizer: &optimizer::RenderOptimizer) -> Self 
    where 
        W: HasWindowHandle + HasDisplayHandle + 'window
    {
        let instance = wgpu::Instance::default();
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&*window) }.unwrap();
        let surface = unsafe { instance.create_surface_unsafe(target) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("FragmentCAD Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                    trace: Default::default(),
                },
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let width = if width == 0 { 800 } else { width };
        let height = if height == 0 { 600 } else { height };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Setup mock camera matching window sizes
        let aspect = width as f32 / height as f32;
        let view_proj = glam::Mat4::orthographic_rh(-10.0 * aspect, 10.0 * aspect, -10.0, 10.0, -1.0, 1.0);
        let camera_uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Grid Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut vertices = Vec::new();
        for inst in optimizer.instances.iter() {
            vertices.push(Vertex {
                position: [inst.start[0], inst.start[1], 0.0],
                color: inst.color,
            });
            vertices.push(Vertex {
                position: [inst.end[0], inst.end[1], 0.0],
                color: inst.color,
            });
        }

        // Si el ECS está vacío (no debería), aseguramos al menos alocar 1 vertex dummy
        if vertices.is_empty() {
            vertices.push(Vertex { position: [0.0; 3], color: [0.0; 4] });
        }

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        // Buffer inicial para la grilla (vacio por ahora, se llenara en el primer update_camera)
        let grid_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Vertex Buffer"),
            size: 1024 * 1024, // 1MB buffer para lineas de grilla
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let grid_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Index Buffer"),
            size: 1024 * 1024,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Buffer persistente para feedback (1000 vértices de margen)
        let ephemeral_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ephemeral Vertex Buffer"),
            size: (std::mem::size_of::<Vertex>() * 1000) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            camera_bind_group,
            camera_buffer,
            vertex_buffer,
            num_vertices: vertices.len() as u32,
            grid_pipeline,
            grid_vertex_buffer,
            grid_index_buffer,
            num_grid_indices: 0,
            ephemeral_vertex_buffer,
            num_ephemeral_verts: 0,
            active_theme: fcad_core::domain::theme::Theme::default(),
        }
    }

    pub fn update_ephemeral(&mut self, vertices: &[Vertex]) {
        self.num_ephemeral_verts = vertices.len() as u32;
        if !vertices.is_empty() {
            self.queue.write_buffer(&self.ephemeral_vertex_buffer, 0, bytemuck::cast_slice(vertices));
        }
    }

    pub fn update_theme(&mut self, theme: fcad_core::domain::theme::Theme) {
        self.active_theme = theme;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Actualiza el uniform buffer de la cámara y regenera la grilla.
    pub fn update_camera(&mut self, camera: &fcad_core::domain::viewport::Camera) {
        let vp = camera.build_view_projection_matrix();
        let uniform = CameraUniform {
            view_proj: vp.to_cols_array_2d(),
        };
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniform]));

        // Regenerar grilla
        let (grid_verts, grid_indices) = grid::generate_grid_vertices(camera, 10.0);
        self.num_grid_indices = grid_indices.len() as u32;

        if !grid_verts.is_empty() {
             self.queue.write_buffer(&self.grid_vertex_buffer, 0, bytemuck::cast_slice(&grid_verts));
             self.queue.write_buffer(&self.grid_index_buffer, 0, bytemuck::cast_slice(&grid_indices));
        }
    }

    pub fn draw(&mut self, viewport: Option<ViewportRect>) {
        let output = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(wgpu::SurfaceError::Outdated) => {
                // Ocurre durante un resize, ignoramos un frame silenciosamente.
                return;
            }
            Err(e) => {
                eprintln!("Surface error: {:?}", e);
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear({
                            let c = &self.active_theme.background;
                            // Parse hex color (simplificado, asumiendo #RRGGBB)
                            let r = u8::from_str_radix(&c[1..3], 16).unwrap_or(0) as f64 / 255.0;
                            let g = u8::from_str_radix(&c[3..5], 16).unwrap_or(0) as f64 / 255.0;
                            let b = u8::from_str_radix(&c[5..7], 16).unwrap_or(0) as f64 / 255.0;
                            wgpu::Color { r, g, b, a: 1.0 }
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            if let Some(mut vp) = viewport {
                if vp.x > self.config.width { vp.x = self.config.width; vp.width = 0; }
                if vp.y > self.config.height { vp.y = self.config.height; vp.height = 0; } 
                if vp.x + vp.width > self.config.width { vp.width = self.config.width - vp.x; }
                if vp.y + vp.height > self.config.height { vp.height = self.config.height - vp.y; }

                if vp.width > 0 && vp.height > 0 {
                    render_pass.set_viewport(vp.x as f32, vp.y as f32, vp.width as f32, vp.height as f32, 0.0, 1.0);
                    render_pass.set_scissor_rect(vp.x, vp.y, vp.width, vp.height);
                }
            }

            // 1. Draw Grid first (behind)
            if self.num_grid_indices > 0 {
                render_pass.set_pipeline(&self.grid_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.grid_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.grid_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_grid_indices, 0, 0..1);
            }

            // 2. Draw Main Geometry
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);

            // 3. Draw Ephemeral Feedback (Rubber-banding) on top
            if self.num_ephemeral_verts > 0 {
                render_pass.set_vertex_buffer(0, self.ephemeral_vertex_buffer.slice(..));
                render_pass.draw(0..self.num_ephemeral_verts, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

pub fn spawn_render_thread<W>(
    window: Arc<W>, 
    width: u32, 
    height: u32,
    world: Arc<Mutex<bevy_ecs::world::World>>,
    rx: Receiver<RenderMessage>
) 
where 
    W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static 
{
    std::thread::spawn(move || {
        let ncs_yaml = include_str!("../../fcad-assets/standards/layers/ncs_layers_A.yaml");
        let mut ncs_standards = fcad_core::infrastructure::ecs::ncs::LayerStandards::new();
        let _ = ncs_standards.load_from_yaml(ncs_yaml);
            
        let mut optimizer = optimizer::RenderOptimizer::new(ncs_standards);

        // Iniciamos con el ECS vacío de geometría de prueba
        {
            let mut _w = world.lock().unwrap();
        }

        // Sync inicial ECS -> Renderer Optimizer
        {
            let mut w = world.lock().unwrap();
            let mut added_query = w.query_filtered::<(
                bevy_ecs::entity::Entity, 
                &fcad_core::domain::Geometry, 
                Option<&fcad_core::domain::Layer>, 
                Option<&fcad_core::domain::ColorOverride>
            ), bevy_ecs::query::Added<fcad_core::domain::Geometry>>();
            
            let mut changed_query = w.query_filtered::<(
                bevy_ecs::entity::Entity, 
                &fcad_core::domain::Geometry, 
                Option<&fcad_core::domain::Layer>, 
                Option<&fcad_core::domain::ColorOverride>
            ), bevy_ecs::query::Changed<fcad_core::domain::Geometry>>();
            
            let mut deleted_query = w.query_filtered::<
                bevy_ecs::entity::Entity, 
                bevy_ecs::query::Added<fcad_core::domain::Deleted>
            >();

            optimizer.sync_system(
                added_query.iter(&w),
                changed_query.iter(&w),
                deleted_query.iter(&w)
            );
        }

        let mut renderer = pollster::block_on(Renderer::new(window, width, height, &optimizer));
        println!("Renderer initialized. Starting 60FPS loop with ECS data...");
        
        let mut current_vp: Option<ViewportRect> = None;

        loop {
            // Procesar mensajes pendientes sin bloquear
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    RenderMessage::ViewportUpdate(vp) => {
                        current_vp = Some(vp);
                    }
                    RenderMessage::WindowResize(w, h) => {
                        renderer.resize(w, h);
                    }
                    RenderMessage::UpdateTheme(theme) => {
                        renderer.update_theme(theme);
                    }
                    RenderMessage::UpdateEphemeral(vertices) => {
                        renderer.update_ephemeral(&vertices);
                    }
                }
            }

            let cam = {
                let w = world.lock().unwrap();
                w.get_resource::<fcad_core::domain::viewport::Camera>().cloned()
            };
            
            if let Some(c) = cam {
                // To avoid drawing duplicates logic could check if dirty here
                renderer.update_camera(&c);
            }

            renderer.draw(current_vp);
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
        }
    });
}
