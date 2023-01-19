mod bytes;
mod camera;
mod compute_pipeline;
mod globals;
mod materials;
mod model;
mod ray;
mod render_pipeline;
mod sphere;
mod texture;
mod vector3;
mod wgpu_context;

use bytes::*;
use camera::*;
use compute_pipeline::*;
use globals::*;
use materials::*;
use model::*;
use render_pipeline::*;
use sphere::*;
use texture::*;
use vector3::*;
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

// const WHITE: Vector3 = Vector3::new(1.0, 1.0, 1.0);
// const GREY: Vector3 = Vector3::new(0.1, 0.1, 0.1);
// const BLACK: Vector3 = Vector3::new(0.0, 0.0, 0.0);

// const BLUE: Vector3 = Vector3::new(0.0, 0.0, 1.0);
// const RED: Vector3 = Vector3::new(1.0, 0.0, 0.0);
// const GREEN: Vector3 = Vector3::new(0.0, 1.0, 0.0);

fn load_ron<P, T>(path: P) -> T
where
    P: AsRef<std::path::Path>,
    T: serde::de::DeserializeOwned,
{
    let content = std::fs::read_to_string(path.as_ref()).expect(&format!(
        "the file {} should be able to be read",
        path.as_ref().to_string_lossy()
    ));
    ron::from_str::<T>(&content).expect("the formatting of the file should be correct")
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
struct Settings {
    samples: i32,
    depth: i32,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Scene {
    camera: CameraSettings,
    spheres: Vec<Sphere>,

    lights: Vec<Light>,
    lambertians: Vec<Lambertian>,
    metals: Vec<Metal>,
    glass: Vec<Glass>,
}

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
    compute_pipeline: ComputePipeline,

    model: Model,

    texture: Texture,
    sampler: wgpu::Sampler,

    spheres_buffer: wgpu::Buffer,
    lights_buffer: wgpu::Buffer,
    lambertians_buffer: wgpu::Buffer,
    metals_buffer: wgpu::Buffer,
    glass_buffer: wgpu::Buffer,
    camera_config: CameraConfig,
    camera_buffer: wgpu::Buffer,
    camera_controller: CameraController,

    globals: Globals,

    samples: i32,
    dirty: bool,
}
impl App {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let width = size.width;
        let height = size.height;

        // wgpu stuff
        let ctx = WgpuContext::new(window).await;
        let render_pipeline = RenderPipeline::new(&ctx);
        let compute_pipeline = ComputePipeline::new(&ctx.device);

        // model that fills the entire screen
        let model = Model::new(&ctx.device, &VERTICIES, &INDECIES);

        // render texture and sampler
        let texture = Texture::new(&ctx.device, width, height);
        let sampler = ctx
            .device
            .create_sampler(&wgpu::SamplerDescriptor::default());

        // load scene
        let scene: Scene = load_ron("scene.ron");
        let camera_config = CameraConfig::new(scene.camera, width as f32 / height as f32);

        // load settings
        let settings: Settings = load_ron("settings.ron");
        let globals = Globals::new(rand::random(), settings.samples, settings.depth);

        // get spheres and materials onto the gpu
        let spheres_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Spheres Buffer"),
                contents: &scene.spheres.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let lights_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: &scene.lights.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let lambertians_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lambertians Buffer"),
                contents: &scene.lambertians.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let metals_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Metals Buffer"),
                contents: &scene.metals.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let glass_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Glass Buffer"),
                contents: &scene.glass.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // get camera onto the gpu
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Config Buffer"),
                contents: &camera_config.build().bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // camera controller for real time
        let camera_controller = CameraController::new();

        let samples = -1; // number of frames accumulated
        let dirty = true; // if accumulated texture should be cleared

        Self {
            ctx,
            render_pipeline,
            compute_pipeline,
            model,
            texture,
            sampler,
            spheres_buffer,
            lights_buffer,
            lambertians_buffer,
            metals_buffer,
            glass_buffer,
            camera_config,
            camera_buffer,
            camera_controller,
            globals,
            dirty,
            samples,
        }
    }

    fn reload_scene(&mut self) {
        // reload scene
        let scene: Scene = load_ron("scene.ron");
        let camera_config = CameraConfig::new(scene.camera, self.camera_config.aspect);

        // recreate spheres and matrial buffers
        let spheres_buffer =
            self.ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Spheres Buffer"),
                    contents: &scene.spheres.bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let lights_buffer = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: &scene.lights.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let lambertians_buffer =
            self.ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Lambertians Buffer"),
                    contents: &scene.lambertians.bytes(),
                    usage: wgpu::BufferUsages::STORAGE,
                });
        let metals_buffer = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Metals Buffer"),
                contents: &scene.metals.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let glass_buffer = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Glass Buffer"),
                contents: &scene.glass.bytes(),
                usage: wgpu::BufferUsages::STORAGE,
            });

        // update app
        self.camera_config = camera_config;
        self.spheres_buffer = spheres_buffer;
        self.lights_buffer = lights_buffer;
        self.lambertians_buffer = lambertians_buffer;
        self.metals_buffer = metals_buffer;
        self.glass_buffer = glass_buffer;
        self.update_camera();
    }
    fn reload_settings(&mut self) {
        // reload settings
        let settings: Settings = load_ron("settings.ron");

        self.globals.samples = settings.samples;
        self.globals.depth = settings.depth;
    }

    fn render(&mut self) {
        // window view
        let output = self
            .ctx
            .surface
            .get_current_texture()
            .expect("the surface texture is required to present path tracing in real time");
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

        // reset renderer if dirty
        if self.dirty {
            self.dirty = false;
            self.samples = 0;
            self.clear(&mut encoder);
        }

        // accumulate one frame
        self.compute_pass(&mut encoder);
        // draw accumulated texuter
        self.render_pass(&mut encoder, &view);

        // finish frame
        self.ctx.queue.submit([encoder.finish()]);
        output.present();
    }

    fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
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
    }

    fn compute_pass(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let seed_buffer = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Global Buffer"),
                contents: &self.globals.bytes(),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = self
            .ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: self.compute_pipeline.bind_group_layout(),
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
                    // spheres
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.spheres_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    // lights
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.lights_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    // lambertians
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.lambertians_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    // metals
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.metals_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                    // glass
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &self.glass_buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(self.compute_pipeline.pipeline());
        cpass.set_bind_group(0, &bind_group, &[]);
        let t_desc = self.texture.desc();
        let width = (t_desc.size.width as f32 / 16.0).ceil() as u32;
        let height = (t_desc.size.height as f32 / 16.0).ceil() as u32;
        cpass.dispatch_workgroups(width, height, 1);

        self.globals.seed = rand::random();
        self.samples += 1;
    }

    fn render_pass(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let globals = self
            .ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Render Globals"),
                contents: &self.samples.bytes(),
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
                        resource: wgpu::BindingResource::TextureView(self.texture.view()),
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
        // context surface
        self.ctx.surface_config.width = width;
        self.ctx.surface_config.height = height;
        self.ctx.surface_configure();

        // compute shader texture
        let desc = self.texture.desc_mut();
        desc.size.width = width;
        desc.size.height = height;
        self.texture.update(&self.ctx.device);

        // camera
        self.camera_config.aspect = width as f32 / height as f32;
        self.update_camera();
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
        let mut cam_dirty = false; // if camera buffer needs to be recreated at the end
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
                // if mouse was moved from previous frame
                self.camera_config.yaw += delta[0] * sensitivity * dt;
                self.camera_config.pitch -= delta[1] * sensitivity * dt;

                // clamp pitch
                // camera breaks when looking directly up or down
                self.camera_config.pitch = self
                    .camera_config
                    .pitch
                    .clamp(-80.0f32.to_radians(), 80.0f32.to_radians());

                cam_dirty = true;
                self.camera_controller.delta_mouse = None;
            }
        }

        let cam_dir = self.camera_config.dir();
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
            self.camera_config.pos += offset * speed * dt;
            cam_dirty = true;
        }

        // update camera if neccesary
        if cam_dirty {
            self.update_camera();
        }
    }

    fn update_camera(&mut self) {
        self.camera_buffer =
            self.ctx
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera Config Buffer"),
                    contents: &self.camera_config.build().bytes(),
                    usage: wgpu::BufferUsages::UNIFORM,
                });
        self.dirty = true;
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
}

fn main() {
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
                WindowEvent::MouseInput { state, button, .. } => {
                    if let MouseButton::Right = button {
                        match state {
                            ElementState::Pressed => {
                                window.set_cursor_visible(false);
                                window
                                    .set_cursor_grab(CursorGrabMode::Locked)
                                    .or_else(|_err| {
                                        window.set_cursor_grab(CursorGrabMode::Confined)
                                    })
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
                    };
                }
                _ => {}
            },
            Event::DeviceEvent { event, .. } => {
                if let DeviceEvent::MouseMotion { delta } = event {
                    app.camera_controller.delta_mouse = Some([delta.0 as f32, delta.1 as f32]);
                }
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
