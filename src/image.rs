use crate::math::vector3::Vector3;

pub const BLACK: Vector3 = Vector3::new(0.0, 0.0, 0.0);

#[derive(Clone)]
pub struct Image {
    width: usize,
    height: usize,
    buffer: Vec<Vector3>,
}
impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        let buffer = vec![BLACK; width * height];

        Self {
            width,
            height,
            buffer,
        }
    }
    pub fn get(&self, x: usize, y: usize) -> &Vector3 {
        &self.buffer[y * self.width + x]
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut Vector3 {
        &mut self.buffer[y * self.width + x]
    }
    pub fn set(&mut self, x: usize, y: usize, colour: Vector3) {
        self.buffer[y * self.width + x] = colour;
    }
    pub fn clear(&mut self, colour: Vector3) {
        for pixel in &mut self.buffer {
            *pixel = colour;
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
}
impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("width", &self.width)
            .field("height", &self.height)
            // .field("buffer", &self.buffer)
            .finish()
    }
}
