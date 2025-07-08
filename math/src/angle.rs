use std::{
    ops::{Add, AddAssign, Shr, Sub, SubAssign},
    sync::OnceLock,
};

use crate::{
    FRACBITS, FRACUNIT, FloatAngle, VecF2, bam_to_radian, fixed_t,
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

const FINESINE_ENTRIES: u32 = (5 * FINEANGLES / 4);

const DEG_TO_RAD: f32 = std::f32::consts::PI / 180.0;

static FINESIN: OnceLock<[fixed_t; FINESINE_ENTRIES as usize]> = OnceLock::new();

fn init_fine_sine() -> [fixed_t; FINESINE_ENTRIES as usize] {
    let mut result: [fixed_t; FINESINE_ENTRIES as usize] =
        [fixed_t::new(0); FINESINE_ENTRIES as usize];

    for i in 0..(5 * FINEANGLES / 4) {
        let a = ((i as f64 + 0.5) * std::f64::consts::PI * 2.0) / (FINEANGLES as f64);
        let t = (a.sin() * (65536 as f64)) as i32;
        result[i as usize] = fixed_t::new(t);
    }

    result
}

fn get_fine_sin() -> &'static [fixed_t; FINESINE_ENTRIES as usize] {
    FINESIN.get_or_init(|| init_fine_sine())
}

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
        FloatAngle::new(bam_to_radian(self.0))
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
    pub fn finesin(&self) -> fixed_t {
        get_fine_sin()[(self.0 >> 19) as usize]
    }

    #[inline]
    pub fn sin(&self) -> fixed_t {
        get_fine_sin()[(self.0 >> 19) as usize]
    }

    #[inline]
    pub fn finecos(&self) -> fixed_t {
        get_fine_sin()[((self.0 >> 19) + 2048) as usize]
    }

    #[inline]
    pub fn cos(&self) -> fixed_t {
        get_fine_sin()[((self.0 >> 19) + 2048) as usize]
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

impl std::ops::Neg for Angle {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Angle::new(0) - self
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
