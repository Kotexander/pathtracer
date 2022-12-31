mod compute_pipeline;
// mod math;
mod model;
mod render_pipeline;
// mod renderer;
mod texture;
mod wgpu_context;

use compute_pipeline::ComputePipeline;
// use math::vector3::Vector3;
use model::Model;
use render_pipeline::RenderPipeline;
use texture::Texture;
use wgpu_context::WgpuContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// struct CameraController {
//     forward: bool,
//     backward: bool,
//     strafe_left: bool,
//     strafe_right: bool,
//     up: bool,
//     down: bool,
// }
// impl CameraController {
//     fn new() -> Self {
//         Self {
//             forward: false,
//             backward: false,
//             strafe_left: false,
//             strafe_right: false,
//             up: false,
//             down: false,
//         }
//     }
// }

const VERTICIES: [[f32; 2]; 4] = [[-1.0, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]];
const INDECIES: [u16; 6] = [0, 1, 2, 2, 3, 0];

struct App {
    ctx: WgpuContext,
    render_pipeline: RenderPipeline,
    compute_pipeline: ComputePipeline,

    model: Model,

    texture: Texture,
    sampler: wgpu::Sampler,
    // camera_controller: CameraController,
}
impl App {
    async fn new(window: &Window) -> Self {
        let ctx = WgpuContext::new(window).await;
        let render_pipeline = RenderPipeline::new(&ctx);
        let compute_pipeline = ComputePipeline::new(&ctx.device);

        let model = Model::new(&ctx.device, &VERTICIES, &INDECIES);

        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        let texture = Texture::new(&ctx.device, width, height);

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // let camera_controller = CameraController::new();
        Self {
            ctx,
            render_pipeline,
            compute_pipeline,
            model,
            texture,
            sampler, // camera_controller,
        }
    }
    fn render(&mut self) {
        let output = self.ctx.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        self.compute_pass(&mut encoder);
        self.render_pass(&mut encoder, &view);

        self.ctx.queue.submit([encoder.finish()]);
        output.present();
    }

    fn compute_pass(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let bind_group = self
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: self.compute_pipeline.bind_group_layout(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(self.texture.view()),
                }],
            });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(self.compute_pipeline.pipeline());
        cpass.set_bind_group(0, &bind_group, &[]);
        let t_desc = self.texture.desc();
        let width = (t_desc.size.width as f32 / 16.0).ceil() as u32;
        let height = (t_desc.size.height as f32 / 16.0).ceil() as u32;
        cpass.dispatch_workgroups(width, height, 1);
    }

    fn render_pass(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // let mut encoder = self
        // .ctx
        // .device
        // .create_command_encoder(&wgpu::CommandEncoderDescriptor {
        // label: Some("Render Encoder"),
        // });

        let bind_group = self
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Render Bind Group"),
                layout: self.render_pipeline.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(self.texture.view()),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: view,
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
            render_pass.set_pipeline(self.render_pipeline.pipeline());
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.model.vertecies().slice(..));
            render_pass
                .set_index_buffer(self.model.indecies().slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.model.num_indecies(), 0, 0..1);
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ctx.surface_config.width = width;
        self.ctx.surface_config.height = height;
        self.ctx.surface_configure();

        let desc = self.texture.desc_mut();
        desc.size.width = width;
        desc.size.height = height;
        self.texture.update(&self.ctx.device);
    }

    // fn save(&self) {
    // let img = self.renderer.image();
    // let img = image::ImageBuffer::from_fn(img.width() as u32, img.height() as u32, |x, y| {
    // let p = img.get(x as usize, y as usize);
    // image::Rgb([p.x, p.y, p.z])
    // });
    // let img = image::DynamicImage::ImageRgb32F(img).into_rgb8();

    // img.save("Renders/image.png").unwrap();
    // }

    // fn input(&mut self, key: &VirtualKeyCode, state: &ElementState) {
    //     let state = matches!(state, ElementState::Pressed);
    //     match key {
    //         VirtualKeyCode::W => {
    //             self.camera_controller.forward = state;
    //         }
    //         VirtualKeyCode::S => {
    //             self.camera_controller.backward = state;
    //         }

    //         VirtualKeyCode::A => {
    //             self.camera_controller.strafe_left = state;
    //         }
    //         VirtualKeyCode::D => {
    //             self.camera_controller.strafe_right = state;
    //         }

    //         VirtualKeyCode::E => {
    //             self.camera_controller.up = state;
    //         }
    //         VirtualKeyCode::Q => {
    //             self.camera_controller.down = state;
    //         }
    //         _ => {}
    //     }
    // }

    // fn update(&mut self, dt: f32) {
    //     let speed = 5.0;
    //     let mut dir = Vector3::ZERO;
    //     if self.camera_controller.forward {
    //         dir.z += 1.0;
    //     }
    //     if self.camera_controller.backward {
    //         dir.z -= 1.0;
    //     }
    //     if self.camera_controller.strafe_left {
    //         dir.x -= 1.0;
    //     }
    //     if self.camera_controller.strafe_right {
    //         dir.x += 1.0;
    //     }
    //     if self.camera_controller.up {
    //         dir.y += 1.0;
    //     }
    //     if self.camera_controller.down {
    //         dir.y -= 1.0;
    //     }
    //     if dir != Vector3::ZERO {
    //         dir.normalize();
    //         self.renderer.camera_config.ray.pos += dir * speed * dt;
    //         self.renderer.update_camera();
    //     }
    // }
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
                    let width = physical_size.width;
                    let height = physical_size.height;
                    app.resize(width, height);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    let width = new_inner_size.width;
                    let height = new_inner_size.height;
                    app.resize(width, height);
                }
                // WindowEvent::KeyboardInput {
                //     input:
                //         KeyboardInput {
                //             virtual_keycode: Some(vkeycode),
                //             state,
                //             ..
                //         },
                //     ..
                // } => {
                //     // app.input(&vkeycode, &state);
                // }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                let dt = std::time::Instant::now();
                app.render();
                // app.update(dt.elapsed().as_secs_f32());
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
