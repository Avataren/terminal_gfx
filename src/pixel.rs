#[derive(Clone)] // Derive Clone for Pixel
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub alpha: u8,
}

impl Pixel {
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }
}
