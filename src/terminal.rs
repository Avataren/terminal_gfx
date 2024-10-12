use ncurses::*;
use crate::framebuffer::Framebuffer;
use crate::terminalbuffer::TerminalBuffer;
use crate::ascii::{angle_to_ascii, brightness_to_ascii};
// use std::env;
use lazy_static::lazy_static;
use std::sync::Once;

const COLOR_PAIRS: usize = 216; // 6 levels for each R, G, B (6^3 = 216)
const ANGLE_TO_ASCII_THRESHOLD: f32 = 40.0;

lazy_static! {
    static ref COLOR_PAIRS_INITIALIZED: Once = Once::new();
}

fn supports_true_color() -> bool {
    false
    // env::var("COLORTERM").map_or(false, |val| val == "truecolor" || val == "24bit")
}

fn init_color_pairs() {
    COLOR_PAIRS_INITIALIZED.call_once(|| {
        start_color();
        use_default_colors();
        for i in 0..COLOR_PAIRS {
            let r = (i / 36) as i16 * 200;
            let g = ((i / 6) % 6) as i16 * 200;
            let b = (i % 6) as i16 * 200;
            init_color(i as i16, r, g, b);
            init_pair(i as i16 + 1, i as i16, -1); // -1 for default background
        }
    });
}

fn get_closest_color_pair(r: u8, g: u8, b: u8) -> i16 {
    let r_index = (r as usize * 5) / 255;
    let g_index = (g as usize * 5) / 255;
    let b_index = (b as usize * 5) / 255;
    let index = r_index * 36 + g_index * 6 + b_index;
    (index.min(COLOR_PAIRS - 1) + 1) as i16
}

fn average_neighbor_colors(fb: &Framebuffer, x: usize, y: usize) -> (u8, u8, u8) {
    let mut r_sum = 0;
    let mut g_sum = 0;
    let mut b_sum = 0;
    let mut count = 0;

    for dy in -1..=1 {
        for dx in -1..=1 {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < fb.width as i32 && ny >= 0 && ny < fb.height as i32 {
                let pixel = fb.get_pixel(nx as usize, ny as usize);
                let (r, g, b) = pixel.to_rgb();
                r_sum += r as u32;
                g_sum += g as u32;
                b_sum += b as u32;
                count += 1;
            }
        }
    }
    count /=2;
    (
        (r_sum / count) as u8,
        (g_sum / count) as u8,
        (b_sum / count) as u8,
    )
}

pub fn draw_colored_frame(fb: &Framebuffer, gradients: &[(f32, f32)], buffer: &mut TerminalBuffer) {
    let is_true_color = supports_true_color();
    if !is_true_color {
        init_color_pairs();
    }

    buffer.clear();

    for y in 0..fb.height {
        for x in 0..fb.width {
            let (magnitude, angle) = gradients[y * fb.width + x];
            let brightness = fb.get_brightness(x, y);
            let ch = if magnitude > ANGLE_TO_ASCII_THRESHOLD {
                angle_to_ascii(angle)
            } else {
                brightness_to_ascii(brightness, false)
            };

            let (r, g, b) = if magnitude > ANGLE_TO_ASCII_THRESHOLD {
                average_neighbor_colors(fb, x, y)
            } else {
                fb.get_pixel(x, y).to_rgb()
            };

            if is_true_color {
                // Note: True color support might need to be handled differently with double buffering
                buffer.set_char(x, y, ch, 0);
            } else {
                let color_pair = get_closest_color_pair(r, g, b);
                buffer.set_char(x, y, ch, color_pair);
            }
        }
    }

    buffer.swap_buffers();
    buffer.render();
}