use crate::{
    math::types::*,
    operator::{atomic, single::*},
};

#[inline(always)]
pub fn x(a_mask: Mask) -> SingleOp {
    atomic::x::Op::new(a_mask).into()
}

#[inline(always)]
pub fn y(a_mask: Mask) -> SingleOp {
    atomic::y::Op::new(a_mask).into()
}

#[inline(always)]
pub fn z(a_mask: Mask) -> SingleOp {
    atomic::z::Op::new(a_mask).into()
}

#[inline(always)]
pub fn s(a_mask: Mask) -> SingleOp {
    atomic::s::Op::new(a_mask).into()
}

#[inline(always)]
pub fn t(a_mask: Mask) -> SingleOp {
    atomic::t::Op::new(a_mask).into()
}

#[inline(always)]
pub fn u1(a_mask: Mask, matrix: M1) -> Option<SingleOp> {
    single_op_checked!(atomic::u1::Op::new(a_mask, matrix))
}

#[inline(always)]
pub fn u2(a_mask: Mask, b_mask: Mask, matrix: M2) -> Option<SingleOp> {
    single_op_checked!(atomic::u2::Op::new(a_mask, b_mask, matrix))
}
