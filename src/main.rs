use std::sync::{Arc, Mutex};

use math::Smoothstep;
use ncurses::*;
use raymarch::{ray_march, update_globals};
use std::env;
use std::f32::consts::PI;
use minifb::{Window, WindowOptions};
use std::time::Instant;
use rayon::prelude::*;

mod raymarch;
mod framebuffer;
mod sobel;
mod terminal;
mod ascii;
mod pixel;
mod terminalbuffer;
mod math;

use crate::framebuffer::Framebuffer;
use crate::sobel::compute_gradients;
use crate::terminal::draw_colored_frame;
use crate::pixel::Pixel;
use crate::terminalbuffer::TerminalBuffer;
use crate::math::{Vec2, Vec3, Mat4};

const CHUNK_SIZE: usize = 8; 

fn main() {
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.contains(&"--debug".to_string());

    initscr();  // Start the ncurses session
    noecho();   // Disable echoing of characters
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);  // Hide the cursor
    nodelay(stdscr(), true);  // Don't block the getch call

    // Create framebuffer and window dimensions based on terminal size
    let framebuffer = Arc::new(Mutex::new(create_framebuffer()));
    let mut paused = false; // Track whether the animation is paused
    let target_fps = 60.0;
    let mut last_time = Instant::now();

    // Initialize minifb window for debug mode
    let mut window = if debug_mode {
        Some(Window::new(
            "Debug Framebuffer - ESC to exit",
            framebuffer.lock().unwrap().width,
            framebuffer.lock().unwrap().height,
            WindowOptions::default(),
        ).unwrap_or_else(|e| {
            panic!("{}", e);
        }))
    } else {
        None
    };

    let mut terminal_buffer = {
        let fb = framebuffer.lock().unwrap();
        TerminalBuffer::new(fb.width, fb.height)
    };

    // Minifb buffer for graphical rendering (used only in debug mode)
    let mut buffer = if debug_mode {
        let fb = framebuffer.lock().unwrap();
        vec![0; fb.width * fb.height]
    } else {
        vec![]
    };

    let mut total_elapsed_time = 0.0;
    let start_time = Instant::now();
    
    let mut prev_width;
    let mut prev_height;
    {
        let fb = framebuffer.lock().unwrap();
        prev_width = fb.width;
        prev_height = fb.height;
    }

    loop {
        // Calculate deltaTime
        let now = Instant::now();
        let total_elapsed_time = now.duration_since(start_time).as_secs_f32();  // Accurate elapsed time

        let delta_time = now.duration_since(last_time).as_secs_f32();
        last_time = now;
        
        // Handle user input
        let ch = getch();
        if ch == 32 {  // Spacebar is ASCII 32
            paused = !paused;
        }
        if ch == 27 {  // ESC is ASCII 27
            break;
        }

        // Check if terminal size has changed
        let mut new_width = 0;
        let mut new_height = 0;
        getmaxyx(stdscr(), &mut new_height, &mut new_width);

        // Convert i32 to usize for framebuffer comparison
        let new_width_usize = new_width as usize;
        let new_height_usize = new_height as usize;

        if new_width_usize != prev_width || new_height_usize != prev_height {
            // Terminal has been resized, adjust framebuffer
            terminal_buffer.resize(new_width_usize, new_height_usize);
            let mut fb = framebuffer.lock().unwrap();
            *fb = Framebuffer::new(new_width_usize, new_height_usize);
            prev_width = new_width_usize;
            prev_height = new_height_usize;

            clear();  // Clear the screen after resizing
        }

        if !paused {
            {
                let mut fb = framebuffer.lock().unwrap();
                fb.clear();  // Clear framebuffer before drawing
            }
            update(delta_time, total_elapsed_time, &framebuffer);
            draw(&framebuffer, &mut window, &mut buffer, &mut terminal_buffer, debug_mode);        
        }

        // Sleep to maintain the target framerate
        let elapsed_time = now.elapsed().as_secs_f32();
        let sleep_time = (1.0 / target_fps - elapsed_time).max(0.0);
        std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time));
    }

    endwin();  // End the ncurses session
}

fn update(delta_time: f32, total_time: f32, framebuffer: &Arc<Mutex<Framebuffer>>) {
    let fb = framebuffer.lock().unwrap();
    let width = fb.width as f32;
    let height = fb.height as f32;
    drop(fb); // Release the lock

    update_globals(Vec2::new(width, height), total_time);
    draw_test_scene(framebuffer, total_time);
}

pub fn draw_test_scene(framebuffer: &Arc<Mutex<Framebuffer>>, total_time: f32) {
    let fb = framebuffer.lock().unwrap();
    let width = fb.width;
    let height = fb.height;
    drop(fb); // Release the lock

    let aspect_ratio = width as f32 / height as f32;

    // Camera setup
    let camera_radius = 8.0;
    let camera_height = 3.0 + (total_time * 0.2).sin();
    let camera_angle = total_time * 0.5;

    let eye = Vec3::new(0.0, 1.25, -1.75); // Positioned at (0, 5, 5)
    let target = Vec3::new(0.0, 0.0, 0.0); // Looking directly at the origin
    let up = Vec3::new(0.0, 1.0, 0.0);    

    // let eye = Vec3::new(
    //     camera_radius * camera_angle.cos(),
    //     camera_height,
    //     camera_radius * camera_angle.sin()
    // );
    // let target = Vec3::new(0.0, 1.0, 0.0); // Look at the center of the scene, slightly above the ground
    // let up = Vec3::new(0.0, 1.0, 0.0);

    let view_matrix = look_at(eye, target, up);

    let chunks: Vec<_> = (0..height)
        .step_by(CHUNK_SIZE)
        .flat_map(|y| {
            (0..width).step_by(CHUNK_SIZE).map(move |x| (x, y))
        })
        .collect();

    let chunk_results: Vec<Vec<Pixel>> = chunks.par_iter().map(|&(start_x, start_y)| {
        let mut chunk_pixels = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE);
        for y in start_y..std::cmp::min(start_y + CHUNK_SIZE, height) {
            for x in start_x..std::cmp::min(start_x + CHUNK_SIZE, width) {
                let ndc_x = (2.0 * x as f32 / width as f32 - 1.0) * aspect_ratio;
                let ndc_y = 1.0 - 2.0 * y as f32 / height as f32;
                let ray_dir = calculate_ray_direction(ndc_x, ndc_y, &view_matrix);

                let color = ray_march(eye, ray_dir, total_time);
                chunk_pixels.push(color);
            }
        }
        chunk_pixels
    }).collect();

    let mut fb = framebuffer.lock().unwrap();
    for (&(start_x, start_y), chunk_pixels) in chunks.iter().zip(chunk_results.iter()) {
        let mut pixel_index = 0;
        for y in start_y..std::cmp::min(start_y + CHUNK_SIZE, height) {
            for x in start_x..std::cmp::min(start_x + CHUNK_SIZE, width) {
                fb.set_pixel(x, y, chunk_pixels[pixel_index]);
                pixel_index += 1;
            }
        }
    }
}

fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let f = (target - eye).normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f);

    Mat4([
        [s.x, u.x, -f.x, 0.0],
        [s.y, u.y, -f.y, 0.0],
        [s.z, u.z, -f.z, 0.0],
        [-s.dot(&eye), -u.dot(&eye), f.dot(&eye), 1.0]
    ])
}

fn calculate_ray_direction(ndc_x: f32, ndc_y: f32, view_matrix: &Mat4) -> Vec3 {
    let fov = 45.0f32.to_radians(); // Adjusted field of view
    let tan_fov = (fov * 0.5).tan();
    
    // Adjust y scaling to account for font aspect ratio (twice as high as wide)
    let adjusted_ndc_y = ndc_y * 2.0;
    
    let ray_target = Vec3::new(ndc_x * tan_fov, adjusted_ndc_y * tan_fov, -1.0);
    
    // Apply the inverse view matrix transformation
    let inv_view = view_matrix.inverse();
    let world_ray = inv_view.transform_point3(ray_target);
    
    (world_ray - Vec3::zero()).normalize()
}

fn draw(framebuffer: &Arc<Mutex<Framebuffer>>, window: &mut Option<Window>, buffer: &mut Vec<u32>, terminal_buffer: &mut TerminalBuffer, debug_mode: bool) {
    let mut fb = framebuffer.lock().unwrap();
    
    // Compute brightness buffer and gradients
    fb.compute_brightness_buffer(32);
    fb.increase_brightness(1.6);
    fb.increase_contrast(1.25);
    fb.apply_sharpening(1.25);
    fb.apply_bayer_dithering();
    let gradients = compute_gradients(&fb);

    // Render to terminal using ncurses
    draw_colored_frame(&fb, &gradients, terminal_buffer);

    // If in debug mode, render to minifb window as well
    if debug_mode {
        if let Some(ref mut win) = window {
            for (i, pixel) in fb.data.iter().enumerate() {
                buffer[i] = ((pixel.r as u32) << 16) | ((pixel.g as u32) << 8) | (pixel.b as u32);
            }
            win.update_with_buffer(&buffer, fb.width, fb.height).unwrap();
        }
    }
}

// Function to create the framebuffer
fn create_framebuffer() -> Framebuffer {
    let mut width = 0;
    let mut height = 0;
    getmaxyx(stdscr(), &mut height, &mut width);  // Get current terminal size
    Framebuffer::new(width as usize, height as usize)
}