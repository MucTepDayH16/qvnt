use crate::operator::{atomic, single::*};
use crate::math::{C, R, N};

#[inline(always)]
pub (crate) fn swap(ab_mask: N) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::swap::Op::new(ab_mask))
}

#[inline(always)]
pub (crate) fn sqrt_swap(ab_mask: N) -> Option<SingleOp> {
    todo!()
}

#[inline(always)]
pub (crate) fn i_swap(ab_mask: N) -> Option<SingleOp> {
    todo!()
}

#[inline(always)]
pub (crate) fn sqrt_i_swap(ab_mask: N) -> Option<SingleOp> {
    todo!()
}