use super::*;

#[inline(always)]
pub fn x(a_mask: Mask) -> SingleOp {
    X::new(a_mask).into()
}

#[inline(always)]
pub fn y(a_mask: Mask) -> SingleOp {
    Y::new(a_mask).into()
}

#[inline(always)]
pub fn z(a_mask: Mask) -> SingleOp {
    Z::new(a_mask).into()
}

#[inline(always)]
pub fn s(a_mask: Mask) -> SingleOp {
    S::new(a_mask).into()
}

#[inline(always)]
pub fn t(a_mask: Mask) -> SingleOp {
    T::new(a_mask).into()
}

#[inline(always)]
pub fn u1(a_mask: Mask, matrix: M1) -> Option<SingleOp> {
    single_op_checked!(U1::new(a_mask, matrix))
}

#[inline(always)]
pub fn u2(a_mask: Mask, b_mask: Mask, matrix: M2) -> Option<SingleOp> {
    single_op_checked!(U2::new(a_mask, b_mask, matrix))
}
