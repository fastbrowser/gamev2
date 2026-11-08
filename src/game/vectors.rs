#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Vector2i {
    pub x: i32,
    pub y: i32,
}
impl Vector2i {
    pub const ZERO: Self = Self { x: 0, y: 0 };
    pub const ONE: Self = Self { x: 1, y: 1 };
    pub fn from(vec: Vector2f) -> Self {
        Self { x: vec.x as i32, y: vec.y as i32 }
    }
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
    pub fn addvec(&self, vec: Vector2i) -> Vector2i {
        Vector2i { x: self.x+vec.x, y: self.y+vec.y }
    }
    pub fn add(&self, x: i32, y: i32) -> Vector2i {
        Vector2i { x: self.x+x, y: self.y+y }
    }
    pub fn mulvec(&self, vec: Vector2i) -> Vector2i {
        Vector2i { x: self.x*vec.x, y: self.y*vec.y }
    }
    pub fn mul(&self, x: i32, y: i32) -> Vector2i {
        Vector2i { x: self.x*x, y: self.y*y }
    }
    pub fn subvec(&self, vec: Vector2i) -> Vector2i {
        Vector2i { x: self.x-vec.x, y: self.y-vec.y }
    }
    pub fn sub(&self, x: i32, y: i32) -> Vector2i {
        Vector2i { x: self.x-x, y: self.y-y }
    }
    pub fn distsqrvec(&self, vec: Vector2i) -> i32 {
        (self.x-vec.x).pow(2)+(self.y-vec.y).pow(2)
    }
    pub fn distsqr(&self, x: i32, y: i32) -> i32 {
        (self.x-x).pow(2)+(self.y-y).pow(2)
    }
}

impl std::ops::Add for Vector2i {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl std::ops::Sub for Vector2i {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2f {
    pub x: f32,
    pub y: f32,
}
impl Vector2f {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    pub fn from(vec: Vector2i) -> Self {
        Self { x: vec.x as f32, y: vec.y as f32 }
    }
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn addvec(&self, vec: Vector2f) -> Vector2f {
        Vector2f { x: self.x+vec.x, y: self.y+vec.y }
    }
    pub fn add(&self, x: f32, y: f32) -> Vector2f {
        Vector2f { x: self.x+x, y: self.y+y }
    }
    pub fn mulvec(&self, vec: Vector2f) -> Vector2f {
        Vector2f { x: self.x*vec.x, y: self.y*vec.y }
    }
    pub fn mul(&self, x: f32, y: f32) -> Vector2f {
        Vector2f { x: self.x*x, y: self.y*y }
    }
    pub fn subvec(&self, vec: Vector2f) -> Vector2f {
        Vector2f { x: self.x-vec.x, y: self.y-vec.y }
    }
    pub fn sub(&self, x: f32, y: f32) -> Vector2f {
        Vector2f { x: self.x-x, y: self.y-y }
    }
    pub fn distsqrvec(&self, vec: Vector2f) -> f32 {
        (self.x-vec.x).powi(2)+(self.y-vec.y).powi(2)
    }
    pub fn distsqr(&self, x: f32, y: f32) -> f32 {
        (self.x-x).powi(2)+(self.y-y).powi(2)
    }
    pub fn lerp(&self, other: Vector2f, t: f32) -> Vector2f {
        Vector2f {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl std::ops::Add for Vector2f {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl std::ops::Sub for Vector2f {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}