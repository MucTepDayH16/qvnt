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

#[enum_dispatch::enum_dispatch(AtomicOpDispatch)]
pub (crate) trait AtomicOp: Clone + Sync + Send {
    fn atomic_op(&self, psi: &[C], idx: N) -> C;

    fn get_dispatch(self) -> Box<dyn Fn(&[C], N) -> C + Sync>
        where Self: 'static
    {
        Box::new(move |psi, idx| self.atomic_op(psi, idx))
    }

    fn name(&self) -> String;

    fn is_valid(&self) -> bool {
        true
    }

    fn acts_on(&self) -> N;

    fn this(self) -> AtomicOpDispatch;

    fn dgr(self) -> AtomicOpDispatch;
}

#[enum_dispatch::enum_dispatch]
#[derive(Clone)]
pub (crate) enum AtomicOpDispatch {
    Id,
    X, RX, RXX,
    Y, RY, RYY,
    Z, S, T, RZ, RZZ,
    Phi,
    H1, H2,
    Swap, ISwap,
    SqrtSwap, SqrtISwap,
}
