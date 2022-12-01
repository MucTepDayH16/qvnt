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
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        if (idx & self.a_mask).count_ones() & 1 == 1 {
            -psi[idx]
        } else {
            psi[idx]
        }
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("Z{}", self.a_mask)
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Z(self)
    }

    fn dgr(self) -> dispatch::AtomicOpDispatch {
        dispatch::AtomicOpDispatch::Z(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0 };
    const I: C = C { re: 1.0, im: 0.0 };

    let op: SingleOp = Op::new(0b1).into();
    assert_eq!(op.name(), "Z1");
    assert_eq!(op.matrix(1), [[I, O], [O, -I]]);
}
