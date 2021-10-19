use crate::{
    operator::{atomic, single::*},
    math::{C, R, N},
};

#[inline(always)]
pub (crate) fn rx(a_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::rx::Op::new(a_mask, phase))
}

#[inline(always)]
pub (crate) fn rxx(ab_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::rxx::Op::new(ab_mask, phase))
}

#[inline(always)]
pub (crate) fn ry(a_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::ry::Op::new(a_mask, phase))
}

#[inline(always)]
pub (crate) fn ryy(ab_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::ryy::Op::new(ab_mask, phase))
}

#[inline(always)]
pub (crate) fn rz(a_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::rz::Op::new(a_mask, phase))
}

#[inline(always)]
pub (crate) fn rzz(ab_mask: N, phase: R) -> Option<SingleOp> {
    SingleOp::from_atomic(atomic::rzz::Op::new(ab_mask, phase))
}