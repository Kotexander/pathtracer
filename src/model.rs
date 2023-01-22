use wgpu::util::DeviceExt;

pub struct Model {
    vertecies: wgpu::Buffer,
    indecies: wgpu::Buffer,
    num_indecies: u32,
}
impl Model {
    pub fn new(device: &wgpu::Device, vertecies: &[[f32; 2]], indecies: &[u16]) -> Self {
        let num_indecies = indecies.len() as u32;
        let vertecies = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertecies),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indecies = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indecies),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertecies,
            indecies,
            num_indecies,
        }
    }

    pub fn vertecies(&self) -> &wgpu::Buffer {
        &self.vertecies
    }

    pub fn indecies(&self) -> &wgpu::Buffer {
        &self.indecies
    }

    pub fn num_indecies(&self) -> u32 {
        self.num_indecies
    }
}
