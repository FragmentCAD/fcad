struct CameraUniform {
    view_proj: mat4x4<f32>,
    resolution: vec2<f32>,
    padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(0) start: vec2<f32>,
    @location(1) end: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) thickness: f32, // En píxeles físicos de pantalla, por defecto 1.0
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color;

    // Transformar los extremos del mundo a clip space con la cámara
    let start_clip = camera.view_proj * vec4<f32>(instance.start, 0.0, 1.0);
    let end_clip = camera.view_proj * vec4<f32>(instance.end, 0.0, 1.0);

    // Asumiendo cámara ortográfica (W = 1.0) sacamos NDC
    let start_ndc = start_clip.xy;
    let end_ndc = end_clip.xy;

    // Las proporciones pueden ser asimétricas, llevamos esto a espacio pantalla físico para la normal
    let start_screen = start_ndc * camera.resolution;
    let end_screen = end_ndc * camera.resolution;

    let dir = end_screen - start_screen;
    let dir_norm = normalize(dir);
    
    // Normal perpendicular
    let normal = vec2<f32>(-dir_norm.y, dir_norm.x);

    let half_t = instance.thickness / 2.0;

    // Calcular las 6 esquinas base para un Quad grueso a lo largo de la línea
    var pos_screen = vec2<f32>(0.0);
    switch (vertex_index) {
        case 0u: { pos_screen = start_screen - normal * half_t; }
        case 1u: { pos_screen = start_screen + normal * half_t; }
        case 2u: { pos_screen = end_screen - normal * half_t; }
        case 3u: { pos_screen = start_screen + normal * half_t; }
        case 4u: { pos_screen = end_screen - normal * half_t; }
        case 5u: { pos_screen = end_screen + normal * half_t; }
        default: {}
    }

    // Regresar al NDC normalizado de la gráfica
    let out_ndc = pos_screen / camera.resolution;
    
    out.clip_position = vec4<f32>(out_ndc, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
