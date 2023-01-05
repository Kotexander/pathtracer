use crate::{bytes::Bytes, vector3::Vector3};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pos: Vector3,
    rad: f32,
    albedo: Vector3,
}
impl Sphere {
    pub fn new(pos: Vector3, rad: f32, albedo: Vector3) -> Self {
        Self { pos, rad, albedo }
    }
}

impl Bytes for Sphere {
    fn bytes(&self) -> Vec<u8> {
        let b_p = bytemuck::bytes_of(&self.pos);
        let b_r = bytemuck::bytes_of(&self.rad);
        let b_a = bytemuck::bytes_of(&self.albedo);
        let b_4 = [0u8; 4];
        let mut v = vec![];

        v.extend(b_p);

        v.extend(b_r);

        v.extend(b_a);
        v.extend(b_4);

        v
    }
}
