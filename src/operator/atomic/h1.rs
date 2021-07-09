use super::*;

const SQRT_1_2: R = crate::math::SQRT_2 * 0.5;

pub (crate) struct Op {
    a_mask: N,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: N) -> Self {
        Self{ a_mask }
    }
}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut psi = (psi[ idx ],
                       psi[ idx ^ self.a_mask ]);
        if idx & self.a_mask != 0 { psi.0 = -psi.0 };
        (psi.0 + psi.1).scale(SQRT_1_2)
    }

    fn name(&self) -> String {
        format!("H{}", self.a_mask)
    }

    fn is_valid(&self) -> bool {
        self.a_mask.count_ones() == 1
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        self
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const SQRT_1_2: C = C{ re: crate::math::SQRT_2 * 0.5, im: 0.0 };

    let op = SingleOp::from_atomic(Op::new(0b1)).unwrap();
    assert_eq!(op.name(), "H1");
    assert_eq!(op.matrix(1),
               [   [SQRT_1_2, SQRT_1_2],
                   [SQRT_1_2, -SQRT_1_2]    ]);
}