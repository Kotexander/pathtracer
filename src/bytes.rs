pub trait Bytes {
    fn bytes(&self) -> Vec<u8>;
}

impl<T: Bytes> Bytes for &[T] {
    fn bytes(&self) -> Vec<u8> {
        let mut b = vec![];
        for i in *self {
            b.extend(i.bytes());
        }
        b
    }
}
impl<T: Bytes> Bytes for Vec<T> {
    fn bytes(&self) -> Vec<u8> {
        let mut b = vec![];
        for i in self {
            b.extend(i.bytes());
        }
        b
    }
}

impl Bytes for i32 {
    fn bytes(&self) -> Vec<u8> {
        Vec::from(bytemuck::bytes_of(self))
    }
}
