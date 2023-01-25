pub mod bounding_box;
pub mod bvh;
pub mod bytes;
pub mod camera;
pub mod compute_pipeline;
pub mod globals;
pub mod materials;
pub mod ray;
pub mod scene;
pub mod sphere;
pub mod texture;
pub mod vector3;

use camera::CameraConfig;
use compute_pipeline::ComputePipeline;
use globals::Globals;
use wgpu::util::DeviceExt;

use self::{
    bvh::{flatten, BVHTree},
    bytes::Bytes,
    scene::Scene,
    texture::Texture,
};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub samples: i32,
    pub depth: i32,
}

struct SceneBuffers {
    // spheres
    spheres_buffer: wgpu::Buffer,

    // materials
    lights_buffer: wgpu::Buffer,
    lambertians_buffer: wgpu::Buffer,
    metals_buffer: wgpu::Buffer,
    glass_buffer: wgpu::Buffer,
    bvh_buffer: wgpu::Buffer,
}
impl SceneBuffers {
    fn new(device: &wgpu::Device, scene: Scene) -> Self {
        // get spheres onto the gpu
        let spheres_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Spheres Buffer"),
            contents: &scene.spheres.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        // get materials onto the gpu
        let lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights Buffer"),
            contents: &scene.lights.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let lambertians_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lambertians Buffer"),
            contents: &scene.lambertians.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let metals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Metals Buffer"),
            contents: &scene.metals.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let glass_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Glass Buffer"),
            contents: &scene.glass.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let bvh_scene = flatten(BVHTree::new(scene.spheres));
        let bvh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BVH Buffer"),
            contents: &bvh_scene.bytes(),
            usage: wgpu::BufferUsages::STORAGE,
        });

        Self {
            spheres_buffer,
            lights_buffer,
            lambertians_buffer,
            metals_buffer,
            glass_buffer,
            bvh_buffer,
        }
    }
}

pub struct Renderer {
    compute_pipeline: ComputePipeline,

    scene_buffers: SceneBuffers,
    camera_buffer: wgpu::Buffer,
    camera_config: CameraConfig,
    scene_bind_group: wgpu::BindGroup,
    globals: Globals,

    texture: Texture,

    samples: i32,
    dirty: bool,
}
impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        scene: Scene,
        settings: Settings,
        width: u32,
        height: u32,
    ) -> Self {
        let compute_pipeline = ComputePipeline::new(device);

        let camera_config = CameraConfig::new(scene.camera, width as f32 / height as f32);

        let scene_buffers = SceneBuffers::new(device, scene);

        // get camera onto the gpu
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Config Buffer"),
            contents: &camera_config.build().bytes(),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let scene_bind_group = make_scene_bind_group(device, &compute_pipeline, &scene_buffers);

        let globals = Globals::new(rand::random(), settings.samples, settings.depth);

        let texture = Texture::new(device, width, height);

        let samples = -1;
        let dirty = true;

        Self {
            compute_pipeline,
            scene_buffers,
            camera_buffer,
            camera_config,
            scene_bind_group,
            globals,
            texture,
            samples,
            dirty,
        }
    }

    pub fn reload_scene(&mut self, device: &wgpu::Device, scene: Scene) {
        self.camera_config = CameraConfig::new(scene.camera, self.camera_config.aspect);
        self.scene_buffers = SceneBuffers::new(device, scene);

        self.scene_bind_group =
            make_scene_bind_group(device, &self.compute_pipeline, &self.scene_buffers);
        self.dirty = true;
    }

    pub fn reload_settings(&mut self, settings: &Settings) {
        self.globals.samples = settings.samples;
        self.globals.depth = settings.depth;
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.texture.resize(device, width, height);
        self.camera_config.aspect = width as f32 / height as f32;
        self.dirty = true;
    }

    pub fn render(&mut self, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if self.dirty {
            self.camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Config Buffer"),
                contents: &self.camera_config.build().bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.texture.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.dirty = false;
            self.samples = 0;
        }

        let seed_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global Buffer"),
            contents: &self.globals.bytes(),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let main_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: self.compute_pipeline.main_bind_group_layout(),
            entries: &[
                // output texture
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture.view()),
                },
                // camera
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.camera_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                // globals
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &seed_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(self.compute_pipeline.pipeline());
        cpass.set_bind_group(0, &main_bind_group, &[]);
        cpass.set_bind_group(1, &self.scene_bind_group, &[]);
        let t_desc = self.texture.desc();
        let width = (t_desc.size.width as f32 / 16.0).ceil() as u32;
        let height = (t_desc.size.height as f32 / 16.0).ceil() as u32;
        cpass.dispatch_workgroups(width, height, 1);

        self.globals.seed = rand::random();
        self.samples += 1;
    }

    pub fn samples(&self) -> i32 {
        self.samples
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn camera_config_mut(&mut self) -> &mut CameraConfig {
        self.dirty = true;
        &mut self.camera_config
    }

    pub fn camera_config(&self) -> CameraConfig {
        self.camera_config
    }

    pub fn start_save(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> SaveInfo {
        use std::num::NonZeroU32;
        let tex_desc = self.texture.desc();
        let tex_width = tex_desc.size.width;
        let tex_height = tex_desc.size.height;
        // wgpu requires texture -> buffer copies to be aligned using
        // wgpu::COPY_BYTES_PER_ROW_ALIGNMENT. Because of this we'll
        // need to save both the padded_bytes_per_row as well as the
        // unpadded_bytes_per_row
        let pixel_size = std::mem::size_of::<[f32; 4]>() as u32;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unpadded_bytes_per_row = pixel_size * tex_width;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padding;

        let output_buffer_desc = wgpu::BufferDescriptor {
            size: (padded_bytes_per_row * tex_height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            label: None,
            mapped_at_creation: false,
        };
        let buffer = device.create_buffer(&output_buffer_desc);
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: self.texture.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new(padded_bytes_per_row).unwrap()),
                    rows_per_image: None,
                    // rows_per_image: Some(NonZeroU32::new(tex_height).unwrap()),
                },
            },
            tex_desc.size,
        );

        SaveInfo {
            buffer,
            padded: padded_bytes_per_row,
            unpadded: unpadded_bytes_per_row,
            tex_width,
            tex_height,
        }
    }

    pub fn globals(&self) -> Globals {
        self.globals
    }
}

fn make_scene_bind_group(
    device: &wgpu::Device,
    pipeline: &ComputePipeline,
    scene: &SceneBuffers,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: pipeline.scene_bind_group_layout(),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.bvh_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // spheres
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.spheres_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // lights
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.lights_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // lambertians
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.lambertians_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // metals
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.metals_buffer,
                    offset: 0,
                    size: None,
                }),
            },
            // glass
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &scene.glass_buffer,
                    offset: 0,
                    size: None,
                }),
            },
        ],
    })
}

pub struct SaveInfo {
    buffer: wgpu::Buffer,
    padded: u32,
    unpadded: u32,
    tex_width: u32,
    tex_height: u32,
}
impl SaveInfo {
    pub fn finish(self, device: &wgpu::Device, samples: i32) {
        let buffer_slice = self.buffer.slice(..);

        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::Maintain::Wait);
        let _ = rx.recv().unwrap();

        let padded_data = buffer_slice.get_mapped_range();
        let data = padded_data
            .chunks(self.padded as _)
            .flat_map(|chunk| &chunk[..self.unpadded as _])
            .copied()
            .collect::<Vec<_>>();

        let mut img = image::Rgba32FImage::from_raw(
            self.tex_width,
            self.tex_height,
            bytemuck::cast_slice(&data).to_vec(),
        )
        .unwrap();

        let gamma = 1.0 / 2.2;
        let samples = 1.0 / samples as f32;
        for p in img.pixels_mut() {
            p[0] *= samples;
            p[1] *= samples;
            p[2] *= samples;

            p[0] = p[0].powf(gamma);
            p[1] = p[1].powf(gamma);
            p[2] = p[2].powf(gamma);
        }
        let img = image::DynamicImage::ImageRgba32F(img);
        let img = img.to_rgb8();

        img.save("img.png").unwrap();
    }

    pub fn tex_width(&self) -> u32 {
        self.tex_width
    }

    pub fn tex_height(&self) -> u32 {
        self.tex_height
    }
}
