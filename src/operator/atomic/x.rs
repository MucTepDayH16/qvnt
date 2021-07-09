use super::*;

pub (crate) struct Op {
    a_mask: N,
}

simple_op_impl!{a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        psi[ idx ^ self.a_mask ]
    }

    fn name(&self) -> String {
        format!("X{}", self.a_mask)
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        self
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };

    let op = SingleOp::from_atomic(Op::new(0b1)).unwrap();
    assert_eq!(op.name(), "X1");
    assert_eq!(op.matrix(1),
               [   [O, I],
                   [I, O]   ]);

    let op = SingleOp::from_atomic(Op::new(0b01)).unwrap();
    assert_eq!(op.name(), "X1");
    assert_eq!(op.matrix(2),
               [   [O,  I,  O,  O   ],
                   [I,  O,  O,  O   ],
                   [O,  O,  O,  I   ],
                   [O,  O,  I,  O   ]   ]);
}