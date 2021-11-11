use crate::{math::{C, R, N}, operator::{atomic, single::*}};

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

#[inline(always)]
pub fn phi(phases: Vec<(R, N)>) -> SingleOp {
    atomic::phi::Op::new(phases).into()
}
