use crate::vector3::Vector3;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    pos: Vector3,
    rad: f32,
}
impl Sphere {
    pub fn new(pos: Vector3, rad: f32) -> Self {
        Self { pos, rad }
    }
}
