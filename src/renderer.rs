mod camera;
mod image;
mod ray;
mod sphere;

use self::image::*;
use super::math::vector3::*;
use camera::*;
use ray::*;
use sphere::*;

pub struct Renderer {
    pub camera: Camera,
    pub sphere: Sphere,
    pub image: Image,
}
impl Renderer {
    pub fn new() -> Self {
        let distance = 1.0;
        let camera = Camera::new(
            // &Ray::new(Vector3::Z * -distance, Vector3::Z), // front
            &Ray::new(Vector3::Z * distance, -Vector3::Z), // behind
            // &Ray::new(Vector3::X * -distance, Vector3::X), // left
            // &Ray::new(Vector3::X * distance, -Vector3::X), // right
            70.0f32.to_radians(),
        );
        let sphere = Sphere::new(0.5, Vector3::ZERO);
        let image = Image::new(800, 800);

        Self {
            camera,
            sphere,
            image,
        }
    }
    pub fn render(&mut self) {
        for x in 0..self.image.width() {
            for y in 0..self.image.height() {
                let u = x as f32 / self.image.width() as f32;
                let v = (self.image.height() - y) as f32 / self.image.height() as f32;

                let colour = self.render_pixel(u, v);
                self.image.set(x, y, colour);
            }
        }
    }
    pub fn render_pixel(&mut self, u: f32, v: f32) -> Vector3 {
        let ray = self.camera.get_ray(u, v);
        if let Some(hit) = self.sphere.intersect(&ray, 0.0, f32::INFINITY) {
            (hit.norm + Vector3::ONE) * 0.5
        } else {
            Vector3::ZERO
        }
    }
}
