use float_cmp::*;

use super::types::*;

const ULPS: i64 = 2;

#[inline]
pub fn approx_eq_real(x: R, y: R) -> bool {
    approx_eq!(R, x, y, ulps = ULPS)
}

#[inline]
pub fn approx_real(x: &C) -> bool {
    approx_eq_real(x.im, 0.0)
}

#[inline]
pub fn approx_eq_complex(a: &C, b: &C) -> bool {
    approx_eq_real(a.re, b.re) && approx_eq_real(a.im, b.im)
}

#[inline]
pub fn approx_eq_conj(a: &C, b: &C) -> bool {
    approx_eq_real(a.re, b.re) && approx_eq_real(a.im, -b.im)
}
