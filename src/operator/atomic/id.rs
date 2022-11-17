use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op;

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        psi[idx]
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        "Id".to_string()
    }

    fn acts_on(&self) -> Mask {
        0
    }

    fn this(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Id(self)
    }

    fn dgr(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Id(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0 };
    const I: C = C { re: 1.0, im: 0.0 };

    let op: SingleOp = Op.into();
    assert_eq!(op.name(), "Id");
    assert_eq!(op.matrix(1), [[I, O], [O, I]]);
}
