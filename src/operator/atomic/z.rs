use super::*;

pub (crate) struct Op {
    a_mask: N,
}

simple_op_impl!{a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.a_mask).count_ones() & 1 == 1 {
            -psi[idx]
        } else {
            psi[idx]
        }
    }

    fn name(&self) -> String {
        format!("Z{}", self.a_mask)
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
    assert_eq!(op.name(), "Z1");
    assert_eq!(op.matrix(1),
               [   [I, O],
                   [O, -I]   ]);
}