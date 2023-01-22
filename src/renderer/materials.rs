use super::{bytes::Bytes, vector3::Vector3};

pub mod indecies {
    pub const LIGHT: u32 = 0;
    pub const LAMBERTIAN: u32 = 1;
    pub const METAL: u32 = 2;
    pub const GLASS: u32 = 3;
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Light {
    pub colour: Vector3,
}
impl Light {
    pub const fn new(colour: Vector3) -> Self {
        Self { colour }
    }
}
impl Default for Light {
    fn default() -> Self {
        Self {
            colour: Vector3::ONE,
        }
    }
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
    pub albedo: Vector3,
}

impl Lambertian {
    pub const fn new(albedo: Vector3) -> Self {
        Self { albedo }
    }
}
impl Default for Lambertian {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(0.5, 0.5, 0.5),
        }
    }
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
    pub albedo: Vector3,
    pub roughness: f32,
}

impl Metal {
    pub const fn new(albedo: Vector3, roughness: f32) -> Self {
        Self { albedo, roughness }
    }
}
impl Default for Metal {
    fn default() -> Self {
        Self {
            albedo: Vector3::new(0.5, 0.5, 0.5),
            roughness: 0.5,
        }
    }
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
    pub ir: f32,
}

impl Glass {
    pub const fn new(ir: f32) -> Self {
        Self { ir }
    }
}
impl Default for Glass {
    fn default() -> Self {
        Self { ir: 1.5 }
    }
}
impl Bytes for Glass {
    fn bytes(&self) -> Vec<u8> {
        Vec::from(bytemuck::bytes_of(&self.ir))
    }
}
