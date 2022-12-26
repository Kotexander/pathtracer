use super::*;

const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);

#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pos: Vector3,

    horizontal: Vector3,
    vertical: Vector3,
    lower_left: Vector3,
}
impl Camera {
    pub fn new(ray: &Ray, vfov: f32) -> Self {
        let h = (vfov / 2.0).tan();
        let viewport = h * 2.0;

        let h = cross(&UP, &ray.dir);
        let v = cross(&ray.dir, &h);

        let horizontal = h * viewport;
        let vertical = v * viewport;

        let lower_left = ray.pos - horizontal / 2.0 - vertical / 2.0 + ray.dir;

        Self {
            pos: ray.pos,
            horizontal,
            vertical,
            lower_left,
        }
    }
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
            self.pos,
            (self.lower_left + self.horizontal * u + self.vertical * v - self.pos).normal(),
        )
    }
}
