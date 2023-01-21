use camera::CameraSettings;
use sphere::Sphere;
use materials::*;

pub mod bytes;
pub mod camera;
pub mod compute_pipeline;
pub mod globals;
pub mod materials;
pub mod model;
pub mod ray;
pub mod render_pipeline;
pub mod sphere;
pub mod texture;
pub mod vector3;
pub mod wgpu_context;

pub fn load_ron<P, T>(path: P) -> T
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
pub fn save_ron<P>(path: P, scene: &Scene)
where
    P: AsRef<std::path::Path>,
{
    let content = ron::ser::to_string_pretty(scene, ron::ser::PrettyConfig::default()).unwrap();
    std::fs::write(path, content).unwrap();
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    pub camera: CameraSettings,
    pub spheres: Vec<Sphere>,

    pub lights: Vec<Light>,
    pub lambertians: Vec<Lambertian>,
    pub metals: Vec<Metal>,
    pub glass: Vec<Glass>,
}