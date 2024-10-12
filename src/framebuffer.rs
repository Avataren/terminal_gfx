use crate::pixel::Pixel;

pub struct Framebuffer {
    pub width: usize, 
    pub height: usize,
    pub data: Vec<Pixel>,
    z_buffer: Vec<f32>,
    brightness_buffer: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let initial_pixel = Pixel { r: 0, g: 0, b: 0, alpha: 255 };
        Framebuffer {
            width,
            height,
            data: vec![initial_pixel; width * height],
            z_buffer: vec![0.0; width * height],
            brightness_buffer: vec![0; width * height],
        }
    }

    pub fn clear(&mut self) {
        let default_pixel = Pixel { r: 0, g: 0, b: 0, alpha: 255 };
        self.data.fill(default_pixel);
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &Pixel {
        &self.data[y * self.width + x]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.data[y * self.width + x] = pixel;
    }

    pub fn compute_brightness_buffer(&mut self, posterize_levels: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y);
                let brightness = (0.299 * pixel.r as f32 + 0.587 * pixel.g as f32 + 0.114 * pixel.b as f32) as u8;
                let posterized_brightness = Self::posterize_brightness(brightness, posterize_levels);
                self.brightness_buffer[y * self.width + x] = posterized_brightness;
            }
        }
    }

    pub fn apply_sharpening(&mut self, sharpening_factor: f32) {
        let mut temp_buffer = self.brightness_buffer.clone();

        for y in 1..self.height - 1 {
            for x in 1..self.width - 1 {
                let current = temp_buffer[y * self.width + x] as f32;
                let neighbors = [
                    temp_buffer[(y - 1) * self.width + x] as f32,
                    temp_buffer[(y + 1) * self.width + x] as f32,
                    temp_buffer[y * self.width + (x - 1)] as f32,
                    temp_buffer[y * self.width + (x + 1)] as f32,
                ];
                let blur = neighbors.iter().sum::<f32>() / 4.0;
                let sharpened = current + sharpening_factor * (current - blur);
                self.brightness_buffer[y * self.width + x] = sharpened.clamp(0.0, 255.0) as u8;
            }
        }
    }

    pub fn posterize_brightness(brightness: u8, levels: u8) -> u8 {
        let step = 255.0 / (levels - 1) as f32;
        ((brightness as f32 / step).round() * step) as u8
    }
    
    pub fn get_brightness(&self, x: usize, y: usize) -> u8 {
        self.brightness_buffer[y * self.width + x]
    }
}