mod math;
mod renderer;

use math::vector3::Vector3;
use wgpu::util::DeviceExt;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

struct CameraController {
    forward: bool,
    backward: bool,
    strafe_left: bool,
    strafe_right: bool,
    up: bool,
    down: bool,
}
impl CameraController {
    fn new() -> Self {
        Self {
            forward: false,
            backward: false,
            strafe_left: false,
            strafe_right: false,
            up: false,
            down: false,
        }
    }
}

struct WpguContext {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    vertecies: wgpu::Buffer,
    indecies: wgpu::Buffer,
    num_indecies: u32,

    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
}
impl WpguContext {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::GL);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let vertecies = [[-1.0f32, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]];
        let indecies = [[0u16, 1, 2], [2, 3, 0]];
        let num_indecies = (indecies.len() * 3) as u32;

        let vertecies = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertecies),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indecies = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indecies),
            usage: wgpu::BufferUsages::INDEX,
        });
        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2],
        };

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Texture Bind Group Layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
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
            multiview: None,
        });

        Self {
            surface,
            config,
            device,
            queue,
            vertecies,
            indecies,
            num_indecies,
            render_pipeline,
            texture_bind_group_layout,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }
}

struct App {
    renderer: renderer::Renderer,
    state: WpguContext,

    camera_controller: CameraController,
}
impl App {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(size.width as usize, size.height as usize);
        let state = WpguContext::new(window).await;

        let camera_controller = CameraController::new();

        Self {
            renderer,
            state,
            camera_controller,
        }
    }
    fn resize(&mut self, width: usize, height: usize) {
        self.state.resize(width as u32, height as u32);
        self.renderer.resize(width, height);
    }
    fn render(&mut self) {
        self.renderer.render();

        // convert renderer's rgb32f image into rgba8 image
        let img = self.renderer.image();
        let img = image::ImageBuffer::from_fn(img.width() as u32, img.height() as u32, |x, y| {
            let p = img.get(x as usize, y as usize);
            image::Rgb([p.x, p.y, p.z])
        });
        let img = image::DynamicImage::ImageRgb32F(img).into_rgba8();

        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = self.state.device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Texture"),
        });
        self.state.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.state.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let texture_bind_group = self
            .state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &self.state.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: Some("Texture Bind Group"),
            });

        let output = self.state.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.state.render_pipeline);
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.state.vertecies.slice(..));
            render_pass.set_index_buffer(self.state.indecies.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.state.num_indecies, 0, 0..1);
        }

        self.state.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
    fn save(&self) {
        let img = self.renderer.image();
        let img = image::ImageBuffer::from_fn(img.width() as u32, img.height() as u32, |x, y| {
            let p = img.get(x as usize, y as usize);
            image::Rgb([p.x, p.y, p.z])
        });
        let img = image::DynamicImage::ImageRgb32F(img).into_rgb8();

        img.save("Renders/image.png").unwrap();
    }
    fn input(&mut self, key: &VirtualKeyCode, state: &ElementState) {
        match state {
            ElementState::Pressed => self.input_pressed(key),
            ElementState::Released => self.input_released(key),
        }
    }
    fn input_pressed(&mut self, key: &VirtualKeyCode) {
        match key {
            VirtualKeyCode::W => {
                self.camera_controller.forward = true;
            }
            VirtualKeyCode::S => {
                self.camera_controller.backward = true;
            }

            VirtualKeyCode::A => {
                self.camera_controller.strafe_left = true;
            }
            VirtualKeyCode::D => {
                self.camera_controller.strafe_right = true;
            }

            VirtualKeyCode::E => {
                self.camera_controller.up = true;
            }
            VirtualKeyCode::Q => {
                self.camera_controller.down = true;
            }

            _ => {}
        }
    }
    fn input_released(&mut self, key: &VirtualKeyCode) {
        match key {
            VirtualKeyCode::W => {
                self.camera_controller.forward = false;
            }
            VirtualKeyCode::S => {
                self.camera_controller.backward = false;
            }

            VirtualKeyCode::A => {
                self.camera_controller.strafe_left = false;
            }
            VirtualKeyCode::D => {
                self.camera_controller.strafe_right = false;
            }

            VirtualKeyCode::E => {
                self.camera_controller.up = false;
            }
            VirtualKeyCode::Q => {
                self.camera_controller.down = false;
            }

            VirtualKeyCode::Return => {
                self.save();
            }
            _ => {}
        }
    }
    fn update(&mut self, dt: f32) {
        let speed = 5.0;
        let mut dir = Vector3::ZERO;
        if self.camera_controller.forward {
            dir.z += 1.0;
        }
        if self.camera_controller.backward {
            dir.z -= 1.0;
        }
        if self.camera_controller.strafe_left {
            dir.x -= 1.0;
        }
        if self.camera_controller.strafe_right {
            dir.x += 1.0;
        }
        if self.camera_controller.up {
            dir.y += 1.0;
        }
        if self.camera_controller.down {
            dir.y -= 1.0;
        }
        if dir != Vector3::ZERO {
            dir.normalize();
            self.renderer.camera_config.ray.pos += dir * speed * dt;
            self.renderer.update_camera();
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Path Tracer")
        .build(&event_loop)
        .unwrap();

    let mut app = pollster::block_on(App::new(&window));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    let width = physical_size.width as usize;
                    let height = physical_size.height as usize;
                    app.resize(width, height);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    let width = new_inner_size.width as usize;
                    let height = new_inner_size.height as usize;
                    app.resize(width, height);
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(vkeycode),
                            state,
                            ..
                        },
                    ..
                } => {
                    app.input(&vkeycode, &state);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                let dt = std::time::Instant::now();
                app.render();
                app.update(dt.elapsed().as_secs_f32());
                // app.update(0.01);
                window.set_title(&format!(
                    "Pathtracer: {} ms FPS: {:.0}",
                    dt.elapsed().as_millis(),
                    1.0 / dt.elapsed().as_secs_f32()
                ));
            }
            _ => (),
        }
    });
}
