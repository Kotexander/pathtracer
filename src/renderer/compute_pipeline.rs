const MAIN_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Main Compute Bind Group Layout"),
        entries: &[
            // output texture
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::ReadWrite,
                    format: wgpu::TextureFormat::Rgba32Float,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
            // camera
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // globals
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };
const SCENE_BIND_GROUP_LAYOUT_DESC: wgpu::BindGroupLayoutDescriptor =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Scene Compute Bind Group Layout"),
        entries: &[
            // bvh
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // spheres
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // lights
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // lambertians
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // metals
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // glass
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    };

pub struct ComputePipeline {
    pipeline: wgpu::ComputePipeline,
    main_bind_group_layout: wgpu::BindGroupLayout,
    scene_bind_group_layout: wgpu::BindGroupLayout,
}
impl ComputePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        #[cfg(not(debug_assertions))]
        let shader = device.create_shader_module(wgpu::include_wgsl!("compute.wgsl"));

        #[cfg(debug_assertions)]
        let shader = {
            let file = std::fs::read_to_string("src/renderer/compute.wgsl").unwrap();
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Compute Shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::from(file)),
            })
        };

        let main_bind_group_layout = device.create_bind_group_layout(&MAIN_BIND_GROUP_LAYOUT_DESC);
        let scene_bind_group_layout =
            device.create_bind_group_layout(&SCENE_BIND_GROUP_LAYOUT_DESC);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&main_bind_group_layout, &scene_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        Self {
            pipeline,
            main_bind_group_layout,
            scene_bind_group_layout,
        }
    }

    pub fn pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
    pub fn main_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.main_bind_group_layout
    }

    pub fn scene_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.scene_bind_group_layout
    }
}
