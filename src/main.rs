#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use model::*;
use pathtracer::{
    renderer::{bytes::Bytes, vector3::*},
    *,
};
use render_pipeline::*;
use renderer::*;
use scene::*;
use wgpu::util::DeviceExt;
use wgpu_context::*;

use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{CursorGrabMode, Window, WindowBuilder},
};

const VERTICIES: [[f32; 2]; 4] = [[-1.0, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]];
const INDECIES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Clone, Copy, Debug)]
struct CameraController {
    forward: bool,
    backward: bool,
    strafe_left: bool,
    strafe_right: bool,
    up: bool,
    down: bool,

    shift_down: bool,
    ctrl_down: bool,

    right_down: bool,
    delta_mouse: Option<[f32; 2]>,
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

            shift_down: false,
            ctrl_down: false,

            right_down: false,
            delta_mouse: None,
        }
    }
}

struct App {
    ctx: WgpuContext,

    render_pipeline: RenderPipeline,

    renderer: Renderer,

    model: Model,

    sampler: wgpu::Sampler,

    camera_controller: CameraController,

    save_next_frame: bool,
}
impl App {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        // wgpu stuff
        let ctx = WgpuContext::new(window).await;
        let render_pipeline = RenderPipeline::new(&ctx);

        // model that fills the entire screen
        let model = Model::new(&ctx.device, &VERTICIES, &INDECIES);

        // sampler
        let sampler = ctx
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());

        // load scene
        let scene: Scene = load_ron("scene.ron");
        // load settings
        let settings: Settings = load_ron("settings.ron");

        let renderer = Renderer::new(&ctx.device, scene, settings, width, height);

        // camera controller for real time
        let camera_controller = CameraController::new();

        let save_next_frame = false;

        Self {
            ctx,
            render_pipeline,
            renderer,
            model,
            sampler,
            camera_controller,
            save_next_frame,
        }
    }

    fn reload_scene(&mut self) {
        let scene: Scene = load_ron("scene.ron");
        self.renderer.reload_scene(&self.ctx.device, &scene);
    }
    fn reload_settings(&mut self) {
        let settings: Settings = load_ron("settings.ron");
        self.renderer.reload_settings(&settings);
    }

    fn render(&mut self) {
        // window view
        let output = match self.ctx.surface.get_current_texture() {
            Ok(output) => output,
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Lost => {
                        self.ctx.surface_configure();
                    }
                    wgpu::SurfaceError::OutOfMemory => panic!("Out of memory"),
                    _ => {
                        println!("{e:?}")
                    }
                }
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // begin encoding
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

        // accumulate one frame
        self.renderer.render(&self.ctx.device, &mut encoder);
        // draw accumulated texture
        self.render_pass(&mut encoder, &view);

        let mut save_info = None;
        if self.save_next_frame {
            save_info = Some(self.renderer.start_save(&self.ctx.device, &mut encoder));
            self.save_next_frame = false;
        }

        // finish frame
        self.ctx.queue.submit([encoder.finish()]);
        output.present();

        if let Some(save_info) = save_info {
            let buffer_slice = save_info.buffer.slice(..);

            let (tx, rx) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            self.ctx.device.poll(wgpu::Maintain::Wait);
            let _ = rx.recv().unwrap();

            // let data = buffer_slice.get_mapped_range();
            let padded_data = buffer_slice.get_mapped_range();
            let data = padded_data
                .chunks(save_info.padded as _)
                .flat_map(|chunk| &chunk[..save_info.unpadded as _])
                .copied()
                .collect::<Vec<_>>();

            let mut img = image::Rgba32FImage::from_raw(
                save_info.tex_width,
                save_info.tex_height,
                bytemuck::cast_slice(&data).to_vec(),
            )
            .unwrap();

            let samples = self.renderer.samples() as f32;
            for p in img.pixels_mut() {
                p[0] /= samples;
                p[1] /= samples;
                p[2] /= samples;

                let gamma = 1.0 / 2.2;
                p[0] = p[0].powf(gamma);
                p[1] = p[1].powf(gamma);
                p[2] = p[2].powf(gamma);
            }
            let img = image::DynamicImage::ImageRgba32F(img);
            let img = img.to_rgba8();

            println!("Img size: {}, {}", img.width(), img.height());
            println!(
                "Samples: {}",
                self.renderer.samples() * self.renderer.globals().samples
            );

            img.save("img.png").unwrap();
        }
    }

    fn render_pass(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let globals = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Render Globals"),
                contents: &self.renderer.samples().bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = self
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Render Bind Group"),
                layout: self.render_pipeline.bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            self.renderer.texture().view(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &globals,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
        // context surface
        self.ctx.surface_config.width = width;
        self.ctx.surface_config.height = height;
        self.ctx.surface_configure();

        self.renderer.resize(&self.ctx.device, width, height);
    }

    fn input(&mut self, key: &VirtualKeyCode, state: &ElementState) {
        let is_down = matches!(state, ElementState::Pressed);
        match key {
            VirtualKeyCode::W => {
                self.camera_controller.forward = is_down;
            }
            VirtualKeyCode::S => {
                self.camera_controller.backward = is_down;
            }

            VirtualKeyCode::A => {
                self.camera_controller.strafe_left = is_down;
            }
            VirtualKeyCode::D => {
                self.camera_controller.strafe_right = is_down;
            }

            VirtualKeyCode::LShift => {
                self.camera_controller.shift_down = is_down;
            }
            VirtualKeyCode::LControl => {
                self.camera_controller.ctrl_down = is_down;
            }

            VirtualKeyCode::E => {
                self.camera_controller.up = is_down;
            }
            VirtualKeyCode::Q => {
                self.camera_controller.down = is_down;
            }
            _ => {}
        }

        if let ElementState::Pressed = state {
            match key {
                VirtualKeyCode::Z => {
                    self.save_next_frame = true;
                }
                VirtualKeyCode::R => {
                    self.reload_scene();
                }
                VirtualKeyCode::F => {
                    self.reload_settings();
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, dt: f32) {
        let sensitivity = 0.5; // mouse sensitivity

        // movement speed
        let mut speed = 5.0;
        if self.camera_controller.shift_down {
            // shift to speed up
            speed *= 2.0;
        }
        if self.camera_controller.ctrl_down {
            // ctrl to slow down
            speed /= 2.0;
        }

        if self.camera_controller.right_down {
            // if right mouse button is down
            if let Some(delta) = self.camera_controller.delta_mouse {
                let camera_config = self.renderer.camera_config_mut();
                // if mouse was moved from previous frame
                camera_config.yaw += delta[0] * sensitivity * dt;
                camera_config.pitch -= delta[1] * sensitivity * dt;

                // clamp pitch
                // camera breaks when looking directly up or down
                camera_config.pitch = camera_config
                    .pitch
                    .clamp(-80.0f32.to_radians(), 80.0f32.to_radians());

                self.camera_controller.delta_mouse = None;
            }
        }

        let cam_dir = self.renderer.camera_config().dir();
        let cam_right = cross(&camera::UP, &cam_dir);

        let mut offset = Vector3::ZERO;
        if self.camera_controller.forward {
            offset += cam_dir;
        }
        if self.camera_controller.backward {
            offset -= cam_dir;
        }
        if self.camera_controller.strafe_left {
            offset -= cam_right;
        }
        if self.camera_controller.strafe_right {
            offset += cam_right;
        }
        if self.camera_controller.up {
            offset.y += 1.0;
        }
        if self.camera_controller.down {
            offset.y -= 1.0;
        }
        if offset != Vector3::ZERO {
            offset.normalize();
            self.renderer.camera_config_mut().pos += offset * speed * dt;
        }
    }
}

fn main() {
    #[cfg(debug_assertions)]
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Path Tracer")
        .build(&event_loop)
        .expect("a window is required for the path tracer to work");

    let mut app = pollster::block_on(App::new(&window));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

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
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Right,
                    ..
                } => {
                    match state {
                        ElementState::Pressed => {
                            window.set_cursor_visible(false);
                            window
                                .set_cursor_grab(CursorGrabMode::Locked)
                                .or_else(|_err| window.set_cursor_grab(CursorGrabMode::Confined))
                                .expect("the cursor should be able to lock");
                            app.camera_controller.right_down = true;
                        }
                        ElementState::Released => {
                            window
                                .set_cursor_grab(CursorGrabMode::None)
                                .expect("the cursor should be able to unlock");
                            window.set_cursor_visible(true);
                            app.camera_controller.right_down = false;
                        }
                    };
                }
                _ => {}
            },
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                app.camera_controller.delta_mouse = Some([delta.0 as f32, delta.1 as f32]);
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                let dt = std::time::Instant::now();
                app.render();
                app.update(dt.elapsed().as_secs_f32());
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
