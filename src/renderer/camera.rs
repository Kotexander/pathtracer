use super::{bytes::Bytes, ray::Ray, vector3::*};

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct CameraSettings {
    pub pos: Vector3,
    /// horizontal angle in degree
    pub yaw: f32,
    /// vertical angle in degree
    pub pitch: f32,
    /// vertical in degrees
    pub vfov: f32,
}
impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, 0.0, -3.0),
            yaw: 0.0,
            pitch: 0.0,
            vfov: 60.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CameraConfig {
    pub pos: Vector3,
    /// horizontal angle in radians
    pub yaw: f32,
    /// vertical angle in radians
    pub pitch: f32,
    /// vertical fov in radians
    pub vfov: f32,
    pub aspect: f32,
}
impl CameraConfig {
    pub fn new(settings: CameraSettings, aspect: f32) -> Self {
        Self {
            pos: settings.pos,
            yaw: settings.yaw.to_radians(),
            pitch: settings.pitch.to_radians(),
            vfov: settings.vfov.to_radians(),
            aspect,
        }
    }
    pub fn build(&self) -> Camera {
        let dir = self.dir();
        let ray = Ray::new(self.pos, dir);

        Camera::new(&ray, self.vfov, self.aspect)
    }
    pub fn dir(&self) -> Vector3 {
        let y = self.pitch.sin();
        let z = self.yaw.cos() * self.pitch.cos();
        let x = self.yaw.sin() * self.pitch.cos();

        Vector3::new(x, y, z)
    }
}
pub const UP: Vector3 = Vector3::new(0.0, 1.0, 0.0);

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pos: Vector3,

    horizontal: Vector3,
    vertical: Vector3,
    center: Vector3,
}
impl Camera {
    pub fn new(ray: &Ray, vfov: f32, aspect: f32) -> Self {
        let h = (vfov / 2.0).tan();
        let viewport = h * 2.0;

        let h = cross(&UP, &ray.dir).normal();
        let v = cross(&ray.dir, &h);

        let horizontal = h * viewport * aspect;
        let vertical = v * viewport;

        let center = ray.pos + ray.dir;

        Self {
            pos: ray.pos,
            horizontal,
            vertical,
            center,
        }
    }
}
impl Bytes for Camera {
    fn bytes(&self) -> Vec<u8> {
        let b_pos = bytemuck::bytes_of(&self.pos);
        let b_horizontal = bytemuck::bytes_of(&self.horizontal);
        let b_vertical = bytemuck::bytes_of(&self.vertical);
        let b_center = bytemuck::bytes_of(&self.center);
        let byte = [0u8; 4];
        let mut v = vec![];

        v.extend(b_pos);
        v.extend(byte);

        v.extend(b_horizontal);
        v.extend(byte);

        v.extend(b_vertical);
        v.extend(byte);

        v.extend(b_center);
        v.extend(byte);

        v
    }
}
