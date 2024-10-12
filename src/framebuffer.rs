use crate::pixel::Pixel;

use lazy_static::lazy_static;

pub struct ColorPalette {
    colors: Vec<(u8, u8, u8)>,
}


impl ColorPalette {
    pub fn new() -> Self {
        let colors: Vec<(u8, u8, u8)> = (0..216)
            .map(|i| {
                let r = (i / 36) * 51;
                let g = ((i / 6) % 6) * 51;
                let b = (i % 6) * 51;
                (r as u8, g as u8, b as u8)
            })
            .collect();
        ColorPalette { colors }
    }

    pub fn closest_color(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        *self.colors
            .iter()
            .min_by_key(|&&(cr, cg, cb)| {
                let dr = (r as i32 - cr as i32).abs();
                let dg = (g as i32 - cg as i32).abs();
                let db = (b as i32 - cb as i32).abs();
                dr * dr + dg * dg + db * db
            })
            .unwrap()
    }
}

lazy_static! {
    static ref TERMINAL_COLORS: ColorPalette = ColorPalette::new();
}

pub struct Framebuffer {
    pub width: usize, 
    pub height: usize,
    pub data: Vec<Pixel>,
    z_buffer: Vec<f32>,
    brightness_buffer: Vec<u8>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let initial_pixel = Pixel { r: 0, g: 0, b: 0, a: 255 };
        Framebuffer {
            width,
            height,
            data: vec![initial_pixel; width * height],
            z_buffer: vec![0.0; width * height],
            brightness_buffer: vec![0; width * height],
        }
    }

    pub fn clear(&mut self) {
        let default_pixel = Pixel { r: 0, g: 0, b: 0, a: 255 };
        self.data.fill(default_pixel);
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &Pixel {
        &self.data[y * self.width + x]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: Pixel) {
        self.data[y * self.width + x] = pixel;
    }

    pub fn apply_bayer_dithering(&mut self) {
        const BAYER_MATRIX: [[f32; 2]; 2] = [
            [0.0 / 4.0, 2.0 / 4.0],
            [3.0 / 4.0, 1.0 / 4.0],
        ];

        for y in 0..self.height {
            for x in 0..self.width {
                let pixel = self.get_pixel(x, y);
                let (r, g, b) = (pixel.r as f32, pixel.g as f32, pixel.b as f32);

                // Apply Bayer matrix threshold
                let threshold = BAYER_MATRIX[y % 2][x % 2] * 255.0;
                
                // Apply dithering with reduced intensity
                let dither_factor = 0.1; // Adjust this value to control dithering intensity
                let r_dithered = (r + (threshold - 128.0) * dither_factor).clamp(0.0, 255.0) as u8;
                let g_dithered = (g + (threshold - 128.0) * dither_factor).clamp(0.0, 255.0) as u8;
                let b_dithered = (b + (threshold - 128.0) * dither_factor).clamp(0.0, 255.0) as u8;

                // Find the closest terminal color
                let closest_color = TERMINAL_COLORS.closest_color(r_dithered, g_dithered, b_dithered);

                // Set the pixel to the closest terminal color
                self.set_pixel(x, y, Pixel {
                    r: closest_color.0,
                    g: closest_color.1,
                    b: closest_color.2,
                    a: 255,
                });
            }
        }
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

    pub fn increase_brightness(&mut self, brightness_factor: f32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let brightness = self.get_brightness(x, y) as f32;
                
                // Apply brightness adjustment
                let brightened = (brightness * brightness_factor).clamp(0.0, 255.0);

                // Store the result back in the brightness buffer
                self.brightness_buffer[y * self.width + x] = brightened as u8;
            }
        }
    }

    pub fn increase_contrast(&mut self, contrast_factor: f32) {
        for y in 0..self.height {
            for x in 0..self.width {
                let brightness = self.get_brightness(x, y) as f32 / 255.0;

                // Apply contrast transformation
                let contrasted = ((brightness - 0.5) * contrast_factor + 0.5).clamp(0.0, 1.0);

                // Convert back to u8 and store in the brightness buffer
                self.brightness_buffer[y * self.width + x] = (contrasted * 255.0) as u8;
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