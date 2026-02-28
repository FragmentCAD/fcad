use bytemuck::{Pod, Zeroable};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};

/// Vértice genérico de geometría compleja rellenada u delineada por Lyon
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, PartialEq)]
pub struct TessVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl TessVertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct GeometryTessellator {
    fill_tessellator: FillTessellator,
}

impl Default for GeometryTessellator {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometryTessellator {
    pub fn new() -> Self {
        Self {
            fill_tessellator: FillTessellator::new(),
        }
    }

    /// Construye una "Dona" (Donut/Anulus) 2D y la tesela matemáticamente pasando la información a un buffer wgpu.
    /// center: Centro del objeto en coordinadas World
    /// outer_radius: Radio del borde exterior
    /// inner_radius: Radio del hueco interior
    /// color: RGBA para pintar los triangulos resultantes
    pub fn tessellate_donut(
        &mut self,
        center: [f32; 2],
        outer_radius: f32,
        inner_radius: f32,
        color: [f32; 4],
    ) -> Result<VertexBuffers<TessVertex, u16>, lyon::tessellation::TessellationError> {
        let mut buffers: VertexBuffers<TessVertex, u16> = VertexBuffers::new();

        // 1. Construir la ruta geométrica de la dona (Path en lyon)
        let mut builder = Path::builder();
        
        let c = point(center[0], center[1]);
        
        // Círculo exterior (agregarlo en sentido horario)
        builder.add_circle(c, outer_radius, lyon::path::Winding::Positive);
        
        // Círculo interior (agregarlo en sentido anti-horario para crear el hueco subtrativo)
        builder.add_circle(c, inner_radius, lyon::path::Winding::Negative);
        
        let path = builder.build();

        // 2. Opciones de Relleno. Ojo: La dona usa regla EvenOdd o NonZero para calar el centro.
        let mut options = FillOptions::default();
        options.tolerance = 0.5; // Precisión sub-píxel para los arcos

        // 3. Ejecutar teselación
        self.fill_tessellator.tessellate_path(
            &path,
            &options,
            &mut BuffersBuilder::new(&mut buffers, |vertex: FillVertex| {
                TessVertex {
                    position: vertex.position().to_array(),
                    color, // Todos los vértices generados llevan el color solictado
                }
            }),
        )?;

        Ok(buffers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tessellate_donut_creates_mesh() {
        let mut tessellator = GeometryTessellator::new();
        
        // Dona de prueba: Centro Origen, Radio Exterior 100, Radio Interior 50
        let color = [1.0, 0.5, 0.0, 1.0]; // Naranja
        let result = tessellator.tessellate_donut([0.0, 0.0], 100.0, 50.0, color);
        
        assert!(result.is_ok(), "Falló la teselación de polígonos cóncavos/complejos");
        let math_buffer = result.unwrap();
        
        // Constatamos matemáticamente que la malla se generó
        assert!(!math_buffer.vertices.is_empty(), "La malla no devió producir 0 vértices");
        assert!(!math_buffer.indices.is_empty(), "La malla no devió producir 0 índices");
        
        // Comprobar propagación del color a cada vértice del motor
        for v in math_buffer.vertices.iter() {
            assert_eq!(v.color, color);
        }
        
        // Cada triángulo se define por 3 u16.
        assert_eq!(math_buffer.indices.len() % 3, 0, "Los índices no son múltiplos puros trigonométricos (3)");
    }
}
