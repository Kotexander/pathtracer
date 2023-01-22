use super::bytes::Bytes;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub seed: u32,
    pub samples: i32,
    pub depth: i32,
}
impl Globals {
    pub fn new(seed: u32, samples: i32, depth: i32) -> Self {
        Self {
            seed,
            samples,
            depth,
        }
    }
}
impl Bytes for Globals {
    fn bytes(&self) -> Vec<u8> {
        Vec::from(bytemuck::bytes_of(self))
    }
}
