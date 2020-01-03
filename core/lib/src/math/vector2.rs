use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

#[macro_export]
macro_rules! vec2 {
    () => {
        Vector2 { x: 0.0, y: 0.0 }
    };
    ($c:expr) => {
        Vector2 { x: $c, y: $c }
    };
    ($x:expr, $y:expr) => {
        Vector2 { x: $x, y: $y }
    };
}

impl Vector2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vector2 { x, y }
    }
    pub fn zero() -> Self {
        Self::new(0., 0.)
    }
    pub fn magnitude(self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn normalized(self) -> Self {
        self / self.magnitude()
    }
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
}

impl Default for Vector2 {
    fn default() -> Self {
        Self::zero()
    }
}

impl Into<[f64; 2]> for Vector2 {
    fn into(self) -> [f64; 2] {
        [self.x, self.y]
    }
}
impl From<[f64; 2]> for Vector2 {
    fn from([x, y]: [f64; 2]) -> Self {
        Self { x, y }
    }
}

impl Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for Vector2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}
impl Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, other: f64) -> Self {
        Self {
            x: self.x * other,
            y: self.y * other,
        }
    }
}
impl Mul<Vector2> for f64 {
    type Output = Vector2;
    fn mul(self, Vector2 { x, y }: Vector2) -> Self::Output {
        Vector2 {
            x: self * x,
            y: self * y,
        }
    }
}
impl Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, other: f64) -> Self {
        Self {
            x: self.x / other,
            y: self.y / other,
        }
    }
}
