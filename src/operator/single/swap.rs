use crate::{
    operator::{atomic, single::*},
    math::{C, R, N},
};

#[inline(always)]
pub (crate) fn swap(ab_mask: N) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::swap::Op::new(ab_mask))
}

#[inline(always)]
pub (crate) fn sqrt_swap(ab_mask: N) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::sqrt_swap::Op::new(ab_mask))
}

#[inline(always)]
pub (crate) fn i_swap(ab_mask: N) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::i_swap::Op::new(ab_mask))
}

#[inline(always)]
pub (crate) fn sqrt_i_swap(ab_mask: N) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::sqrt_i_swap::Op::new(ab_mask))
}