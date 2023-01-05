use crate::vector3::Vector3;

#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub pos: Vector3,
    pub dir: Vector3,
}
impl Ray {
    pub const fn new(pos: Vector3, dir: Vector3) -> Self {
        Self { pos, dir }
    }
    // pub fn at(&self, t: f32) -> Vector3 {
    //     self.pos + (self.dir * t)
    // }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn at() {
//         let ray = Ray::new(Vector3::new(0.0, 1.0, 0.0), Vector3::new(0.0, 1.0, 1.0));
//         assert_eq!(ray.at(5.0), Vector3::new(0.0, 6.0, 5.0));
//     }
// }
