use crate::{bytes::Bytes, vector3::Vector3};

#[repr(C)]
#[derive(
    Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize,
)]
pub struct Sphere {
    pos: Vector3,
    rad: f32,
    albedo: Vector3,
    roughness: f32,
}

impl Bytes for Sphere {
    fn bytes(&self) -> Vec<u8> {
        let b_pos = bytemuck::bytes_of(&self.pos);
        let b_rad = bytemuck::bytes_of(&self.rad);
        let b_albedo = bytemuck::bytes_of(&self.albedo);
        let b_roughness = bytemuck::bytes_of(&self.roughness);
        let mut v = vec![];

        v.extend(b_pos);
        v.extend(b_rad);
        v.extend(b_albedo);
        v.extend(b_roughness);

        v
    }
}
