use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    a_mask: Mask,
}

impl Op {
    pub fn new(a_mask: Mask) -> Self {
        Self { a_mask }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        psi[idx ^ self.a_mask]
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("X{}", self.a_mask)
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::X(self)
    }

    fn dgr(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::X(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0 };
    const I: C = C { re: 1.0, im: 0.0 };

    let op: SingleOp = Op::new(0b1).into();
    assert_eq!(op.name(), "X1");
    assert_eq!(op.matrix(1), [[O, I], [I, O]]);

    let op: SingleOp = Op::new(0b01).into();
    assert_eq!(op.name(), "X1");
    assert_eq!(
        op.matrix(2),
        [[O, I, O, O], [I, O, O, O], [O, O, O, I], [O, O, I, O]]
    );
}
