use crate::{math::vector3::*, ray::Ray};

pub struct Hit {
    t: f32,
    pos: Vector3,
    norm: Vector3,
}

#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    rad: f32,
    pos: Vector3,
}
impl Sphere {
    pub fn new(rad: f32, pos: Vector3) -> Self {
        Self { rad, pos }
    }
    pub fn intersect(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
        let a = dot(&ray.dir, &ray.dir);
        let b = 2.0 * dot(&ray.pos, &ray.dir);
        let c = dot(&ray.pos, &ray.pos) - (self.rad * self.rad);

        let d = b * b - 4.0 * a * c;
        if d < 0.0 {
            return None;
        }

        let d_sqrt = d.sqrt();
        // let t1 = (-b + d_sqrt) / (2.0 * a);
        let t = (-b - d_sqrt) / (2.0 * a); // this will always be closer

        if t > t_max || t < t_min {
            return None;
        }

        let pos = ray.at(t);
        let norm = (pos - self.pos).normal();

        Some(Hit { t, pos, norm })
    }
}

mod tests {
    use super::*;
    #[test]
    fn intersect() {
        let ray = Ray::new(-Vector3::Z * 3.0, Vector3::Z);
        let sphere = Sphere::new(0.5, Vector3::ZERO);

        let hit = sphere.intersect(&ray, 0.0, f32::INFINITY).unwrap();
        assert_eq!(hit.t, 2.5);
        assert_eq!(hit.pos, Vector3::Z * -0.5);
        assert_eq!(hit.norm, -Vector3::Z);

        let ray = Ray::new(-Vector3::Z * 3.0, Vector3::Y);
        assert!(sphere.intersect(&ray, 0.0, f32::INFINITY).is_none());
    }
}
