use rayon::prelude::*;
use crate::framebuffer::Framebuffer;

const CHUNK_SIZE: usize = 4;

pub fn compute_gradients(fb: &Framebuffer) -> Vec<(f32, f32)> {
    let width = fb.width;
    let height = fb.height;

    let gx: [[i32; 3]; 3] = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy: [[i32; 3]; 3] = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    // Compute gradients in parallel
    let gradients: Vec<(f32, f32)> = (0..height)
        .into_par_iter()
        .flat_map(|y| {
            (0..width).into_par_iter().map(move |x| {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    (0.0, 0.0)
                } else {
                    let mut grad_x = 0;
                    let mut grad_y = 0;
                    for dy in 0..3 {
                        for dx in 0..3 {
                            let px = fb.get_brightness(x + dx - 1, y + dy - 1) as i32;
                            grad_x += px * gx[dy][dx];
                            grad_y += px * gy[dy][dx];
                        }
                    }
                    let mag = ((grad_x * grad_x + grad_y * grad_y) as f32).sqrt();
                    let angle = (grad_y as f32).atan2(grad_x as f32);
                    (mag, angle)
                }
            })
        })
        .collect();

    // Apply non-maximum suppression
    apply_non_maximum_suppression(&gradients, width, height)
}

fn apply_non_maximum_suppression(gradients: &[(f32, f32)], width: usize, height: usize) -> Vec<(f32, f32)> {
    (0..height)
        .into_par_iter()
        .flat_map(|y| {
            (0..width).into_par_iter().map(move |x| {
                if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    (0.0, 0.0)
                } else {
                    let (mag, angle) = gradients[y * width + x];
                    let angle_deg = angle.to_degrees();
                    let (nx, ny): (i32, i32) = if (-22.5..22.5).contains(&angle_deg) || (157.5..202.5).contains(&angle_deg) {
                        (1, 0)
                    } else if (22.5..67.5).contains(&angle_deg) || (-157.5..-112.5).contains(&angle_deg) {
                        (1, -1)
                    } else if (67.5..112.5).contains(&angle_deg) || (-112.5..-67.5).contains(&angle_deg) {
                        (0, -1)
                    } else {
                        (1, 1)
                    };
                    let prev_y = (y as i32 - ny).max(0) as usize;
                    let prev_x = (x as i32 - nx).max(0) as usize;
                    let next_y = (y as i32 + ny).min(height as i32 - 1) as usize;
                    let next_x = (x as i32 + nx).min(width as i32 - 1) as usize;
                    let prev = gradients[prev_y * width + prev_x].0;
                    let next = gradients[next_y * width + next_x].0;
                    if mag >= prev && mag >= next {
                        (mag, angle)
                    } else {
                        (0.0, angle)
                    }
                }
            })
        })
        .collect()
}