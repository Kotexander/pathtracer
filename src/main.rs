mod math;
mod model;
mod render_pipeline;
mod renderer;
mod wgpu_context;

use math::vector3::Vector3;
use model::Model;
use render_pipeline::RenderPipeline;
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

const VERTICIES: [[f32; 2]; 4] = [[-1.0, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]];
const INDECIES: [u16; 6] = [0u16, 1, 2, 2, 3, 0];

struct State {
    ctx: wgpu_context::WgpuContext,

    model: Model,

    render_pipeline: RenderPipeline,
}
impl State {
    async fn new(window: &Window) -> Self {
        let ctx = wgpu_context::WgpuContext::new(window).await;

        let model = Model::new(&ctx.device, &VERTICIES, &INDECIES);

        let render_pipeline = RenderPipeline::new(&ctx);

        Self {
            ctx,
            model,
            render_pipeline,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.surface_config.width = width;
        self.ctx.surface_config.height = height;
        self.ctx.surface_configure();
    }
}

struct App {
    renderer: renderer::Renderer,
    state: State,

    camera_controller: CameraController,
}
impl App {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(size.width as usize, size.height as usize);
        let state = State::new(window).await;

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
        let texture = self
            .state
            .ctx
            .device
            .create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("Texture"),
            });
        self.state.ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&img.as_raw()),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self
            .state
            .ctx
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());
        let texture_bind_group =
            self.state
                .ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: self.state.render_pipeline.bind_group_layout(),
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

        let output = self.state.ctx.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.state
                .ctx
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
            render_pass.set_pipeline(self.state.render_pipeline.pipeline());
            render_pass.set_bind_group(0, &texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.state.model.vertecies().slice(..));
            render_pass.set_index_buffer(
                self.state.model.indecies().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..self.state.model.num_indecies(), 0, 0..1);
        }

        self.state
            .ctx
            .queue
            .submit(std::iter::once(encoder.finish()));
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
                    "Pathtracer: {} ms | FPS: {:.0}",
                    dt.elapsed().as_millis(),
                    1.0 / dt.elapsed().as_secs_f32()
                ));
            }
            _ => (),
        }
    });
}
