use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use crate::fixed::fixed_t;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct VecF2 {
    pub x: fixed_t,
    pub y: fixed_t,
}

impl VecF2 {
    #[inline(always)]
    pub const fn new(x: fixed_t, y: fixed_t) -> Self {
        VecF2 { x, y }
    }

    #[inline]
    pub fn length(self) -> fixed_t {
        let dx = self.x.abs();
        let dy = self.y.abs();
        if dx < dy {
            return dx + dy - (dx >> 1);
        } else {
            dx + dy - (dy >> 1)
        }
    }

    #[inline]
    pub fn distance(self, other: VecF2) -> fixed_t {
        (self - other).length()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> fixed_t {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    #[inline]
    pub const fn to_vec_2(self) -> glam::Vec2 {
        glam::Vec2::new(self.x.to_float(), self.y.to_float())
    }
}

impl Sub for VecF2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        VecF2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Add for VecF2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        VecF2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for VecF2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl Mul<fixed_t> for VecF2 {
    type Output = VecF2;

    fn mul(self, rhs: fixed_t) -> Self::Output {
        VecF2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::fmt::Display for VecF2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(p) = f.precision() {
            write!(f, "[{:.*}, {:.*}]", p, self.x, p, self.y)
        } else {
            write!(f, "[{}, {}]", self.x, self.y)
        }
    }
}

impl Div<fixed_t> for VecF2 {
    type Output = VecF2;

    fn div(self, rhs: fixed_t) -> Self::Output {
        VecF2::new(self.x / rhs, self.y / rhs)
    }
}

impl MulAssign<fixed_t> for VecF2 {
    fn mul_assign(&mut self, rhs: fixed_t) {
        self.x = self.x.mul(rhs);
        self.y = self.y.mul(rhs);
    }
}

impl std::fmt::Debug for VecF2 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_tuple(stringify!(Vec2))
            .field(&self.x)
            .field(&self.y)
            .finish()
    }
}
