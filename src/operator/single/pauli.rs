use std::borrow::Borrow;
use crate::{
    operator::{atomic, single::*},
    math::{C, R, N},
};

#[inline(always)]
pub (crate) fn x(a_mask: N) -> SingleOp {
    atomic::x::Op::new(a_mask).into()
}

#[inline(always)]
pub (crate) fn y(a_mask: N) -> SingleOp {
    atomic::y::Op::new(a_mask).into()
}

#[inline(always)]
pub (crate) fn z(a_mask: N) -> SingleOp {
    atomic::z::Op::new(a_mask).into()
}

#[inline(always)]
pub (crate) fn s(a_mask: N) -> SingleOp {
    atomic::s::Op::new(a_mask).into()
}

#[inline(always)]
pub (crate) fn t(a_mask: N) -> SingleOp {
    atomic::t::Op::new(a_mask).into()
}

#[inline(always)]
pub (crate) fn phi(phases: Vec<(R, N)>) -> SingleOp {
    atomic::phi::Op::new(phases).into()
}