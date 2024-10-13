use crate::math::{Mat4, Smoothstep, Vec2, Vec3, Vec4};
use crate::pixel::Pixel;
use std::sync::LazyLock;
use std::sync::Mutex;

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
    let mut globals = GLOBALS.lock().unwrap();
    globals.resolution = resolution;
    globals.time = time;
}

pub fn ray_march(origin: Vec3, direction: Vec3, time: f32, light_dir: Vec3) -> Pixel {
    let globals = GLOBALS.lock().unwrap();
    let anim = 1.1 + 0.5 * (0.1 * globals.time).cos().smoothstep(-0.3, 0.3);
    let col = render(origin, direction, anim, time);
    vec4_to_pixel(col)
}

fn vec4_to_pixel(v: Vec4) -> Pixel {
    Pixel {
        r: (v.x.clamp(0.0, 1.0) * 255.0) as u8,
        g: (v.y.clamp(0.0, 1.0) * 255.0) as u8,
        b: (v.z.clamp(0.0, 1.0) * 255.0) as u8,
        a: 255,
    }
}

static mut ORB: Vec4 = Vec4 { x: 1000.0, y: 1000.0, z: 1000.0, w: 1000.0 };

fn render(ro: Vec3, rd: Vec3, anim: f32, time: f32) -> Vec4 {
    let mut col = Vec3::zero();
    let t = trace(ro, rd, anim);
    if t > 0.0 {
        unsafe {
            let tra = ORB;
            let pos = ro + rd * t;
            let nor = calc_normal(pos, t, anim);

            // lighting
            let light1 = Vec3::new(0.577, 0.577, -0.577);
            let light2 = Vec3::new(-0.707, 0.000, 0.707);
            let key = light1.dot(nor).clamp(0.0, 1.0);
            let bac = (0.2 + 0.8 * light2.dot(nor)).clamp(0.0, 1.0);
            let amb = 0.7 + 0.3 * nor.y;
            let ao = (tra.w * 2.0).clamp(0.0, 1.0).powf(1.2);

            let brdf = Vec3::new(0.40, 0.40, 0.40) * Vec3::splat(amb * ao)
                + Vec3::new(1.00, 1.00, 1.00) * Vec3::splat(key * ao)
                + Vec3::new(0.40, 0.40, 0.40) * Vec3::splat(bac * ao);

            // material
            let mut rgb = Vec3::new(1.0, 1.0, 1.0);
            rgb = rgb.lerp(&Vec3::new(1.0, 0.80, 0.2), (6.0 * tra.y).clamp(0.0, 1.0));
            rgb = rgb.lerp(&Vec3::new(1.0, 0.55, 0.0), (1.0 - 2.0 * tra.z).clamp(0.0, 1.0).powf(8.0));

            // color
            col = rgb * brdf * Vec3::splat((-0.2 * t).exp());
        }
    }
    let col_sqrt = col.sqrt();
    Vec4::new(col_sqrt.x, col_sqrt.y, col_sqrt.z, 1.0)
}

fn map(mut p: Vec3, s: f32) -> f32 {
    let mut scale = 1.0;
    unsafe {
        ORB = Vec4::splat(1000.0);
    }

    for _ in 0..8 {
        p = Vec3::splat(-1.0) + (p * 0.5 + Vec3::splat(0.5)).fract() * 2.0;
        let r2 = p.dot(p);
        
        unsafe {
            ORB = ORB.min(Vec4::new(p.x.abs(), p.y.abs(), p.z.abs(), r2));
        }

        let k = s / r2;
        p = p * Vec3::splat(k);
        scale *= k;
    }

    0.25 * p.y.abs() / scale
}

fn trace(ro: Vec3, rd: Vec3, s: f32) -> f32 {
    let max_dist = 30.0;
    let mut t = 0.01;
    for _ in 0..512 {
        let precis = 0.001 * t;
        let h = map(ro + rd * t, s);
        if h < precis || t > max_dist {
            break;
        }
        t += h;
    }
    if t > max_dist {
        -1.0
    } else {
        t
    }
}

fn calc_normal(pos: Vec3, t: f32, s: f32) -> Vec3 {
    let precis = 0.001 * t;
    let e = Vec2::new(1.0, -1.0) * precis;
    Vec3::new(
        e.x * map(pos + Vec3::new(e.x, 0.0, 0.0), s) + e.y * map(pos + Vec3::new(e.y, 0.0, 0.0), s),
        e.x * map(pos + Vec3::new(0.0, e.x, 0.0), s) + e.y * map(pos + Vec3::new(0.0, e.y, 0.0), s),
        e.x * map(pos + Vec3::new(0.0, 0.0, e.x), s) + e.y * map(pos + Vec3::new(0.0, 0.0, e.y), s),
    ).normalize()
}