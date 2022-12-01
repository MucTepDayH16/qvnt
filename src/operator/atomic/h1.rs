use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    a_mask: Mask,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: Mask) -> Self {
        Self { a_mask }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        let mut psi = (psi[idx], psi[idx ^ self.a_mask]);
        if idx & self.a_mask != 0 {
            psi.0 = -psi.0
        };
        (psi.0 + psi.1).scale(FRAC_1_SQRT_2)
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("H{}", self.a_mask)
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
    }

    fn acts_on(&self) -> Mask {
        self.a_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H1(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H1(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const SQRT_1_2: C = C {
        re: FRAC_1_SQRT_2,
        im: 0.0,
    };

    let op: SingleOp = Op::new(0b1).into();
    assert_eq!(op.name(), "H1");
    assert_eq!(op.matrix(1), [[SQRT_1_2, SQRT_1_2], [SQRT_1_2, -SQRT_1_2]]);
}
