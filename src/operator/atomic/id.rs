pub use super::*;

#[derive(Clone, Copy, Eq, PartialEq,)]
pub(crate) struct Op;

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N,) -> C {
        psi[idx]
    }

    fn name(&self,) -> String {
        "Id".to_string()
    }

    fn acts_on(&self,) -> N {
        0
    }

    fn this(self,) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Id(self,)
    }

    fn dgr(self,) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Id(self,)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0, };
    const I: C = C { re: 1.0, im: 0.0, };

    let op: SingleOp = Op.into();
    assert_eq!(op.name(), "Id");
    assert_eq!(op.matrix(1), [[I, O], [O, I]]);
}
