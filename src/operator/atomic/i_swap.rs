use super::*;

op_impl!{d ab_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        if (idx & self.ab_mask).count_ones() & 1 == 1 {
            let psi = psi[idx ^ self.ab_mask];
            if self.dagger {
                C{ re: psi.im, im: -psi.re }
            } else {
                C{ re: -psi.im, im: psi.re }
            }
        } else {
            psi[idx]
        }
    }

    fn name(&self) -> String {
        format!("iSWAP{}", self.ab_mask)
    }

    fn is_valid(&self) -> bool {
        self.ab_mask.count_ones() == 2
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ dagger: !self.dagger, ..*self })
    }
}

#[cfg(test)] #[test]
fn tests() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };
    const i: C = C{ re: 0.0, im: 1.0 };

    let op: SingleOp = Op::new(0b11).into();
    let op = op.dgr();
    assert_eq!(op.name(), "iSWAP3");
    assert_eq!(op.matrix(2),
               [   [I,  O,  O,  O],
                   [O,  O,  -i, O],
                   [O,  -i, O,  O],
                   [O,  O,  O,  I]  ]);
}