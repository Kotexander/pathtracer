#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn length_sqrd(&self) -> f32 {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }
    pub fn length(&self) -> f32 {
        self.length_sqrd().sqrt()
    }
    pub fn normal(&self) -> Self {
        *self / self.length()
    }
    pub fn normalize(&mut self) {
        *self /= self.length()
    }
}

use std::ops::*;
impl Add for Vector3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}
impl AddAssign for Vector3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
impl SubAssign for Vector3 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}
impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}
impl Div<f32> for Vector3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        let inv_rhs = 1.0 / rhs;
        Self::new(self.x * inv_rhs, self.y * inv_rhs, self.z * inv_rhs)
    }
}
impl DivAssign<f32> for Vector3 {
    fn div_assign(&mut self, rhs: f32) {
        let inv_rhs = 1.0 / rhs;
        self.x *= inv_rhs;
        self.y *= inv_rhs;
        self.z *= inv_rhs;
    }
}

mod tests {
    use super::*;

    #[test]
    fn length() {
        let v = Vector3::new(2.0, 0.0, 0.0);
        assert_eq!(v.length(), 2.0);

        let v = Vector3::new(1.0, 1.0, 1.0);
        assert_eq!(v.length(), 3.0f32.sqrt());

        let v = Vector3::new(2.0, 3.0, 4.0);
        assert_eq!(v.length_sqrd(), 29.0)
    }

    #[test]
    fn ops() {
        let v1 = Vector3::new(5.0, 4.0, 3.0);
        let v2 = Vector3::new(10.0, 11.0, 12.0);

        assert_eq!(v1 + v2, Vector3::new(15.0, 15.0, 15.0));
        assert_eq!(v1 - v2, Vector3::new(-5.0, -7.0, -9.0));
        assert_eq!(v1 * 3.0, Vector3::new(15.0, 12.0, 9.0));
        assert_eq!(v2 / 2.0, Vector3::new(5.0, 5.5, 6.0));
    }

    #[test]
    fn normal() {
        let mut v = Vector3::new(20.0, 0.0, 0.0);
        assert_eq!(v.normal(), Vector3::new(1.0, 0.0, 0.0));
        v.normalize();
        assert_eq!(v, Vector3::new(1.0, 0.0, 0.0));
    }
}
