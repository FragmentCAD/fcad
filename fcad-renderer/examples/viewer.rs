use std::borrow::Cow;
use std::sync::Arc;
use bytemuck::{Pod, Zeroable};
use fcad_core::domain::viewport::Camera;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    resolution: [f32; 2],
    padding: [f32; 2], // 16-byte alignment
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct LineInstance {
    start: [f32; 2],
    end: [f32; 2],
    color: [f32; 4],
    thickness: f32,
    _padding: [f32; 3], // Pad to alignment requirements
}

impl LineInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

struct ViewerApp {
    window: Option<Arc<Window>>,
    surface: Option<wgpu::Surface<'static>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    
    // Gráficos propios
    camera: Camera,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,
    pipeline: Option<wgpu::RenderPipeline>,
    instance_buffer: Option<wgpu::Buffer>,
    num_instances: u32,
}

impl Default for ViewerApp {
    fn default() -> Self {
        Self {
            window: None,
            surface: None,
            device: None,
            queue: None,
            config: None,
            camera: Camera::default(),
            camera_buffer: None,
            camera_bind_group: None,
            pipeline: None,
            instance_buffer: None,
            num_instances: 0,
        }
    }
}

impl ViewerApp {
    fn update_camera_buffer(&mut self) {
        if let (Some(queue), Some(buffer)) = (&self.queue, &self.camera_buffer) {
            let uniform = CameraUniform {
                view_proj: self.camera.build_view_projection_matrix().to_cols_array_2d(),
                resolution: [self.camera.screen_width, self.camera.screen_height],
                padding: [0.0, 0.0],
            };
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniform]));
        }
    }
}

impl ApplicationHandler for ViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attr = Window::default_attributes()
                .with_title("FragmentCAD Renderer (WGSL lines)")
                .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));
            
            let window = Arc::new(event_loop.create_window(window_attr).unwrap());
            self.window = Some(window.clone());
            
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::PRIMARY,
                ..Default::default()
            });

            let surface = instance.create_surface(window.clone()).unwrap();
            
            let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })).expect("Adapter missed");

            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    ..Default::default()
                },
            )).expect("Device failed");

            let size = window.inner_size();
            let caps = surface.get_capabilities(&adapter);
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: caps.formats[0],
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&device, &config);

            // Shader inicialización
            let shader_source = std::fs::read_to_string("shaders/lines.wgsl").unwrap();
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Lines Shader"),
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&shader_source)),
            });

            // Set up Camera Uniform Buffer
            self.camera.screen_width = size.width as f32;
            self.camera.screen_height = size.height as f32;
            let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Camera Buffer"),
                size: std::mem::size_of::<CameraUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

            let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            });

            // Render Pipeline
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                immediate_size: 0,
            });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Lines Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"), // Option is passed from v24 to None by default, in old some versions. Correct format is Option<&str>. In wgpu >v0.19 it's Option<&str>
                    compilation_options: Default::default(),
                    buffers: &[LineInstance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

            // Instances creation
            let cross_lines = [
                LineInstance {
                    start: [-100.0, -100.0],
                    end: [100.0, 100.0],
                    color: [0.2, 0.8, 0.2, 1.0], // Verde
                    thickness: 1.0,              // 1 píxel exacto y físico en la pantalla
                    _padding: [0.0; 3],
                },
                LineInstance {
                    start: [-100.0, 100.0],
                    end: [100.0, -100.0],
                    color: [0.2, 0.2, 0.8, 1.0], // Azul
                    thickness: 1.0,
                    _padding: [0.0; 3],
                },
            ];
            
            use wgpu::util::DeviceExt;
            let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lines Instance Buffer"),
                contents: bytemuck::cast_slice(&cross_lines),
                usage: wgpu::BufferUsages::VERTEX,
            });

            self.surface = Some(surface);
            self.device = Some(device);
            self.queue = Some(queue);
            self.config = Some(config);
            self.camera_buffer = Some(camera_buffer);
            self.camera_bind_group = Some(camera_bind_group);
            self.pipeline = Some(pipeline);
            self.instance_buffer = Some(instance_buffer);
            self.num_instances = cross_lines.len() as u32;

            self.update_camera_buffer();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if let Some(window) = self.window.clone() {
            if id != window.id() {
                return;
            }

            match event {
                WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                    event: KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                    ..
                } => {
                    event_loop.exit();
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    match delta {
                        MouseScrollDelta::LineDelta(_x, y) => {
                            // Aplicar Zoom dinámicamente con Scroll
                            self.camera.zoom *= 1.0 + (y * 0.1);
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            self.camera.zoom *= 1.0 + ((pos.y as f32) * 0.01);
                        }
                    }
                    if self.camera.zoom < 0.1 {
                        self.camera.zoom = 0.1; // Limit clamp
                    }
                    self.update_camera_buffer();
                    window.request_redraw();
                }
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width > 0 && physical_size.height > 0 {
                        if let (Some(config), Some(device), Some(surface)) = 
                            (&mut self.config, &self.device, &self.surface) {
                            config.width = physical_size.width;
                            config.height = physical_size.height;
                            surface.configure(device, config);
                            
                            self.camera.screen_width = physical_size.width as f32;
                            self.camera.screen_height = physical_size.height as f32;
                            self.update_camera_buffer();
                        }
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let (Some(surface), Some(device), Some(queue), Some(pipeline), Some(c_bind), Some(i_buf)) = 
                        (&self.surface, &self.device, &self.queue, &self.pipeline, &self.camera_bind_group, &self.instance_buffer) {
                        
                        let frame = surface.get_current_texture().expect("Failed next texture");
                        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

                        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

                        {
                            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Lines Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: &view,
                                    resolve_target: None,
                                    ops: wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                            r: 0.05, g: 0.05, b: 0.1, a: 1.0, 
                                        }),
                                        store: wgpu::StoreOp::Store,
                                    },
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                timestamp_writes: None,
                                occlusion_query_set: None,
                                ..Default::default()
                            });
                            
                            render_pass.set_pipeline(pipeline);
                            render_pass.set_bind_group(0, c_bind, &[]);
                            render_pass.set_vertex_buffer(0, i_buf.slice(..));
                            // Emitimos 6 vértices en base a la lógica del Shader para generar un Quad instanciado.
                            render_pass.draw(0..6, 0..self.num_instances);
                        }

                        queue.submit(std::iter::once(encoder.finish()));
                        frame.present();
                    }
                }
                _ => {}
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.clone() {
            window.request_redraw();
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    
    let mut app = ViewerApp::default();
    event_loop.run_app(&mut app)
}
