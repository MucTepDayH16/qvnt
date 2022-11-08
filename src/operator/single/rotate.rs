use crate::{
    math::types::*,
    operator::{atomic, single::*},
};

#[inline(always)]
pub fn rx(a_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::rx::Op::new(a_mask, phase))
}

#[inline(always)]
pub fn rxx(ab_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::rxx::Op::new(ab_mask, phase))
}

#[inline(always)]
pub fn ry(a_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::ry::Op::new(a_mask, phase))
}

#[inline(always)]
pub fn ryy(ab_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::ryy::Op::new(ab_mask, phase))
}

#[inline(always)]
pub fn rz(a_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::rz::Op::new(a_mask, phase))
}

#[inline(always)]
pub fn rzz(ab_mask: N, phase: R) -> Option<SingleOp> {
    single_op_checked!(atomic::rzz::Op::new(ab_mask, phase))
}
