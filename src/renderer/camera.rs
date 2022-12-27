use super::*;

const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);

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

#[derive(Clone, Copy, Debug)]
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
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.pos,
            (self.center + self.horizontal * u + self.vertical * v - self.pos).normal(),
        )
    }
}
