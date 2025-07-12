//! Fullscreen quad

#[allow(unused)]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

#[allow(unused)]
impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

// TODO: try a single tri
#[allow(unused)]
#[rustfmt::skip]
const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0,  1.0,  0.0] },
    Vertex { position: [-1.0, -1.0,  0.0] },
    Vertex { position: [ 1.0, -1.0,  0.0] },
    Vertex { position: [ 1.0,  1.0,  0.0] },
];

#[allow(unused)]
#[rustfmt::skip]
const INDICES: &[u16] = &[
    0, 1, 3,
    1, 2, 3,
];
