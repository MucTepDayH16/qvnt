use crate::{
    math::types::*,
    operator::{atomic, single::*},
};

#[inline(always)]
pub fn x(a_mask: N) -> SingleOp {
    atomic::x::Op::new(a_mask).into()
}

#[inline(always)]
pub fn y(a_mask: N) -> SingleOp {
    atomic::y::Op::new(a_mask).into()
}

#[inline(always)]
pub fn z(a_mask: N) -> SingleOp {
    atomic::z::Op::new(a_mask).into()
}

#[inline(always)]
pub fn s(a_mask: N) -> SingleOp {
    atomic::s::Op::new(a_mask).into()
}

#[inline(always)]
pub fn t(a_mask: N) -> SingleOp {
    atomic::t::Op::new(a_mask).into()
}

#[deprecated(since = "0.4.3", note = "it is overhead, use `rz` instead")]
#[inline(always)]
pub fn phi(phases: Vec<(R, N)>) -> SingleOp {
    atomic::phi::Op::new(phases).into()
}

#[inline(always)]
pub fn u1(a_mask: N, matrix: M1) -> Option<SingleOp> {
    single_op_checked!(atomic::u1::Op::new(a_mask, matrix))
}

#[inline(always)]
pub fn u2(a_mask: N, b_mask: N, matrix: M2) -> Option<SingleOp> {
    single_op_checked!(atomic::u2::Op::new(a_mask, b_mask, matrix))
}
