use super::*;

pub (crate) struct Op {
    ab_mask: N,
}

simple_op_impl!{ab_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.ab_mask).count_ones() & 1 == 1 {
            psi[idx ^ self.ab_mask]
        } else {
            psi[idx]
        }
    }

    fn name(&self) -> String {
        format!("SWAP{}", self.ab_mask)
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
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

    let op = SingleOp::from_atomic(Op::new(0b11)).unwrap();
    assert_eq!(op.name(), "SWAP3");
    assert_eq!(op.matrix(2),
               [   [I, O, O, O],
                   [O, O, I, O],
                   [O, I, O, O],
                   [O, O, O, I]   ]);
}