use crate::framebuffer::Framebuffer;

pub fn compute_gradients(fb: &Framebuffer) -> Vec<(f32, f32)> {
    let mut gradients = vec![(0.0, 0.0); fb.width * fb.height];

    let gx: [[i32; 3]; 3] = [
        [-1, 0, 1],
        [-2, 0, 2],
        [-1, 0, 1],
    ];

    let gy: [[i32; 3]; 3] = [
        [-1, -2, -1],
        [ 0,  0,  0],
        [ 1,  2,  1],
    ];

    for y in 1..fb.height - 1 {
        for x in 1..fb.width - 1 {
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

            gradients[y * fb.width + x] = (mag, angle);
        }
    }

    // Non-maximum suppression
    let mut suppressed = vec![(0.0f32, 0.0f32); fb.width * fb.height];
    for y in 1..fb.height - 1 {
        for x in 1..fb.width - 1 {
            let (mag, angle) = gradients[y * fb.width + x];
            let angle_deg = angle.to_degrees();
            
            let (nx, ny) = if (-22.5..22.5).contains(&angle_deg) || (157.5..202.5).contains(&angle_deg) {
                (1, 0)
            } else if (22.5..67.5).contains(&angle_deg) || (-157.5..-112.5).contains(&angle_deg) {
                (1, -1)
            } else if (67.5..112.5).contains(&angle_deg) || (-112.5..-67.5).contains(&angle_deg) {
                (0, -1)
            } else {
                (1, 1)
            };

            let prev = gradients[(y as i32 - ny) as usize * fb.width + (x as i32 - nx) as usize].0;
            let next = gradients[(y as i32 + ny) as usize * fb.width + (x as i32 + nx) as usize].0;

            suppressed[y * fb.width + x] = if mag >= prev && mag >= next {
                (mag, angle)
            } else {
                (0.0, angle)
            };
        }
    }

    suppressed
}

