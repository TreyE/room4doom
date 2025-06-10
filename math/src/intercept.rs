use crate::{FT_ZERO, VecF2, fixed_t, vec_f2::point_on_side};
use glam::Vec2;

/// Used in path tracing for intercepts
/// Is divline + trace types
#[derive(Debug, Clone, Copy)]
pub struct Trace {
    pub xy: VecF2,
    pub dxy: VecF2,
}

impl Trace {
    #[inline]
    pub const fn new(xyz: VecF2, dxyz: VecF2) -> Self {
        Self { xy: xyz, dxy: dxyz }
    }
}

/// Returns the fractional intercept point along the first divline.
///
/// The lines can be pictured as arg1 being an infinite plane, and arg2 being
/// the line to check if intersected by the plane.
///
/// P_InterceptVector
#[inline]
pub fn intercept_vector(v2: Trace, v1: Trace) -> fixed_t {
    // Doom does `v1->dy >> 8`, this is  x * 0.00390625
    let denominator = ((v1.dxy.y >> 8) * v2.dxy.x) - ((v1.dxy.x >> 8) * v2.dxy.y);
    if denominator == FT_ZERO {
        return FT_ZERO;
    }
    let numerator =
        (((v1.xy.x - v2.xy.x) >> 8) * v1.dxy.y) + (((v2.xy.y - v1.xy.y) >> 8) * v1.dxy.x);
    numerator / denominator
}
