use pathtracer::{
    renderer::{
        camera::CameraSettings, materials::*, scene::Scene, sphere::Sphere, vector3::Vector3,
    },
    save_ron,
};
use rand::Rng;

fn gen_scene() -> Scene {
    let mut rng = rand::thread_rng();

    let look_from = Vector3::new(13.0, 2.0, 3.0);
    let look_at = Vector3::ZERO;
    let dir = (look_at - look_from).normal();
    let pitch = dir.y.asin().to_degrees();
    let yaw = dir.x.atan2(dir.z).to_degrees();

    let camera = CameraSettings {
        pos: look_from,
        yaw,
        pitch,
        vfov: 20.0,
    };

    let mut spheres = vec![];

    let lights = vec![Light::default()];
    let mut lambertians = vec![];
    let mut metals = vec![];
    let mut glass = vec![];

    // ground
    spheres.push(Sphere::new(
        Vector3::Y * -1000.0,
        1000.0,
        indecies::LAMBERTIAN,
        lambertians.len() as u32,
    ));
    lambertians.push(Lambertian::new(Vector3::new(0.5, 0.5, 0.5)));

    // center - glass
    spheres.push(Sphere::new(
        Vector3::new(0.0, 1.0, 0.0),
        1.0,
        indecies::GLASS,
        glass.len() as u32,
    ));
    glass.push(Glass::new(1.5));

    // left - lambertian
    spheres.push(Sphere::new(
        Vector3::new(-4.0, 1.0, 0.0),
        1.0,
        indecies::LAMBERTIAN,
        lambertians.len() as u32,
    ));
    lambertians.push(Lambertian::new(Vector3::new(0.4, 0.2, 0.1)));

    // right - metal
    spheres.push(Sphere::new(
        Vector3::new(4.0, 1.0, 0.0),
        1.0,
        indecies::METAL,
        metals.len() as u32,
    ));
    metals.push(Metal::new(Vector3::new(0.7, 0.6, 0.5), 0.0));

    let size = 5;
    for a in -size..size {
        for b in -size..size {
            let mat: f32 = rng.gen();

            let pos = Vector3::new(
                a as f32 + 0.9 * rng.gen::<f32>(),
                0.2,
                b as f32 + 0.9 * rng.gen::<f32>(),
            );
            if pos.length() < size as f32 {
                if mat < 0.25 {
                    let albedo = Vector3::new(
                        rng.gen::<f32>() * rng.gen::<f32>(),
                        rng.gen::<f32>() * rng.gen::<f32>(),
                        rng.gen::<f32>() * rng.gen::<f32>(),
                    );
                    spheres.push(Sphere::new(
                        pos,
                        0.2,
                        indecies::LAMBERTIAN,
                        lambertians.len() as u32,
                    ));
                    lambertians.push(Lambertian::new(albedo));
                } else if mat < 0.5 {
                    let albedo = Vector3::new(
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                        rng.gen_range(0.5..1.0),
                    );
                    let roughness = rng.gen_range(0.0..0.5);
                    spheres.push(Sphere::new(pos, 0.2, indecies::METAL, metals.len() as u32));
                    metals.push(Metal::new(albedo, roughness));
                } else if mat < 0.75 {
                    spheres.push(Sphere::new(pos, 0.2, indecies::GLASS, glass.len() as u32));
                    glass.push(Glass::new(rng.gen_range(1.0..4.0)));
                } else {
                    spheres.push(Sphere::new(pos, 0.2, indecies::LIGHT, 0));
                }
            }
        }
    }

    Scene {
        camera,
        spheres,
        lights,
        lambertians,
        metals,
        glass,
    }
}

fn main() {
    let scene = gen_scene();
    save_ron("scene.ron", &scene);
}
