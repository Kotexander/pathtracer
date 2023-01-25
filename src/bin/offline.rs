use pathtracer::{
    load_ron,
    renderer::{scene::Scene, *},
};

fn main() {
    let time = std::time::Instant::now();
    let ctx = pollster::block_on(WgpuContext::new());

    let width = 3840;
    let height = 2160;

    let samples = 1000;

    // load scene
    let scene: Scene = load_ron("scene.ron");
    // load settings
    let settings: Settings = load_ron("settings.ron");

    let mut renderer = Renderer::new(&ctx.device, scene, settings, width, height);

    for s in 1..=samples {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });
        renderer.render(&ctx.device, &mut encoder);
        ctx.queue.submit([encoder.finish()]);
        ctx.device.poll(wgpu::Maintain::Wait);

        print!(
            "\r{}/{} | {:.1}%                     ",
            s,
            samples,
            s as f32 / samples as f32 * 100.0
        );
    }
    println!();
    // save
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });
    let save_info = renderer.start_save(&ctx.device, &mut encoder);
    ctx.queue.submit([encoder.finish()]);

    println!(
        "Img size: {}, {}",
        save_info.tex_width(),
        save_info.tex_height()
    );
    println!("Samples: {}", samples * renderer.globals().samples);
    save_info.finish(&ctx.device, samples);

    println!("Time took: {}s", time.elapsed().as_secs())
}

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}
impl WgpuContext {
    pub async fn new() -> Self {
        // wgpu instance
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // gpu adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
