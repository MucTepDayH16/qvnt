use super::*;

const EXP_I_PI_4: C = C{ re: crate::math::FRAC_1_SQRT_2, im: crate::math::FRAC_1_SQRT_2 };

op_impl!{d a_mask}

impl AtomicOp for Op {
    fn atomic_op(&self, psi: &[C], idx: N) -> C {
        let mut count = (idx & self.a_mask).count_ones() as usize;
        if self.dagger { count = (!count).wrapping_add(1); }
        let psi = crate::math::rotate(psi[idx], count >> 1);
        if count & 1 == 1 {
            EXP_I_PI_4 * psi
        } else {
            psi
        }
    }

    fn name(&self) -> String {
        format!("T{}", self.a_mask)
    }

    fn dgr(self: Ptr<Self>) -> Ptr<dyn AtomicOp> {
        Ptr::new(Self{ dagger: !self.dagger, ..*self })
    }
}

#[cfg(test)] #[test]
fn matrix_repr() {
    use crate::operator::single::*;

    const O: C = C{ re: 0.0, im: 0.0 };
    const I: C = C{ re: 1.0, im: 0.0 };

    let op: SingleOp = Op::new(0b1).into();
    let op = op.dgr();
    assert_eq!(op.name(), "T1");
    assert_eq!(op.matrix(1),
               [   [I, O                ],
                   [O, EXP_I_PI_4.conj()]   ]);
}