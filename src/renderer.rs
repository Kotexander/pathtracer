pub mod camera;
pub mod image;
pub mod ray;
pub mod sphere;

use self::image::*;
use super::math::vector3::*;
use camera::*;
use ray::*;
use sphere::*;

pub struct Renderer {
    camera: Camera,
    pub camera_config: CameraConfig,
    pub sphere: Sphere,
    image: Image,
}
impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        let distance = 1.0;
        let aspect = width as f32 / height as f32;

        let camera_config = CameraConfig::new(
            Ray::new(Vector3::Z * -distance, Vector3::Z), // front
            // Ray::new(Vector3::Z * distance, -Vector3::Z), // behind
            // Ray::new(Vector3::X * -distance, Vector3::X), // left
            // Ray::new(Vector3::X * distance, -Vector3::X), // right
            50.0f32.to_radians(),
            aspect,
        );
        let camera = camera_config.build();

        let sphere = Sphere::new(0.5, Vector3::ZERO);
        let image = Image::new(width, height);

        Self {
            camera,
            camera_config,
            sphere,
            image,
        }
    }
    pub fn render(&mut self) {
        for x in 0..self.image.width() {
            for y in 0..self.image.height() {
                let u = 2.0 * x as f32 / self.image.width() as f32 - 1.0;
                let v = 2.0 * (self.image.height() - y) as f32 / self.image.height() as f32 - 1.0;

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
    pub fn resize(&mut self, width: usize, height: usize) {
        self.image = Image::new(width, height);
        self.camera_config.aspect = width as f32 / height as f32;
        self.update_camera();
    }
    pub fn update_camera(&mut self) {
        self.camera = self.camera_config.build();
    }

    pub fn image(&self) -> &Image {
        &self.image
    }
}
