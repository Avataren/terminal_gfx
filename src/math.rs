use std::ops::{Add, Sub, Mul, Div};

pub trait Smoothstep {
    fn smoothstep(self, edge0: Self, edge1: Self) -> Self;
}

impl Smoothstep for f32 {
    fn smoothstep(self, edge0: f32, edge1: f32) -> f32 {
        let t = ((self - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
}

pub trait Clamp {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Clamp for f32 {
    fn clamp(self, min: f32, max: f32) -> f32 {
        if self < min { min }
        else if self > max { max }
        else { self }
    }
}

#[derive(Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn lerp<T: Into<Vec2>>(&self, other: T, t: f32) -> Self {
        let other = other.into();
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len != 0.0 {
            *self * (1.0 / len)
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, scalar: f32) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }
}


impl From<&Vec2> for Vec2 {
    fn from(v: &Vec2) -> Self {
        *v
    }
}

#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub fn splat(v: f32) -> Self {
        Self { x: v, y: v, z: v }
    }


    pub fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }


    pub fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn length(&self) -> f32 {
        self.dot(&*self).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len != 0.0 {
            *self * (1.0 / len)
        } else {
            *self
        }
    }

    pub fn abs(&self) -> Self {
        Self::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn max(&self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y), self.z.max(other.z))
    }

    pub fn lerp<T: Into<Vec3>>(&self, other: T, t: f32) -> Self {
        let other = other.into();
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    pub fn sqrt(&self) -> Self {
        Self {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
        }
    }

    pub fn exp(&self) -> Self {
        Self {
            x: self.x.exp(),
            y: self.y.exp(),
            z: self.z.exp(),
        }
    }

    pub fn fract(&self) -> Self {
        Self {
            x: self.x.fract(),
            y: self.y.fract(),
            z: self.z.fract(),
        }
    }

    pub fn clamp(&self, min: f32, max: f32) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
            z: self.z.clamp(min, max),
        }
    }

    pub fn powf(&self, n: f32) -> Self {
        Self {
            x: self.x.powf(n),
            y: self.y.powf(n),
            z: self.z.powf(n),
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Mul for Vec3 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self::new(self.x * other.x, self.y * other.y, self.z * other.z)
    }
}

impl From<&Vec3> for Vec3 {
    fn from(v: &Vec3) -> Self {
        *v
    }
}

#[derive(Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn splat(v: f32) -> Self {
        Self::new(v, v, v, v)
    }

    pub fn min(&self, other: Self) -> Self {
        Self::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
            self.w.min(other.w),
        )
    }

    pub fn lerp<T: Into<Vec4>>(&self, other: T, t: f32) -> Self {
        let other = other.into();
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
            w: self.w + (other.w - self.w) * t,
        }
    }

    pub fn sqrt(&self) -> Self {
        Self {
            x: self.x.sqrt(),
            y: self.y.sqrt(),
            z: self.z.sqrt(),
            w: self.w.sqrt(),
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len != 0.0 {
            *self * (1.0 / len)
        } else {
            *self
        }
    }
}

impl Mul<f32> for Vec4 {
    type Output = Self;
    fn mul(self, scalar: f32) -> Self {
        Self::new(
            self.x * scalar,
            self.y * scalar,
            self.z * scalar,
            self.w * scalar
        )
    }
}

impl From<&Vec4> for Vec4 {
    fn from(v: &Vec4) -> Self {
        *v
    }
}

pub struct Mat4(pub [[f32; 4]; 4]);

impl Mat4 {

    pub fn new(
        m00: f32, m01: f32, m02: f32, m03: f32,
        m10: f32, m11: f32, m12: f32, m13: f32,
        m20: f32, m21: f32, m22: f32, m23: f32,
        m30: f32, m31: f32, m32: f32, m33: f32
    ) -> Self {
        Mat4([
            [m00, m01, m02, m03],
            [m10, m11, m12, m13],
            [m20, m21, m22, m23],
            [m30, m31, m32, m33]
        ])
    }

    pub fn from_rotation_y(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self([
            [cos, 0.0, sin, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [-sin, 0.0, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_rotation_x(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, cos, -sin, 0.0],
            [0.0, sin, cos, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    // New function: rotation around Z-axis
    pub fn from_rotation_z(angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self([
            [cos, -sin, 0.0, 0.0],
            [sin, cos, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    // Utility function to combine rotations
    pub fn from_euler_angles(x_angle: f32, y_angle: f32, z_angle: f32) -> Self {
        let rot_x = Mat4::from_rotation_x(x_angle);
        let rot_y = Mat4::from_rotation_y(y_angle);
        let rot_z = Mat4::from_rotation_z(z_angle);
        rot_z * rot_y * rot_x // Order matters
    }

    pub fn transform_point3(&self, p: Vec3) -> Vec3 {
        let x = self.0[0][0] * p.x + self.0[0][1] * p.y + self.0[0][2] * p.z + self.0[0][3];
        let y = self.0[1][0] * p.x + self.0[1][1] * p.y + self.0[1][2] * p.z + self.0[1][3];
        let z = self.0[2][0] * p.x + self.0[2][1] * p.y + self.0[2][2] * p.z + self.0[2][3];
        Vec3::new(x, y, z)
    }

    pub fn inverse(&self) -> Self {
        // This is a simple implementation and might not be numerically stable for all matrices
        // For a more robust implementation, consider using a full matrix inversion algorithm
        let mut inv = [[0.0; 4]; 4];
        let mat = self.0;
        let det = mat[0][0] * (mat[1][1] * mat[2][2] - mat[2][1] * mat[1][2])
                - mat[0][1] * (mat[1][0] * mat[2][2] - mat[1][2] * mat[2][0])
                + mat[0][2] * (mat[1][0] * mat[2][1] - mat[1][1] * mat[2][0]);
        let inv_det = 1.0 / det;

        inv[0][0] = (mat[1][1] * mat[2][2] - mat[2][1] * mat[1][2]) * inv_det;
        inv[0][1] = (mat[0][2] * mat[2][1] - mat[0][1] * mat[2][2]) * inv_det;
        inv[0][2] = (mat[0][1] * mat[1][2] - mat[0][2] * mat[1][1]) * inv_det;
        inv[1][0] = (mat[1][2] * mat[2][0] - mat[1][0] * mat[2][2]) * inv_det;
        inv[1][1] = (mat[0][0] * mat[2][2] - mat[0][2] * mat[2][0]) * inv_det;
        inv[1][2] = (mat[1][0] * mat[0][2] - mat[0][0] * mat[1][2]) * inv_det;
        inv[2][0] = (mat[1][0] * mat[2][1] - mat[2][0] * mat[1][1]) * inv_det;
        inv[2][1] = (mat[2][0] * mat[0][1] - mat[0][0] * mat[2][1]) * inv_det;
        inv[2][2] = (mat[0][0] * mat[1][1] - mat[1][0] * mat[0][1]) * inv_det;

        Mat4(inv)
    }    
}

impl Mul for Mat4 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        let mut result = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        Self(result)
    }
}
