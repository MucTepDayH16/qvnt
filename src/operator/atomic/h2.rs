use super::*;

#[derive(Clone, Copy)]
pub (crate) struct Op {
    a_mask: N,
    b_mask: N,
    ab_mask: N,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: N, b_mask: N) -> Self {
        Self{ a_mask, b_mask, ab_mask: a_mask | b_mask }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut psi = (psi[idx],
                   psi[idx ^ self.a_mask],
                   psi[idx ^ self.b_mask],
                   psi[idx ^ self.ab_mask]);
        if idx & self.a_mask != 0 { psi.0 = -psi.0; psi.2 = -psi.2; }
        if idx & self.b_mask != 0 { psi.0 = -psi.0; psi.1 = -psi.1; }
        (psi.0 + psi.1 + psi.2 + psi.3).scale(0.5)
    }

    fn name(&self) -> String {
        format!("H{}", self.a_mask | self.b_mask)
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
            && self.b_mask.count_ones() == 1
            && self.ab_mask.count_ones() == 2
    }

    fn acts_on(&self) -> N {
        self.ab_mask
    }

    fn this(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H2(self)
    }

    fn dgr(self) -> AtomicOpDispatch {
        AtomicOpDispatch::H2(self)
    }
}

#[cfg(test)] #[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O_5: C = C{ re: 0.5, im: 0.0 };

    let op: SingleOp = Op::new(0b01, 0b10).into();
    assert_eq!(op.name(), "H3");
    assert_eq!(op.matrix(2),
               [   [O_5, O_5, O_5, O_5],
                   [O_5, -O_5, O_5, -O_5],
                   [O_5, O_5, -O_5, -O_5],
                   [O_5, -O_5, -O_5, O_5]   ]);
}