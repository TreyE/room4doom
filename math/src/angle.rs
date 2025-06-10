use std::ops::{Add, AddAssign, Shr, Sub, SubAssign};

use crate::{
    FRACBITS, FRACUNIT, FloatAngle, VecF2, fixed_t,
    trig::{COS_TABLE, SIN_TABLE},
};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Angle(pub u32);

pub const ANG45: u32 = 0x20000000;
pub const ANG90: u32 = 0x40000000;
pub const ANG180: u32 = 0x80000000;
pub const ANG270: u32 = 0xc0000000;
pub const ANG5: u32 = ANG45 / 45 * 5;
pub const FINEANGLES: u32 = 8192;
pub const FINEMASK: u32 = FINEANGLES - 1;

pub const ANG1: u32 = ANG45 / 45;

const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;

use crate::fixed_tables::finesine_source;

/*

#define ANG135  0x60000000
#define ANG225  0xa0000000
#define ANG315  0xe0000000
#define ANG1      (ANG45/45)
#define ANG60     (ANG180 / 3)
#define ANGLE_MAX 0xffffffff
#ifndef M_PI
#define M_PI    3.14159265358979323846
#endif

#define FIXED_PI 205887
*/

impl Angle {
    pub const fn to_float_angle(self) -> FloatAngle {
        FloatAngle::new((self.0 as f32 * 8.381_903e-8) * DEG_TO_RAD)
    }

    pub const fn from_int(v: i32) -> Self {
        Self::new(v as u32)
    }

    pub const fn from_i16(v: i16) -> Self {
        Self::new(ANG45 * (v / 45) as u32)
    }

    pub const fn new(v: u32) -> Self {
        Angle(v)
    }

    pub const fn to_fixed(self) -> fixed_t {
        fixed_t::new((((self.0 as u64) << FRACBITS) / (ANG1 as u64)) as i32)
    }

    #[inline]
    pub fn sin_cos(&self) -> (fixed_t, fixed_t) {
        (self.sin(), self.cos())
    }

    #[inline]
    pub fn sin(&self) -> fixed_t {
        fixed_t::new(finesine_source[(self.0 >> 19) as usize])
    }

    #[inline]
    pub fn cos(&self) -> fixed_t {
        fixed_t::new(finesine_source[((self.0 >> 19) + 2048) as usize])
    }

    #[inline(always)]
    pub fn unit(&self) -> VecF2 {
        let (y, x) = self.sin_cos();
        VecF2::new(x, y)
    }
}

impl Sub for Angle {
    type Output = Angle;

    fn sub(self, rhs: Self) -> Self::Output {
        Angle::new(self.0.wrapping_sub(rhs.0))
    }
}

impl std::cmp::PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Add for Angle {
    type Output = Angle;

    fn add(self, rhs: Self) -> Self::Output {
        Angle::new(self.0.wrapping_add(rhs.0))
    }
}

impl SubAssign<Angle> for Angle {
    fn sub_assign(&mut self, rhs: Angle) {
        self.0 = self.0.wrapping_sub(rhs.0)
    }
}

impl AddAssign<Angle> for Angle {
    fn add_assign(&mut self, rhs: Angle) {
        self.0 = self.0.wrapping_add(rhs.0)
    }
}

impl Shr<usize> for Angle {
    type Output = Angle;

    fn shr(self, rhs: usize) -> Self::Output {
        Angle::new(self.0 >> rhs)
    }
}

impl std::fmt::Display for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}
