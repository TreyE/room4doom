use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Sub};

use crate::{
    ANG90, ANG180, ANG270, Angle, FT_ZERO, fixed::fixed_t, fixed_tables::tantoangle, fixed_to_float,
};

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
    pub fn exact_length(self) -> fixed_t {
        let dx = self.x.to_float();
        let dy = self.x.to_float();
        fixed_t::from_float((dx * dx + dy * dy).sqrt())
    }

    #[inline]
    pub fn distance(self, other: VecF2) -> fixed_t {
        (other - self).length()
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> fixed_t {
        (self.x * rhs.x) + (self.y * rhs.y)
    }

    #[inline]
    pub const fn to_vec_2(self) -> glam::Vec2 {
        glam::Vec2::new(self.x.to_float(), self.y.to_float())
    }

    pub fn to_angle(self) -> Angle {
        R_PointToAngle(self.x, self.y)
    }
}

pub(crate) fn R_PointToAngleSlope(x1: fixed_t, y1: fixed_t, x: fixed_t, y: fixed_t) -> Angle {
    let mut yt = y - y1;
    let mut xt = x - x1;
    if (xt == FT_ZERO && yt == FT_ZERO) {
        return Angle::new(0);
    }

    if (xt >= FT_ZERO) {
        if (yt >= FT_ZERO) {
            if (xt > yt) {
                return Angle::new(tantoangle[SlopeDiv(yt.0 as u32, xt.0 as u32) as usize]);
            } else {
                return Angle::new(
                    (ANG90 - 1)
                        .wrapping_sub(tantoangle[SlopeDiv(xt.0 as u32, yt.0 as u32) as usize]),
                );
            }
        } else {
            yt = -yt;
            if (xt > yt) {
                return Angle::new(
                    u32::MAX - tantoangle[SlopeDiv(yt.0 as u32, xt.0 as u32) as usize],
                );
            } else {
                return Angle::new(
                    ANG270 + tantoangle[SlopeDiv(xt.0 as u32, yt.0 as u32) as usize],
                );
            }
        }
    } else {
        xt = -xt;
        if (yt >= FT_ZERO) {
            if (xt > yt) {
                return Angle::new(
                    ANG180 - 1 - tantoangle[SlopeDiv(yt.0 as u32, xt.0 as u32) as usize],
                );
            } else {
                return Angle::new(ANG90 + tantoangle[SlopeDiv(xt.0 as u32, yt.0 as u32) as usize]);
            }
        } else {
            yt = -yt;
            if (xt > yt) {
                return Angle::new(
                    ANG180 + tantoangle[SlopeDiv(yt.0 as u32, xt.0 as u32) as usize],
                );
            } else {
                return Angle::new(
                    ANG270 - 1 - tantoangle[SlopeDiv(xt.0 as u32, yt.0 as u32) as usize],
                );
            }
        }
    }
    Angle::new(0)
}

pub(crate) fn R_PointToAngle(x: fixed_t, y: fixed_t) -> Angle {
    R_PointToAngleSlope(FT_ZERO, FT_ZERO, x, y)
}

static SLOPERANGE: u32 = 2048;

pub(crate) fn SlopeDiv(num: u32, den: u32) -> u32 {
    if (den < 512) {
        return SLOPERANGE;
    }

    let ans = (num << 3) / (den >> 8);

    if ans <= SLOPERANGE { ans } else { SLOPERANGE }
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

#[inline]
pub fn point_on_side(ls: &VecF2, le: &VecF2, v: &VecF2) -> usize {
    let ldx = le.x - ls.x;
    let ldy = le.y - ls.y;

    if ldx == FT_ZERO {
        if (v.x <= ls.x) {
            if ldy > FT_ZERO {
                return 1;
            } else {
                return 0;
            }
        } else {
            if ldx < FT_ZERO {
                return 1;
            } else {
                return 0;
            }
        }
    }

    if ldy == FT_ZERO {
        if (v.y <= ls.y) {
            if ldx < FT_ZERO {
                return 1;
            } else {
                return 0;
            }
        } else {
            if ldx > FT_ZERO {
                return 1;
            } else {
                return 0;
            }
        }
    }

    let dx = v.x - ls.x;
    let dy = v.y - ls.y;

    if (ldy.0 ^ ldx.0 ^ dx.0 ^ dy.0) < 0 {
        if (ldy.0 ^ dx.0) < 0 {
            return 1;
        } else {
            return 0;
        }
    } else {
        if (dy * (ldx >> 16)) >= ((ldy >> 16) * dx) {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod test {
    use crate::{ANG45, ANG90, ANG180, ANG270, Angle, FT_ONE, FT_ZERO};

    use super::VecF2;

    #[test]
    fn test_ptoa_values() {
        let righthand = VecF2::new(FT_ONE, FT_ZERO);
        let top = VecF2::new(FT_ZERO, FT_ONE);
        let lefthand = VecF2::new(-FT_ONE, FT_ZERO);
        let bottom = VecF2::new(FT_ZERO, -FT_ONE);
        assert_eq!(righthand.to_angle(), Angle::new(0));
        assert_eq!(top.to_angle(), Angle::new(ANG90 - 1));
        assert_eq!(lefthand.to_angle(), Angle::new(ANG180 - 1));
        assert_eq!(bottom.to_angle(), Angle::new(ANG270));
        let top_right = VecF2::new(FT_ONE, FT_ONE);
        let top_left = VecF2::new(-FT_ONE, FT_ONE);
        let bottom_left = VecF2::new(-FT_ONE, -FT_ONE);
        let bottom_right = VecF2::new(FT_ONE, -FT_ONE);
        assert_eq!(top_right.to_angle(), Angle::new(ANG45 - 1));
        assert_eq!(top_left.to_angle(), Angle::new(ANG45 + ANG90));
        assert_eq!(bottom_left.to_angle(), Angle::new(ANG45 + ANG180 - 1));
        assert_eq!(bottom_right.to_angle(), Angle::new(ANG45 + ANG270));
    }

    #[test]
    fn test_point_on_side_results() {}
}
