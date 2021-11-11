use crate::{math::{C, R, N}, operator::{atomic, single::*}};

#[inline(always)]
pub fn swap(ab_mask: N) -> Option<SingleOp> {
    single_op_checked!(atomic::swap::Op::new(ab_mask))
}

#[inline(always)]
pub fn sqrt_swap(ab_mask: N) -> Option<SingleOp> {
    single_op_checked!(atomic::sqrt_swap::Op::new(ab_mask))
}

#[inline(always)]
pub fn i_swap(ab_mask: N) -> Option<SingleOp> {
    single_op_checked!(atomic::i_swap::Op::new(ab_mask))
}

#[inline(always)]
pub fn sqrt_i_swap(ab_mask: N) -> Option<SingleOp> {
    single_op_checked!(atomic::sqrt_i_swap::Op::new(ab_mask))
}
