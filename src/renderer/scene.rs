use crate::renderer::{materials::*, sphere::Sphere};

use super::{camera::CameraSettings, vector3::Vector3};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    pub camera: CameraSettings,
    pub spheres: Vec<Sphere>,

    pub lights: Vec<Light>,
    pub lambertians: Vec<Lambertian>,
    pub metals: Vec<Metal>,
    pub glass: Vec<Glass>,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            camera: Default::default(),
            spheres: vec![
                Sphere::new(Vector3::X, 1.0, indecies::LAMBERTIAN, 0),
                Sphere::new(-Vector3::X, 1.0, indecies::LIGHT, 0),
                Sphere::new(Vector3::Y, 1.0, indecies::GLASS, 0),
                Sphere::new(-Vector3::Y, 1.0, indecies::METAL, 0),
            ],
            lights: vec![Light::default()],
            lambertians: vec![Lambertian::default()],
            metals: vec![Metal::default()],
            glass: vec![Glass::default()],
        }
    }
}
