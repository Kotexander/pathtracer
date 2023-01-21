use crate::{bytes::Bytes, vector3::Vector3};

#[repr(C)]
#[derive(
    Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, serde::Serialize, serde::Deserialize,
)]
pub struct Sphere {
    pub pos: Vector3,
    pub rad: f32,

    pub mat_type: u32,
    pub mat_index: u32,
}

impl Sphere {
    pub const fn new(pos: Vector3, rad: f32, mat_type: u32, mat_index: u32) -> Self {
        Self {
            pos,
            rad,
            mat_type,
            mat_index,
        }
    }
}

impl Bytes for Sphere {
    fn bytes(&self) -> Vec<u8> {
        let b_pos = bytemuck::bytes_of(&self.pos);
        let b_rad = bytemuck::bytes_of(&self.rad);
        let b_mat_type = bytemuck::bytes_of(&self.mat_type);
        let b_mat_index = bytemuck::bytes_of(&self.mat_index);
        let byte = [0u8; 4];
        let mut v = vec![];

        v.extend(b_pos);
        v.extend(b_rad);
        v.extend(b_mat_type);
        v.extend(b_mat_index);
        v.extend(byte);
        v.extend(byte);

        v
    }
}
