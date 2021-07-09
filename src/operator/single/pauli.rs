use crate::operator::{atomic, single::*};
use crate::math::{C, R, N};

#[inline(always)]
pub (crate) fn x(a_mask: N) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::x::Op::new(a_mask))
}

#[inline(always)]
pub (crate) fn y(a_mask: N) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::y::Op::new(a_mask))
}

#[inline(always)]
pub (crate) fn z(a_mask: N) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::z::Op::new(a_mask))
}

#[inline(always)]
pub (crate) fn s(a_mask: N) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::s::Op::new(a_mask))
}

#[inline(always)]
pub (crate) fn t(a_mask: N) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::t::Op::new(a_mask))
}

#[inline(always)]
pub (crate) fn phi(phases: Vec<(R, N)>) -> SingleOp {
    SingleOp::from_atomic_unchecked(atomic::phi::Op::new(phases))
}