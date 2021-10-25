pub(in crate::math) use float_cmp::*;

use super::types::*;

const ULPS: i64 = 2;

#[inline]
pub(in crate::math) fn approx_cmp(x: R, y: R) -> bool {
    approx_eq!(R, x, y, ulps = ULPS)
}

#[inline]
pub(in crate::math) fn approx_real(x: &C) -> bool {
    approx_eq!(R, x.im, 0.0, ulps = ULPS)
}

#[inline]
pub(in crate::math) fn approx_eq(a: &C, b: &C) -> bool {
    approx_eq!(R, a.re, b.re, ulps = ULPS) && approx_eq!(R, a.im, b.im, ulps = ULPS)
}

#[inline]
pub(in crate::math) fn approx_eq_conj(a: &C, b: &C) -> bool {
    approx_eq!(R, a.re, b.re, ulps = ULPS) && approx_eq!(R, a.im, -b.im, ulps = ULPS)
}
