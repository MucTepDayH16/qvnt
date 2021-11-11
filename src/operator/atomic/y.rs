use super::*;

pub (crate) struct Op {
    a_mask: N,
    i_pow: N,
}

impl Op {
    #[inline(always)]
    pub fn new(a_mask: N) -> Self {
        let i_pow = !a_mask.count_ones().wrapping_add(1) as N;
        Self{ a_mask, i_pow }
    }
}

into_single_op_impl!{a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut i_pow = self.i_pow;
        if (idx & self.a_mask).count_ones() & 1 == 0 { i_pow ^= 2; }
        crate::math::rotate(psi[idx ^ self.a_mask], i_pow)
    }

    fn name(&self) -> String {
        format!("Y{}", self.a_mask)
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        self
    }
}

#[cfg(test)] #[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };
    const i: C = C{ re: 0.0, im: 1.0 };

    let op: SingleOp = Op::new(0b1).into();
    assert_eq!(op.name(), "Y1");
    assert_eq!(op.matrix(1),
               [   [O, -i],
                   [i, O]   ]);

    let op: SingleOp = Op::new(0b11).into();
    assert_eq!(op.name(), "Y3");
    assert_eq!(op.matrix(2),
               [   [O,  O,  O,  -I  ],
                   [O,  O,  I,  O   ],
                   [O,  I,  O,  O   ],
                   [-I, O,  O,  O   ]   ]);
}