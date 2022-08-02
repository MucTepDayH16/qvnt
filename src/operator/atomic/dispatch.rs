use std::fmt;

use super::*;

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
type H1 = h1::Op;
type H2 = h2::Op;
type Swap = swap::Op;
type ISwap = i_swap::Op;
type SqrtSwap = sqrt_swap::Op;
type SqrtISwap = sqrt_i_swap::Op;

#[::dispatch::enum_dispatch(AtomicOpDispatch)]
pub(crate) trait AtomicOp: Clone + PartialEq + Sync + Send {
    fn atomic_op(&self, psi: &[C], idx: N,) -> C;

    fn for_each(&self, psi_i: &[C], psi_o: &mut [C], ctrl: N,) {
        if ctrl != 0 {
            psi_o.into_iter().enumerate().for_each(|(idx, psi,)| {
                *psi = if !idx & ctrl == 0 {
                    self.atomic_op(psi_i, idx,)
                } else {
                    psi_i[idx]
                }
            },)
        } else {
            psi_o
                .into_iter()
                .enumerate()
                .for_each(|(idx, psi,)| *psi = self.atomic_op(psi_i, idx,),)
        }
    }

    #[cfg(feature = "cpu")]
    fn for_each_par(&self, psi_i: &[C], psi_o: &mut [C], ctrl: N,) {
        use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

        if ctrl != 0 {
            psi_o.into_par_iter().enumerate().for_each(|(idx, psi,)| {
                *psi = if !idx & ctrl == 0 {
                    self.atomic_op(psi_i, idx,)
                } else {
                    psi_i[idx]
                }
            },)
        } else {
            psi_o
                .into_par_iter()
                .enumerate()
                .for_each(|(idx, psi,)| *psi = self.atomic_op(psi_i, idx,),)
        }
    }

    fn name(&self,) -> String;

    fn is_valid(&self,) -> bool {
        true
    }

    fn acts_on(&self,) -> N;

    fn this(self,) -> AtomicOpDispatch;

    fn dgr(self,) -> AtomicOpDispatch;
}

#[::dispatch::enum_dispatch]
#[derive(Clone, PartialEq,)]
pub(crate) enum AtomicOpDispatch {
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
    H1,
    H2,
    Swap,
    ISwap,
    SqrtSwap,
    SqrtISwap,
}

impl fmt::Debug for AtomicOpDispatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_,>,) -> fmt::Result {
        write!(f, "Op {{ {} }}", self.name())
    }
}
