#![allow(clippy::upper_case_acronyms)]

use std::fmt;

use super::*;
#[cfg(feature = "multi-thread")]
use crate::backend::multi_thread::MultiThreadOp;
use crate::backend::single_thread::SingleThreadOp;

pub type Id = id::Op;
pub type X = x::Op;
pub type RX = rx::Op;
pub type RXX = rxx::Op;
pub type Y = y::Op;
pub type RY = ry::Op;
pub type RYY = ryy::Op;
pub type Z = z::Op;
pub type S = s::Op;
pub type T = t::Op;
pub type RZ = rz::Op;
pub type RZZ = rzz::Op;
pub type U1 = u1::Op;
pub type U2 = u2::Op;
pub type H1 = h1::Op;
pub type H2 = h2::Op;
pub type Swap = swap::Op;
pub type ISwap = i_swap::Op;
pub type SqrtSwap = sqrt_swap::Op;
pub type SqrtISwap = sqrt_i_swap::Op;

#[enum_dispatch::enum_dispatch(AtomicOpDispatch)]
pub trait AtomicOp: Clone + PartialEq + crate::sealed::Seal {
    fn name(&self) -> String;

    fn is_valid(&self) -> bool {
        true
    }

    fn acts_on(&self) -> Mask;

    fn this(self) -> AtomicOpDispatch;

    fn dgr(self) -> AtomicOpDispatch;
}

pub trait NativeCpuOp: Sync + Send + AtomicOp {
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C;
}

#[enum_dispatch::enum_dispatch]
#[derive(Clone, PartialEq)]
pub enum AtomicOpDispatch {
    Id,
    X,
    RX,
    RXX,
    Y,
    RY,
    RYY,
    Z,
    S,
    T,
    RZ,
    RZZ,
    U1,
    U2,
    H1,
    H2,
    Swap,
    ISwap,
    SqrtSwap,
    SqrtISwap,
}

impl crate::sealed::Seal for AtomicOpDispatch {}

impl fmt::Debug for AtomicOpDispatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Op").field("name", &self.name()).finish()
    }
}
