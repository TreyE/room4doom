use std::{
    f32, i32,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shr, Sub, SubAssign},
};

use crate::{
    Angle, FRACUNIT, fixed_to_float, float_to_fixed,
    trig::{COS_TABLE, SIN_TABLE},
};

#[inline(always)]
#[must_use]
pub const fn fixedt(x: i32) -> fixed_t {
    fixed_t::new(x)
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub struct fixed_t(pub i32);

const FRACBITS: i32 = 16;
pub const FT_ZERO: fixed_t = fixed_t::new(0);
pub const FT_FOURTH: fixed_t = fixed_t::new(1 << 14);
pub const FT_ONE: fixed_t = fixed_t::from_int(1);
pub const FT_TWO: fixed_t = fixed_t::from_int(2);
pub const FT_FOUR: fixed_t = fixed_t::from_int(4);
pub const FT_EIGHT: fixed_t = fixed_t::from_int(8);
pub const FT_SIXTEEN: fixed_t = fixed_t::from_int(16);
pub const FT_MAX: fixed_t = fixed_t::new(i32::MAX);

impl fixed_t {
    pub const fn to_float(self) -> f32 {
        fixed_to_float(self.0)
    }

    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        (self.0.abs() >> 24) as u8
    }

    #[inline(always)]
    pub const fn from_i16(v: i16) -> Self {
        Self::new((v as i32) * 65536)
    }

    #[inline]
    pub const fn from_float(v: f32) -> Self {
        fixed_t::new(float_to_fixed(v))
    }

    pub const fn from_int(v: i32) -> Self {
        fixed_t::new(v * 65536)
    }

    pub const fn new(v: i32) -> Self {
        fixed_t(v)
    }

    pub const fn abs(self) -> Self {
        fixed_t(self.0.abs())
    }

    pub const fn is_sign_negative(self) -> bool {
        self.0.is_negative()
    }

    pub const fn to_int(self) -> i32 {
        self.0 / 65536
    }
}

impl Neg for fixed_t {
    type Output = fixed_t;

    fn neg(self) -> Self::Output {
        fixedt(self.0.neg())
    }
}

impl Shr<usize> for fixed_t {
    type Output = fixed_t;

    fn shr(self, rhs: usize) -> Self::Output {
        fixed_t::new(self.0 >> rhs)
    }
}

impl Add<fixed_t> for fixed_t {
    type Output = fixed_t;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        fixed_t::new(self.0 + rhs.0)
    }
}

impl Sub for fixed_t {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        fixed_t::new(self.0 - rhs.0)
    }
}

impl SubAssign for fixed_t {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

impl AddAssign for fixed_t {
    fn add_assign(&mut self, rhs: Self) {
        self.0 -= self.0 + rhs.0;
    }
}

impl Mul for fixed_t {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        fixed_t::new(((self.0 as i64 * rhs.0 as i64) >> 16) as i32)
    }
}

impl Div for fixed_t {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        if (self.0.abs() >> 14) >= rhs.0.abs() {
            if (self.0 ^ rhs.0) < 0 {
                fixed_t::new(i32::MIN)
            } else {
                fixed_t::new(i32::MAX)
            }
        } else {
            fixed_t::new((((self.0 as i64) << 16) / (rhs.0 as i64)) as i32)
        }
    }
}

impl MulAssign for fixed_t {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.mul(rhs).0;
    }
}

impl DivAssign for fixed_t {
    fn div_assign(&mut self, rhs: fixed_t) {
        self.0 = self.div(rhs).0;
    }
}

impl PartialEq for fixed_t {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for fixed_t {}

impl Ord for fixed_t {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for fixed_t {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Default for fixed_t {
    #[inline(always)]
    fn default() -> Self {
        FT_ZERO
    }
}

impl std::fmt::Display for fixed_t {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(p) = f.precision() {
            write!(f, "[{:.*}]", p, self.0)
        } else {
            write!(f, "[{}]", self.0)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::FT_ONE;

    use super::fixed_t;

    #[test]
    fn test_fmul() {
        let result = fixed_t::from_int(2) * FT_ONE;
        assert_eq!(result, fixed_t::from_int(2))
    }
}
