pub struct Texture {
    texture: wgpu::Texture,
    desc: wgpu::TextureDescriptor<'static>,
    view: wgpu::TextureView,
}

impl Texture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let desc = wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
        };

        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            desc,
            view,
        }
    }

    pub fn update(&mut self, device: &wgpu::Device) {
        self.texture = device.create_texture(&self.desc);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn desc(&self) -> &wgpu::TextureDescriptor<'static> {
        &self.desc
    }

    pub fn desc_mut(&mut self) -> &mut wgpu::TextureDescriptor<'static> {
        &mut self.desc
    }
}
