use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    ab_mask: Mask,
}

impl Op {
    #[inline(always)]
    pub fn new(ab_mask: Mask) -> Self {
        Self { ab_mask }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        if (idx & self.ab_mask).count_ones() & 1 == 1 {
            psi[idx ^ self.ab_mask]
        } else {
            psi[idx]
        }
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("SWAP{}", self.ab_mask)
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> Mask {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::Swap(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::Swap(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C { re: 0.0, im: 0.0 };
    const I: C = C { re: 1.0, im: 0.0 };

    let op: SingleOp = Op::new(0b11).into();
    assert_eq!(op.name(), "SWAP3");
    assert_eq!(
        op.matrix(2),
        [[I, O, O, O], [O, O, I, O], [O, I, O, O], [O, O, O, I]]
    );
}
