use super::*;

#[inline(always)]
pub fn swap(ab_mask: Mask) -> Option<SingleOp> {
    single_op_checked!(Swap::new(ab_mask))
}

#[inline(always)]
pub fn sqrt_swap(ab_mask: Mask) -> Option<SingleOp> {
    single_op_checked!(SqrtSwap::new(ab_mask))
}

#[inline(always)]
pub fn i_swap(ab_mask: Mask) -> Option<SingleOp> {
    single_op_checked!(ISwap::new(ab_mask))
}

#[inline(always)]
pub fn sqrt_i_swap(ab_mask: Mask) -> Option<SingleOp> {
    single_op_checked!(SqrtISwap::new(ab_mask))
}
