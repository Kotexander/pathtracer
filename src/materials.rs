use crate::{bytes::Bytes, vector3::Vector3};

pub mod indecies {
    pub const LIGHT: u32 = 0;
    pub const LAMBERTIAN: u32 = 1;
    pub const METAL: u32 = 2;
    pub const GLASS: u32 = 3;
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Light {
    colour: Vector3,
}
impl Bytes for Light {
    fn bytes(&self) -> Vec<u8> {
        let b_colour = bytemuck::bytes_of(&self.colour);
        let byte = [0u8; 4];
        let mut v = vec![];

        v.extend(b_colour);
        v.extend(byte);

        v
    }
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Lambertian {
    albedo: Vector3,
}
impl Bytes for Lambertian {
    fn bytes(&self) -> Vec<u8> {
        let b_albedo = bytemuck::bytes_of(&self.albedo);
        let byte = [0u8; 4];
        let mut v = vec![];

        v.extend(b_albedo);
        v.extend(byte);

        v
    }
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Metal {
    albedo: Vector3,
    roughness: f32,
}
impl Bytes for Metal {
    fn bytes(&self) -> Vec<u8> {
        let b_albedo = bytemuck::bytes_of(&self.albedo);
        let b_roughness = bytemuck::bytes_of(&self.roughness);
        let mut v = vec![];

        v.extend(b_albedo);
        v.extend(b_roughness);

        v
    }
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Glass {
    ir: f32,
}
impl Bytes for Glass {
    fn bytes(&self) -> Vec<u8> {
        Vec::from(bytemuck::bytes_of(&self.ir))
    }
}
