use core::ops::Add;
use core::ops::Div;
use core::ops::Mul;
use core::prelude::rust_2024::Clone;
use core::prelude::rust_2024::Copy;
use core::prelude::rust_2024::Debug;
use core::prelude::rust_2024::derive;
use libm::cos;
use libm::sin;

const WIN_HEIGHT: i32 = 64;
const WIN_WIDTH: i32 = 64;

pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

pub struct Vec2f {
    pub x: f64,
    pub y: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct Vec3f {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub struct Triangle {
    pub v1: Vec2,
    pub v2: Vec2,
    pub v3: Vec2,
}

pub struct Trianglef64 {
    pub v1: Vec2f,
    pub v2: Vec2f,
    pub v3: Vec2f,
}
#[derive(Debug, Clone)]
pub struct Triangle3D {
    pub v1: Vec3f,
    pub v2: Vec3f,
    pub v3: Vec3f,
}

impl Vec2f {
    pub fn to_screen(&self) -> Vec2 {
        Vec2 {
            x: ((self.x as f64 + 1.0) * WIN_WIDTH as f64 / 2.0) as i32,
            y: ((-self.y as f64 + 1.0) * WIN_HEIGHT as f64 / 2.0) as i32,
        }
    }
}

impl Trianglef64 {
    pub fn to_screen(&self) -> Triangle {
        Triangle {
            v1: self.v1.to_screen(),
            v2: self.v2.to_screen(),
            v3: self.v3.to_screen(),
        }
    }
}

//////////////////////////////////////////////////////////////////////
////////////////////////IMPLÉMENTATION DE LA 3D///////////////////////
//////////////////////////////////////////////////////////////////////

// Implémentation des opérateurs pour les vecteurs Vec3f
impl Add<Vec3f> for Vec3f {
    type Output = Vec3f;

    fn add(self, v: Vec3f) -> Vec3f {
        Vec3f {
            x: self.x + v.x,
            y: self.y + v.y,
            z: self.z + v.z,
        }
    }
}

impl Mul<f64> for Vec3f {
    type Output = Vec3f;

    fn mul(self, c: f64) -> Vec3f {
        Vec3f {
            x: self.x * c as f64,
            y: self.y * c as f64,
            z: self.z * c as f64,
        }
    }
}

impl Div<f64> for Vec3f {
    type Output = Vec3f;

    fn div(self, c: f64) -> Vec3f {
        Vec3f {
            x: self.x / c as f64,
            y: self.y / c as f64,
            z: self.z / c as f64,
        }
    }
}

impl Vec3f {
    pub fn projection(&self) -> Vec2f {
        Vec2f {
            x: self.x,
            y: self.y,
        } / (self.z) // * focallength
    }

    pub fn rotation_x(&self, pitch: f64) -> Vec3f {
        let y1 = cos(pitch) * self.y - sin(pitch) * self.z;
        let z1 = sin(pitch) * self.y + cos(pitch) * self.z;
        return Vec3f {
            x: self.x,
            y: y1,
            z: z1,
        };
    }

    pub fn rotation_y(&self, yaw: f64) -> Vec3f {
        let x1 = cos(yaw) * self.x + sin(yaw) * self.z;
        let z1 = -sin(yaw) * self.x + cos(yaw) * self.z;
        return Vec3f {
            x: x1,
            y: self.y,
            z: z1,
        };
    }
}

// Implémentation des opérateurs pour les vecteurs Vec2f
impl Add<Vec2f> for Vec2f {
    type Output = Vec2f;

    fn add(self, v: Vec2f) -> Vec2f {
        Vec2f {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl Mul<f64> for Vec2f {
    type Output = Vec2f;

    fn mul(self, c: f64) -> Vec2f {
        Vec2f {
            x: self.x * c,
            y: self.y * c,
        }
    }
}

impl Div<f64> for Vec2f {
    type Output = Vec2f;

    fn div(self, c: f64) -> Vec2f {
        Vec2f {
            x: self.x / c,
            y: self.y / c,
        }
    }
}

impl Triangle3D {
    pub fn projection(&self) -> Trianglef64 {
        Trianglef64 {
            v1: self.v1.projection(),
            v2: self.v2.projection(),
            v3: self.v3.projection(),
        }
    }

    pub fn translate(self, v: Vec3f) -> Triangle3D {
        Triangle3D {
            v1: self.v1 + v.clone(),
            v2: self.v2 + v.clone(),
            v3: self.v3 + v.clone(),
        }
    }

    pub fn rotation_x(&self, pitch: f64) -> Triangle3D {
        Triangle3D {
            v1: self.v1.rotation_x(pitch),
            v2: self.v2.rotation_x(pitch),
            v3: self.v3.rotation_x(pitch),
        }
    }

    pub fn rotation_y(&self, yaw: f64) -> Triangle3D {
        Triangle3D {
            v1: self.v1.rotation_y(yaw),
            v2: self.v2.rotation_y(yaw),
            v3: self.v3.rotation_y(yaw),
        }
    }
}
