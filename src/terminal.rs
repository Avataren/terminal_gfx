use ncurses::*;
use crate::framebuffer::Framebuffer;
use crate::ascii::{angle_to_ascii, brightness_to_ascii};
use std::env;
use lazy_static::lazy_static;
use std::sync::Once;

const COLOR_PAIRS: usize = 216; // 6 levels for each R, G, B (6^3 = 216)

lazy_static! {
    static ref COLOR_PAIRS_INITIALIZED: Once = Once::new();
}

fn supports_true_color() -> bool {
    env::var("COLORTERM").map_or(false, |val| val == "truecolor" || val == "24bit")
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

pub fn draw_colored_frame(fb: &Framebuffer, gradients: &[(f32, f32)]) {
    let is_true_color = supports_true_color();

    if !is_true_color {
        init_color_pairs();
    }

    for y in 0..fb.height {
        for x in 0..fb.width {
            let pixel = fb.get_pixel(x, y);
            let (magnitude, angle) = gradients[y * fb.width + x];
            let brightness = fb.get_brightness(x, y);

            let ch = if magnitude > 100.0 {
                angle_to_ascii(angle)
            } else {
                brightness_to_ascii(brightness, false)
            };

            mv(y as i32, x as i32);

            let (r, g, b) = pixel.to_rgb();
            if is_true_color {
                addstr(&format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, ch));
            } else {
                let color_pair = get_closest_color_pair(r, g, b);
                attron(COLOR_PAIR(color_pair));
                addch(ch as u32);
                attroff(COLOR_PAIR(color_pair));
            }
        }
    }

    refresh();
}