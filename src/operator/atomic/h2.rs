use super::*;

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Op {
    a_mask: Mask,
    b_mask: Mask,
    ab_mask: Mask,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: Mask, b_mask: Mask) -> Self {
        Self {
            a_mask,
            b_mask,
            ab_mask: a_mask | b_mask,
        }
    }
}

impl crate::sealed::Seal for Op {}

impl super::NativeCpuOp for Op {
    #[inline(always)]
    fn native_cpu_op(&self, psi: &[C], idx: Mask) -> C {
        let mut psi = (
            psi[idx],
            psi[idx ^ self.a_mask],
            psi[idx ^ self.b_mask],
            psi[idx ^ self.ab_mask],
        );
        if idx & self.a_mask != 0 {
            psi.0 = -psi.0;
            psi.2 = -psi.2;
        }
        if idx & self.b_mask != 0 {
            psi.0 = -psi.0;
            psi.1 = -psi.1;
        }
        (psi.0 + psi.1 + psi.2 + psi.3).scale(0.5)
    }
}

impl AtomicOp for Op {
    fn name(&self) -> String {
        format!("H{}", self.a_mask | self.b_mask)
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
            && self.b_mask.count_ones() == 1
            && self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> Mask {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H2(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H2(self)
    }
}

#[cfg(test)]
#[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O_5: C = C { re: 0.5, im: 0.0 };

    let op: SingleOp = Op::new(0b01, 0b10).into();
    assert_eq!(op.name(), "H3");
    assert_eq!(
        op.matrix(2),
        [
            [O_5, O_5, O_5, O_5],
            [O_5, -O_5, O_5, -O_5],
            [O_5, O_5, -O_5, -O_5],
            [O_5, -O_5, -O_5, O_5]
        ]
    );
}
