mod framebuffer;
mod sobel;
mod terminal;
mod ascii;
mod pixel;

use ncurses::*;
use std::env;
use crate::framebuffer::Framebuffer;
use crate::sobel::compute_gradients;
use crate::terminal::draw_colored_frame;
use crate::pixel::Pixel;

use std::f32::consts::PI;
use minifb::{Window, WindowOptions};
use std::time::Instant;


fn main() {
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.contains(&"--debug".to_string());

    initscr();  // Start the ncurses session
    noecho();   // Disable echoing of characters
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);  // Hide the cursor
    nodelay(stdscr(), true);  // Don't block the getch call

    // Create framebuffer and window dimensions based on terminal size
    let mut framebuffer = create_framebuffer();
    let mut paused = false; // Track whether the animation is paused
    let target_fps = 60.0;
    let mut last_time = Instant::now();

    // Initialize minifb window for debug mode
    let mut window = if debug_mode {
        Some(Window::new(
            "Debug Framebuffer - ESC to exit",
            framebuffer.width,
            framebuffer.height,
            WindowOptions::default(),
        ).unwrap_or_else(|e| {
            panic!("{}", e);
        }))
    } else {
        None
    };

    // Minifb buffer for graphical rendering (used only in debug mode)
    let mut buffer = if debug_mode {
        vec![0; framebuffer.width * framebuffer.height]
    } else {
        vec![]
    };

    let mut total_elapsed_time = 0.0;
    let start_time = Instant::now();
    
    let mut prev_width = framebuffer.width;
    let mut prev_height = framebuffer.height;

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
            framebuffer = Framebuffer::new(new_width_usize, new_height_usize);
            prev_width = new_width_usize;
            prev_height = new_height_usize;

            clear();  // Clear the screen after resizing
        }

        if !paused {
            framebuffer.clear();  // Clear framebuffer before drawing
            update(delta_time, total_elapsed_time, &mut framebuffer);
            draw(&mut framebuffer, &mut window, &mut buffer, debug_mode);        
        }
        

        // Sleep to maintain the target framerate
        let elapsed_time = now.elapsed().as_secs_f32();
        let sleep_time = (1.0 / target_fps - elapsed_time).max(0.0);
        std::thread::sleep(std::time::Duration::from_secs_f32(sleep_time));
    }

    endwin();  // End the ncurses session
}

fn update(delta_time:f32, total_time:f32, framebuffer: &mut Framebuffer)
{
    draw_test_scene(framebuffer, total_time);
}

fn draw_test_scene(framebuffer: &mut Framebuffer, total_time:f32)
{
    for x in 0..framebuffer.width {
        let normalized_x = x as f32 / framebuffer.width as f32;
        let mut sine_value = (total_time + normalized_x * 2.0 * PI).sin();
        sine_value *= (total_time*0.5 + normalized_x * 3.0 * PI).cos();        
        let y = ((sine_value + 1.0) * 0.5 * framebuffer.height as f32) as usize;

        // Interpolate color along the x-axis (e.g., from blue to red)
        let r = (normalized_x * 255.0) as u8;
        let b = ((1.0 - normalized_x) * 255.0) as u8;
        let color = Pixel { r, g: 0, b, a: 255 };

        // Fill the sine wave area with the gradient
        for fill_y in y..framebuffer.height {
            framebuffer.set_pixel(x, fill_y, color.clone());
        }
        for fill_y in 0..y {
            framebuffer.set_pixel(x, fill_y, Pixel{r:32,g:46,b:64,a:255});
        }
    }    
}

// Drawing the frame
fn draw(framebuffer: &mut Framebuffer, window: &mut Option<Window>, buffer: &mut Vec<u32>, debug_mode: bool) {
    // Compute brightness buffer and gradients
    framebuffer.compute_brightness_buffer(255);
    framebuffer.increase_brightness(4.0);
    framebuffer.increase_contrast(1.5);
    framebuffer.apply_sharpening(0.5);
    let gradients = compute_gradients(&framebuffer);

    // Render to terminal using ncurses
    draw_colored_frame(&framebuffer, &gradients);

    // If in debug mode, render to minifb window as well
    if debug_mode {
        if let Some(ref mut win) = window {
            for (i, pixel) in framebuffer.data.iter().enumerate() {
                buffer[i] = ((pixel.r as u32) << 16) | ((pixel.g as u32) << 8) | (pixel.b as u32);
            }
            win.update_with_buffer(&buffer, framebuffer.width, framebuffer.height).unwrap();
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
