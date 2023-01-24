use super::{
    bytes::Bytes,
    sphere::Sphere,
    vector3::{max, min, Vector3},
};

#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Vector3,
    pub max: Vector3,
}
impl From<Sphere> for BoundingBox {
    fn from(value: Sphere) -> Self {
        Self::from(&value)
    }
}
impl From<&Sphere> for BoundingBox {
    fn from(value: &Sphere) -> Self {
        let rad = Vector3::new(value.rad, value.rad, value.rad);
        let padding = 0.01;
        let offset = Vector3::new(padding, padding, padding);
        let min = value.pos - rad - offset;
        let max = value.pos + rad + offset;

        Self { min, max }
    }
}
impl Bytes for BoundingBox {
    fn bytes(&self) -> Vec<u8> {
        let b_min = bytemuck::bytes_of(&self.min);
        let b_max = bytemuck::bytes_of(&self.max);
        let byte = [0u8; 4];
        let mut v = vec![];

        v.extend(b_min);
        v.extend(byte);

        v.extend(b_max);
        v.extend(byte);

        v
    }
}
pub fn combine(lhs: &BoundingBox, rhs: &BoundingBox) -> BoundingBox {
    let min = min(&lhs.min, &rhs.min);
    let max = max(&lhs.max, &rhs.max);
    BoundingBox { min, max }
}
