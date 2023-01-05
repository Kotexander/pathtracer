use crate::{bytes::Bytes, ray::Ray, vector3::*};

#[derive(Clone, Copy, Debug)]
pub struct CameraConfig {
    pub ray: Ray,
    pub vfov: f32,
    pub aspect: f32,
}
impl CameraConfig {
    pub fn new(ray: Ray, vfov: f32, aspect: f32) -> Self {
        Self { ray, vfov, aspect }
    }
    pub fn build(&self) -> Camera {
        Camera::new(self)
    }
}
const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pos: Vector3,

    horizontal: Vector3,
    vertical: Vector3,
    center: Vector3,
}
impl Camera {
    pub fn new(config: &CameraConfig) -> Self {
        let h = (config.vfov / 2.0).tan();
        let viewport = h * 2.0;

        let h = cross(&UP, &config.ray.dir);
        let v = cross(&config.ray.dir, &h);

        let horizontal = h * viewport * config.aspect;
        let vertical = v * viewport;

        let center = config.ray.pos + config.ray.dir;

        Self {
            pos: config.ray.pos,
            horizontal,
            vertical,
            center,
        }
    }
}
impl Bytes for Camera {
    fn bytes(&self) -> Vec<u8> {
        let b_p = bytemuck::bytes_of(&self.pos);
        let b_h = bytemuck::bytes_of(&self.horizontal);
        let b_v = bytemuck::bytes_of(&self.vertical);
        let b_c = bytemuck::bytes_of(&self.center);
        let b_4 = [0u8; 4];
        let mut v = vec![];

        v.extend(b_p);
        v.extend(b_4.clone());

        v.extend(b_h);
        v.extend(b_4.clone());

        v.extend(b_v);
        v.extend(b_4.clone());

        v.extend(b_c);
        v.extend(b_4.clone());

        v
    }
}
