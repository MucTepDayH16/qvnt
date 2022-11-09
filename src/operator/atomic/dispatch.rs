#![allow(clippy::upper_case_acronyms)]

use std::fmt;

use super::*;
use crate::{backend::single_thread::SingleThreadOp, math::Mask};

type Id = id::Op;
type X = x::Op;
type RX = rx::Op;
type RXX = rxx::Op;
type Y = y::Op;
type RY = ry::Op;
type RYY = ryy::Op;
type Z = z::Op;
type S = s::Op;
type T = t::Op;
type RZ = rz::Op;
type RZZ = rzz::Op;
type Phi = phi::Op;
type U1 = u1::Op;
type U2 = u2::Op;
type H1 = h1::Op;
type H2 = h2::Op;
type Swap = swap::Op;
type ISwap = i_swap::Op;
type SqrtSwap = sqrt_swap::Op;
type SqrtISwap = sqrt_i_swap::Op;

#[::dispatch::enum_dispatch(AtomicOpDispatch)]
pub trait AtomicOp: Clone + PartialEq + crate::sealed::Seal {
    fn name(&self) -> String;

    fn is_valid(&self) -> bool {
        true
    }

    fn acts_on(&self) -> Mask;

    fn this(self) -> AtomicOpDispatch;

    fn dgr(self) -> AtomicOpDispatch;
}

pub(crate) trait NativeCpuOp: Sync + Send + AtomicOp {
    fn native_cpu_op(&self, psi: &[C], idx: N) -> C;
}

#[::dispatch::enum_dispatch]
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
    Phi,
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
