use crate::{camera::Camera, math::vector3::Vector3, ray::Ray};

mod camera;
mod math;
mod ray;

fn main() {
    let ray = Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0));
    let camera = Camera::new(&ray, 70.0f32.to_radians());

    println!("{camera:#?}");
    println!("{:#?}", camera.get_ray(0.5, 0.5));
    println!("{:#?}", camera.get_ray(0.0, 0.0));
}
