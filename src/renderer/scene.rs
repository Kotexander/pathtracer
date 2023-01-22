use crate::renderer::{materials::*, sphere::Sphere};

use super::camera::CameraSettings;

// #[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
// pub struct CameraSettings {
//     pub pos: Vector3,
//     /// horizontal angle in degree
//     pub yaw: f32,
//     /// vertical angle in degree
//     pub pitch: f32,
//     /// vertical in degrees
//     pub vfov: f32,
// }

// pub fn new(settings: CameraSettings, aspect: f32) -> Self {
// Self {
// pos: settings.pos,
// yaw: settings.yaw.to_radians(),
// pitch: settings.pitch.to_radians(),
// vfov: settings.vfov.to_radians(),
// aspect,
// }
// }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Scene {
    pub camera: CameraSettings,
    pub spheres: Vec<Sphere>,

    pub lights: Vec<Light>,
    pub lambertians: Vec<Lambertian>,
    pub metals: Vec<Metal>,
    pub glass: Vec<Glass>,
}
