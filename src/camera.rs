use crate::{ray::Ray, vector3::*};

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
    pub fn bytes(&self) -> Vec<u8> {
        let mut b_p = Vec::from(bytemuck::bytes_of(&self.pos));
        let mut b_h = Vec::from(bytemuck::bytes_of(&self.horizontal));
        let mut b_v = Vec::from(bytemuck::bytes_of(&self.vertical));
        let mut b_c = Vec::from(bytemuck::bytes_of(&self.center));
        let b_4 = vec![0u8; 4];
        let mut v = vec![];

        v.append(&mut b_p);
        v.append(&mut b_4.clone());

        v.append(&mut b_h);
        v.append(&mut b_4.clone());

        v.append(&mut b_v);
        v.append(&mut b_4.clone());

        v.append(&mut b_c);
        v.append(&mut b_4.clone());

        v
    }
    // pub fn get_ray(&self, u: f32, v: f32) -> Ray {
    //     Ray::new(
    //         self.pos,
    //         (self.center + self.horizontal * u + self.vertical * v - self.pos).normal(),
    //     )
    // }
}
