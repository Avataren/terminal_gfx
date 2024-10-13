// raymarch.rs

use crate::math::{Vec2, Vec3, Vec4, Mat4};
use crate::pixel::Pixel;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::f32::consts::PI;

struct ShaderGlobals {
    resolution: Vec2,
    time: f32,
}

static GLOBALS: LazyLock<Mutex<ShaderGlobals>> = LazyLock::new(|| {
    Mutex::new(ShaderGlobals {
        resolution: Vec2::new(0.0, 0.0),
        time: 0.0,
    })
});

pub fn update_globals(resolution: Vec2, time: f32) {
    if let Ok(mut globals) = GLOBALS.lock() {
        globals.resolution = resolution;
        globals.time = time;
    } else {
        eprintln!("Failed to lock GLOBALS mutex.");
    }
}

pub fn ray_march(origin: Vec3, direction: Vec3, time: f32) -> Pixel {
    let globals = GLOBALS.lock().unwrap();
    let resolution = globals.resolution;
    drop(globals); // Release the lock early

    // Define light parameters
    let light_radius = 15.0; // Radius of the circular path
    let light_height = 15.0; // Height above the cubes
    let light_speed = 0.5; // Radians per second

    // Calculate light position moving in a circle
    let light_angle = time * light_speed;
    let light_pos = Vec3::new(
        light_radius * light_angle.cos(),
        light_height,
        light_radius * light_angle.sin(),
    );

    // Raymarching setup
    let max_steps = 200;
    let max_dist = 2000.0;
    let epsilon = 0.001;

    let mut t = 0.0;
    for _ in 0..max_steps {
        let p = origin + direction * t;
        let d = scene_sdf(p, time);
        if d < epsilon {
            // Hit detected
            let normal = calculate_normal(p, time);
            // Compute light direction from p to light_pos
            let to_light = (light_pos - p).normalize();
            let distance_to_light = (light_pos - p).length();
            // Compute shadow factor
            let shadow = soft_shadow(p, to_light, distance_to_light, time);
            // Shade the point
            let color = shade(p, normal, direction, to_light, shadow, distance_to_light);
            return vec3_to_pixel(color);
        }
        t += d;
        if t > max_dist {
            break;
        }
    }

    // Background color (sky)
    let t = 0.5 * (direction.y + 1.0);
    let sky_color = Vec3::new(0.5, 0.7, 1.0).lerp(&Vec3::new(1.0, 1.0, 1.0), t);
    vec3_to_pixel(sky_color)
}

fn scene_sdf(p: Vec3, time: f32) -> f32 {
    let plane_sdf = p.y + 1.0;

    let cube_size = 0.5;

    // Define rotation speeds for each axis (radians per second)
    // Each cube has its own rotation speed
    // Cube 1
    let rot_speed1_x = 0.5;
    let rot_speed1_y = 0.8;
    let rot_speed1_z = 0.3;

    // Cube 2
    let rot_speed2_x = 0.3;
    let rot_speed2_y = 0.6;
    let rot_speed2_z = 0.9;

    // Cube 3
    let rot_speed3_x = 0.7;
    let rot_speed3_y = 0.4;
    let rot_speed3_z = 0.5;

    // Rotation angles based on time and speeds
    let angle1_x = time * rot_speed1_x;
    let angle1_y = time * rot_speed1_y;
    let angle1_z = time * rot_speed1_z;

    let angle2_x = time * rot_speed2_x;
    let angle2_y = time * rot_speed2_y;
    let angle2_z = time * rot_speed2_z;

    let angle3_x = time * rot_speed3_x;
    let angle3_y = time * rot_speed3_y;
    let angle3_z = time * rot_speed3_z;

    // Define fixed positions for the cubes
    let cube1_pos = Vec3::new(-1.5, cube_size, 0.0);
    let cube2_pos = Vec3::new(1.5, cube_size, 0.0);
    let cube3_pos = Vec3::new(0.0, cube_size, 1.732); // Positioned to form an equilateral triangle

    // Apply rotations to each cube
    let rotated_p1 = rotate_all_axes(p - cube1_pos, angle1_x, angle1_y, angle1_z);
    let rotated_p2 = rotate_all_axes(p - cube2_pos, angle2_x, angle2_y, angle2_z);
    let rotated_p3 = rotate_all_axes(p - cube3_pos, angle3_x, angle3_y, angle3_z);

    // Compute SDFs for each cube
    let cube1_sdf = box_sdf(rotated_p1, Vec3::new(cube_size, cube_size, cube_size));
    let cube2_sdf = box_sdf(rotated_p2, Vec3::new(cube_size, cube_size, cube_size));
    let cube3_sdf = box_sdf(rotated_p3, Vec3::new(cube_size, cube_size, cube_size));

    // Combine SDFs: plane and cubes
    plane_sdf.min(cube1_sdf).min(cube2_sdf).min(cube3_sdf)
}

// Function to rotate a point around all three axes
fn rotate_all_axes(p: Vec3, angle_x: f32, angle_y: f32, angle_z: f32) -> Vec3 {
    let rot_matrix = Mat4::from_euler_angles(angle_x, angle_y, angle_z);
    rot_matrix.transform_point3(p)
}

fn box_sdf(p: Vec3, b: Vec3) -> f32 {
    let q = Vec3::new(p.x.abs(), p.y.abs(), p.z.abs()) - b;
    q.max(Vec3::new(0.0, 0.0, 0.0)).length() + q.x.max(q.y.max(q.z)).min(0.0)
}

fn calculate_normal(p: Vec3, time: f32) -> Vec3 {
    let epsilon = 0.001;
    Vec3::new(
        scene_sdf(Vec3::new(p.x + epsilon, p.y, p.z), time) - scene_sdf(Vec3::new(p.x - epsilon, p.y, p.z), time),
        scene_sdf(Vec3::new(p.x, p.y + epsilon, p.z), time) - scene_sdf(Vec3::new(p.x, p.y - epsilon, p.z), time),
        scene_sdf(Vec3::new(p.x, p.y, p.z + epsilon), time) - scene_sdf(Vec3::new(p.x, p.y, p.z - epsilon), time)
    ).normalize()
}

// Soft shadow function adjusted for point light
fn soft_shadow(p: Vec3, light_dir: Vec3, distance_to_light: f32, time: f32) -> f32 {
    let mut t = 0.01; // Start slightly offset to avoid self-shadowing
    let max_dist = distance_to_light; // Only check up to the light source
    let mut shadow = 1.0;
    let k = 2.0; // Reduced softness factor for smoother shadows

    for _ in 0..100 {
        let current_p = p + light_dir * t;
        let dist = scene_sdf(current_p, time);
        if dist < 0.001 {
            // Occluder found
            shadow *= 1.0 - (t / max_dist).powf(k);
            break;
        }
        t += dist;
        if t > max_dist {
            break;
        }
    }

    // Clamp shadow factor between 0 and 1
    shadow.clamp(0.0, 1.0)
}

fn shade(
    p: Vec3,
    normal: Vec3,
    view_dir: Vec3,
    light_dir: Vec3,
    shadow: f32,
    distance_to_light: f32
) -> Vec3 {
    let light_color = Vec3::new(1.0, 1.0, 1.0);
    let object_color = if p.y < -0.99 {
        // Checkerboard floor
        let pattern = ((p.x * 0.25).floor() as i32 + (p.z * 0.25).floor() as i32) & 1;
        if pattern == 0 {
            Vec3::new(0.12, 0.14, 0.16)
        } else {
            Vec3::new(0.9, 0.95, 0.99)
        }
    } else {
        // Cube color based on position
        Vec3::new(
            (p.x.sin() * 0.5 + 0.5),
            (p.y.sin() * 0.5 + 0.5),
            (p.z.sin() * 0.5 + 0.5)
        )
    };

    let ambient = 0.1;

    // Diffuse lighting
    let diffuse = normal.dot(&light_dir).max(0.0) * shadow;

    // Light attenuation with scaling factor
    let light_intensity = 500.0; 
    let attenuation = light_intensity / (distance_to_light * distance_to_light + 1.0);

    // Final color with attenuation
    object_color * light_color * (ambient + diffuse) * attenuation
}

fn vec3_to_pixel(v: Vec3) -> Pixel {
    Pixel {
        r: (v.x.clamp(0.0, 1.0) * 255.0) as u8,
        g: (v.y.clamp(0.0, 1.0) * 255.0) as u8,
        b: (v.z.clamp(0.0, 1.0) * 255.0) as u8,
        a: 255,
    }
}
