#[derive(Debug, Clone, Copy)]
pub struct Rgba8(u32);

impl Rgba8 {
    pub fn r(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    pub fn b(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    pub fn a(&self) -> u8 {
        (self.0 & 0xff) as u8
    }

    pub fn to_array(&self) -> [u8; 4] {
        [self.r(), self.g(), self.b(), self.a()]
    }
}

#[cfg(test)]
mod test_color {
    use super::*;

    #[test]
    fn test() {
        let rgba = Rgba8(0xaabbcc77);
        assert_eq!(rgba.r(), 0xaa);
        assert_eq!(rgba.g(), 0xbb);
        assert_eq!(rgba.b(), 0xcc);
        assert_eq!(rgba.a(), 0x77);
        assert_eq!(rgba.to_array(), [0xaa, 0xbb, 0xcc, 0x77]);
    }
}